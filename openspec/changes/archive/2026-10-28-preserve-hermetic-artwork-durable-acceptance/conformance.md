# OpenSpec verify-change report

Change preserve-hermetic-artwork-durable-acceptance; spec-driven. Exact reviewer approval is in approval.md. approved-artifact-pins.json froze approved input hashes before task status edits; approved-artifact-originals.json preserves the matching original text (task original reconstructed from status-only edits and verified against its frozen hash).

| Dimension | Assessment |
| --- | --- |
| Completeness | Approval, both implementation tasks and all implementation checks complete. Task3.3 final lifecycle verification follows this report. Task4.1 publication and new-head CI are later external obligations, explicitly authorized after local archival/gates, not prospective passes. |
| Correctness | The single requirement and all3scenarios map to automated coverage in traceability.md. All required implementation checks pass, with hermetic and genuine source/package evidence separated in verification.md. |
| Coherence | Three private test files only; shared fake tool, runtime, catalogs, dependencies, workflows, profiles, timeouts, original common assertions and historical oracles remain unchanged. Explicit genuine decode remains mandatory in genuine mode. No ambient fallback/readiness skip introduced. |

No unresolved implementation/spec mismatch. Probe facts derive from hash-allowlisted bounded regular owned PNG/WAV bytes with PNG checksum/raster validation and strict PCM header/bounds checks. Configured version result is delegated; invalid or missing tools/inputs fail explicitly. Both no-ambient source/package proofs use actual native core and supplied fake FFmpeg, and assert exact fake output without pixel claims. Genuine source/package independently decode64x64 RGBA. Every original native/project/provenance/history/durable/reconnect assertion remains.

Full source46/46, hermetic package42/42, genuine package42/42, genuine source1focused, full units1188pass/15existing optional skips, native contracts533/533 and TypeScript contracts708/708, type/lint, root policy627/627 and strict81/81 pass. Unit/contracts local worker limit1 is disclosed. Prior unaffected Rust/worker/artwork evidence is reused only after input/toolchain diff audit. Linux cannot prove Windows behavior; exact new-head supported CI remains later task4.1.

Pre-archive protected gate passed538policy controls and81spec items, then rejected only this active change. That is expected rejection, not protected success. Synchronize only the approved additive durable-production-requests requirement and archive only this change; final unchanged protected gate and strict validation must pass before publication. No production lifecycle changes, PR approval/CODEOWNER acceptance, merge, deployment, readiness change or denied text/comment retry is authorized.


Final local assessment: synchronization and archival completed for this change only. Protected Moon538policy+80spec items, separate strict80/80 and final root policy627/627 all pass. Tasks1–3 complete. Publication/new-head supported CI task4.1 remains later, explicitly not claimed complete in this local snapshot. No unresolved local conformance finding remains.
