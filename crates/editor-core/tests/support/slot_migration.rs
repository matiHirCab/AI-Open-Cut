// Included inside store::tests for existing commit-point fault injection.
#[test]
fn legacy_slot_edits_preserve_precommit_bytes_and_recover_complete_generations() {
    for route in 0..3 {
        for phase in [
            PersistencePhase::BeforeFontPublish, PersistencePhase::AfterFontPublish,
            PersistencePhase::BeforeJournal, PersistencePhase::AfterJournal,
            PersistencePhase::AfterProject, PersistencePhase::AfterHistory,
            PersistencePhase::AfterDraftUpdates, PersistencePhase::AfterDraftCleanup,
            PersistencePhase::AfterJournalCleanup,
        ] {
            let (core, _root) = core();
            let id=core.create_project("Legacy slot fault",ProjectSettings::default()).unwrap().project_id;
            let track=core.get_project(&id).unwrap().tracks.iter().find(|t|t.track_type==TrackType::Overlay).unwrap().id.clone();
            let catalog:serde_json::Value=serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../contracts/solid-graphic-color-slots-v1.json"))).unwrap();
            let mut definition=catalog["definition"].clone();
            definition["slots"]=serde_json::json!([]);definition["resultAlias"]=serde_json::json!("card");
            let batch:Vec<BatchEditOperation>=serde_json::from_value(serde_json::json!([
                definition,{"operation":"add_component_instance","resultAlias":"instance","trackId":track,
                "componentId":"@card","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{}}
            ])).unwrap();
            let aliases=core.edit_batch(&id,0,batch).unwrap().aliases;
            let dir=core.paths().project_dir(&id).unwrap();
            let mut old:serde_json::Value=serde_json::from_slice(&std::fs::read(project_path(&dir)).unwrap()).unwrap();
            old["schemaVersion"]=serde_json::json!(44);
            old.as_object_mut().unwrap().remove("narrationManifests");
            std::fs::write(project_path(&dir),serde_json::to_vec(&old).unwrap()).unwrap();
            std::fs::write(history_path(&dir),serde_json::to_vec(&serde_json::json!({"undo":[old.clone()],"redo":[old]})).unwrap()).unwrap();
            let define=serde_json::json!({"operation":"component_define_slots","componentId":aliases["card"],"slots":[{
                "id":"opacity","name":"Opacity","kind":"number","required":true,"defaultValue":{"type":"number","value":0.75},
                "binding":{"targetLayerId":"shape","property":"visual.opacity"},"constraints":{}}]});
            let update=serde_json::json!({"operation":"component_instance_update","itemId":aliases["instance"],
                "componentId":aliases["card"],"startMs":50,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{}});
            let edits:Vec<EditOperation>=serde_json::from_value(if route==0 {serde_json::json!([define])}else if route==1 {serde_json::json!([update])}else{serde_json::json!([define,update])}).unwrap();
            let run=|| if route==2 {core.edit_batch(&id,1,edits.clone())}else{core.edit(&id,1,edits[0].clone())};
            let before=project_file_bytes(&dir);
            set_persistence_fault(&core,phase);
            let result=run();
            if matches!(phase,PersistencePhase::BeforeFontPublish|PersistencePhase::AfterFontPublish|PersistencePhase::BeforeJournal) {
                assert_eq!(result.unwrap_err().code,ErrorCode::InternalError,"{route}/{phase:?}");
                assert_eq!(project_file_bytes(&dir),before,"{route}/{phase:?}");
                assert!(!transaction_path(&dir).exists());
                assert_eq!(run().unwrap().revision,2);
            }else{assert_eq!(result.unwrap().revision,2,"{route}/{phase:?}");}
            let reopened=EditorCore::new(core.paths().clone());
            let project=reopened.get_project(&id).unwrap();
            assert_eq!(project.schema_version,crate::PROJECT_SCHEMA_VERSION);
            assert_eq!(project.revision,2,"{route}/{phase:?}");
            let data=serde_json::to_value(&project).unwrap();
            if route!=1 {assert_eq!(data["components"][0]["slots"][0]["id"],"opacity");}
            if route!=0 {assert_eq!(project.find_item(&aliases["instance"]).unwrap().start_ms(),50);}
            let history:History=read_json(&history_path(&dir)).unwrap();
            assert_eq!(history.undo.len(),2);assert!(history.redo.is_empty());
            assert!(history.undo.iter().all(|p|p.schema_version==crate::PROJECT_SCHEMA_VERSION));
            let stable=project_file_bytes(&dir);reopened.get_project(&id).unwrap();
            assert_eq!(project_file_bytes(&dir),stable,"{route}/{phase:?}");
            assert_no_managed_transaction_files(&dir);
        }
    }
}
