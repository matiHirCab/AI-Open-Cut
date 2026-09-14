/// Construct historical fixtures without fields introduced by font activation.
/// The dedicated font-resolution suite separately tests illegal source fields.
#[cfg(test)]
fn clear_legacy_font_fields(value: &mut serde_json::Value) {
    if value["schemaVersion"].as_u64().is_some_and(|version| version < 19) {
        value.as_object_mut().unwrap().remove("fonts");
        fn strip(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Object(object) => {
                    object.remove("fontBinding");
                    for child in object.values_mut() { strip(child); }
                }
                serde_json::Value::Array(values) => { for child in values { strip(child); } }
                _ => {}
            }
        }
        strip(value);
    }
}
