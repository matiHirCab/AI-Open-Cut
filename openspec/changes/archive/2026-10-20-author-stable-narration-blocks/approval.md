# Specification approval

2026-10-20 20:03UTC: independent reviewer /root/review_177 explicitly approved bounded #209 implementation after two read-only review rounds and six concrete precision resolutions. The approval covers proposal/design/tasks, three delta specifications and normative wire-plan.md. It includes the internal resolved-preview seam without library relookup, exclusive leases, irreversible uncertain-outcome quarantine and semantic native binding snapshots. No executable file had changed before approval. This is not final conformance, CODEOWNER approval, merge or deployment authorization.

Approved SHA256:

```text
proposal.md 2ad957ca113580dcb2f429d6e39b3a2a60eaf5d28124302941dd78a2542448f5
design.md 7dcf1423ffeabaeaffc007b93a464a3ab0a592414ae1e3b9ccfabc1516522d01
tasks.md 481ea6ed087b64222301ee5fb15b27a3044cb861ebc6e42e7ba0aa95d56d17ce
wire-plan.md 8ac3c0c8c7f728051811f95727e3f66cb362daf862012ac9ffdbb3a2cae8f7d7
specs/narration-blocks/spec.md 2b6af4f93fcc35e220c69285d6b27663017ae187e37bcf9997d1547fd0f6121c
specs/project-persistence/spec.md 44f69c741420eca7949294edf30a2075d8b5ffbd55c1aa457d5a602e74eeed89
specs/speech-generation/spec.md 2f8237b09458f475961bdff0155af3957c85ce8bd660c28f31c19088ce36925a
```

Task checkbox updates track observed completion and do not change approved behavior. Any material behavioral precision must receive another explicit reviewer approval before code implementing it.

2026-10-20 precision approved explicitly by /root/review_177: UUID applies only to new manifest/block/group identities. Native references retain existing IDs, including m_<uuid> cues, with nonempty128-byte budget and exact type/scope lookup. Existing generators/grammars stay unchanged; add positive prefixed-cue and missing/type/scope negative cases.

2026-10-20 caption precision approved explicitly by /root/review_177: caption opt-in requires actual non-null alignment.modelId; VALIDATION_FAILED before import otherwise. CaptionSource uses actual alignment provider/model/version, never synthesis fallback or invented identity. Cues/default remain compatible with nullable alignment.modelId. Build and validate complete candidate/refs/budgets before tracked media import.

2026-10-20 CP1 compatibility precision approved explicitly by /root/review_177: portable componentSchemaVersion46 stays fixed; readonly capture accepts source46/47 with strict source raw field rules and all original no-recovery guarantees. New delta captures this compatibility; no component runtime or #229 integration.

2026-10-20 incremental corrections explicitly approved by /root/review_177: StoredManifest records required materialized markerPolicy and optional closed caption selection; shared core policy/actual alignment validates refs, reconcile compares selections without renaming cues. Await active preview+commit before close and compute uncertainty afterward; enforce whole prepared group512KiB before inference. Private validated-native-error identity distinguishes confirmed precommit INTERNAL_ERROR from unknown transport outcomes; no flag or blind code whitelist.

2026-10-20 explicit fixture approval by /root/review_177 before consumers: new canonical wire and wireInvalidCases (five structural examples and nine single-field closed/null negatives), authored from the approved wire plan independently of live outputs. /owned/*.wav examples are deserialization fixtures only; actual publication evidence comes from the real headless/WAV/FFprobe workflow. Negative consumers first accept each unmodified DTO and then reject the designated single-field mutation.

2026-10-20 reviewer-requested lifecycle corrections: preserve publicationUnknown together with cleanup errors; attempt all ownership-ordered closes; retain JobRuntimeGuard when retirement/file cleanup is unconfirmed. Tests cover lost acknowledgement plus cleanup failure and actual source EOF/SIGTERM using only test-owned worker identities. These correct the already-approved uncertainty/cleanup requirements.

2026-10-20 explicit Unicode precision approval by /root/review_177 before code: new block request trimming pins ECMAScript's U+0009..000D,0020,00A0,1680,2000..200A,2028,2029,202F,205F,3000,FEFF; excludes0085/180E/200B. Core planner and immutable span validation share the helper. Literal FEFF/NEL manual cases plus existing-resolver/native preparation and legacy blank-only rejection are required. No legacy resolver/profile/#208 changes.

### Closure sequencing precision approval

Independent reviewer `/root/review_177` explicitly approved keeping sections1–4 as the implementation/verification checklist and sections5.1/5.2 as ordered closure obligations without checkboxes. This avoids counting future synchronization, archive, final protected gate, publication or exact-head CI as completed before they occur. All original gates, separate pending CODEOWNER review, preservation rules and no merge/deploy constraint remain unchanged. Each closing state is recorded separately only after observation.

Closure-precision task artifact SHA256 (checkbox progress included): `e8f4bc1574b2114f2a4a21264bcf5853b1dae7c063833729ab6a99cb229885fc`.

2026-10-20 bounded planning precision explicitly approved by `/root/review_177` before correction: invalid caller maxTextCharacters (including0,4097 and a high safe integer) reaches core unchanged for authoritative rejection; only valid1..4096 budgets may be reduced to current provider capacity. No duplicated project validation, transport-schema narrowing or legacy resolver change. Regression must prove no preparation/inference/publication on rejection and4096-to-lower-provider-cap behavior.
