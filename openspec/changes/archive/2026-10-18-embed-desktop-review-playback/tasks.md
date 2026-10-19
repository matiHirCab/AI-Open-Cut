## 1. Scope and approved design
- [x] 1.1 Refresh #213 dependencies/comments/PRs, establish independent files and probe actual GUI/audio capability without accessing #208.
- [x] 1.2 Approve proposal, delta, design and verification under delegated authority before implementation.

## 2. Playback implementation and scenario tests
- [x] 2.1 Implement bounded controller, metadata/time/clamp/generation behavior and injected failure tests.
- [x] 2.2 Implement optional Linux libmpv software/audio worker, safe typed ABI ownership, bounded frame/admission and deterministic cleanup; test unsupported/missing/codec/native failure paths.
- [x] 2.3 Embed controls/images/time/loop/errors/fallback in review panel, preserve stale identity and explicitly evict old GUI resources; test replacement/navigation/close.
- [x] 2.4 Add independent synthetic native/GUI fixture, require actual pixels/audio/pause/seek/loop/repeated cleanup evidence with 200ms timing tolerance, and document limits/readiness.

## 3. Required verification
- [x] 3.1 Pass Rust format, strict workspace Clippy/all targets, full workspace and affected explicit native/desktop build checks.
- [x] 3.2 Pass TS type/lint/unit, canonical parity, source MCP integration, native packaged smoke and hermetic Python, reusing evidence only for unchanged relevant inputs/environment.
- [x] 3.3 Complete deep independent review and scenario traceability; resolve all correctness/resource/ABI findings.
- [x] 3.4 Pass strict specs, inspect prearchive protected rejection limited to this active change and verify conformance before synchronization/archival.

## Delivery
Synchronize/archive only this verified change and pass final protected/strict gates. Only after required local checks and archive, commit/push a separate verified draft PR; inspect terminal exact-head aggregate/native/platform CI before reporting completion. Keep #225 CI and user-selected review state separate. No merge/deployment; preserve #208 and unauthorized-device boundaries.
