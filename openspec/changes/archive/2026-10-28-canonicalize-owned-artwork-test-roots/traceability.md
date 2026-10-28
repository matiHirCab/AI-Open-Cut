# Scenario traceability

| Modified requirement scenario | Coverage |
| --- | --- |
| Hermetic source/package | Existing mandatory combined actual-core workflow and full source/package suites; supplemental aliased TMPDIR with interpreter-only PATH; fake output assertion unchanged. |
| Explicit genuine source/package | Existing genuine selector and independent64x64 RGBA decode/common assertions unchanged; separate focused source/full package runs. |
| Invalid configuration/fixture | Existing backend tests retain unknown/malformed/changed/oversize/foreign/leaf-link/configuration assertions. Probe and shared backend byte-identical. New alias case adds allowlisted foreign-file and same-root/foreign linked-child negatives plus foreign-byte preservation. |
| Owned temporary-root ancestor alias | New cross-platform backend unit case creates only owned physical-parent/alias directories, builds PNG/WAV fixtures through the alias, requires canonical paths and actual metadata, and rejects explicit alias input. Windows uses an owned junction; other hosts directory symlink. No new skip. |

Only two executable files change: tests/artwork-durable-backends.test.ts (fixture setup parent/canonical created root and isolated regression) and tests/artwork-durable-workflow.ts (canonical created root before paths/config/hash keys). Alias unit regression executes only the existing private Node probe, never native disposal/crash/descendant pathways. Supplementary combined acceptance selects only the existing artwork/narration/durable-render scenario; no disposal fixture is selected. Existing required aggregate registrations remain unchanged. No restricted Windows boundary investigation or native code change.
