# Local verification and first-head CI regression

The first #226 head `2feb2d990274a7f0bf542e597149ade428f4053a` passed local Linux implementation/runtime gates, then failed strict Clippy on macOS and Windows because `Command::Pause(bool)` was unread outside Linux. This is a newly introduced compile regression, not the historical Windows unit-test timeout. Job evidence: [macOS](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39175035786/job/117815833860), [Windows](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39175035786/job/117815833900). Full decoded, masked logs are preserved outside Git in `/tmp/opencut-226-{macos,windows}-clippy.log`. Remote completion for that head is not claimed.

## Narrow correction and fresh checks

The common admission match now explicitly reads/forwards Pause and Loop values unchanged; bounded Seek behavior is unchanged. No backend ABI, UI callback, public/persisted contract, rendering/dependency, warning policy, CI configuration, timeout, narration/#208, security or deployment input changed. The separate read-only reviewer found the correction and coverage coherent with no material blockers.

Fresh follow-up evidence on the changed desktop target:

| Check | Result |
| --- | --- |
| Rust format | PASS `/tmp/opencut-226-followup-format.log` |
| Strict workspace Clippy/all targets | PASS `/tmp/opencut-226-followup-clippy.log` |
| All desktop units | 58 passed, zero failed, two explicit native cases ignored by the default command; `/tmp/opencut-226-followup-desktop.log` |
| Explicit real playback native case | One executed pass, zero skips/failures, 12.19s; `/tmp/opencut-226-followup-native.log` |

The extended controller case observes Playing→Paused and loop on→off, supplementing its existing seek clamp, captured identity and stop/reap checks. The native case additionally sends explicit Pause(true) after resumed loop playback and verifies clock stability for 250ms within the existing 200ms tolerance. It continues to check actual red/blue pixels, real isolated output audio, seeking, loop progress, short clips, output faults and cleanup. Test binaries/source checksums are `/tmp/opencut-226-followup-{native-binary,source}.sha256`.

## Explicit evidence reuse

The full preceding workspace command passed 1,588 Cargo-target cases plus nine subprocess replays. Only desktop controller/tests changed here, so all 58 desktop units are freshly rerun; the unchanged editor-core/headless targets retain that preceding pass. This is composed full-target coverage, not a claim that a second whole-workspace command was executed. Dependency direction prevents this private desktop edit from changing core/headless behavior. Rust manifests/lockfile, pinned toolchain and relevant environment are unchanged.

The 959 TS units, canonical Rust consumers plus 645 TS contracts, 43 source integration and 40 packaged tests, and documented unchanged hermetic provider evidence remain valid for their unchanged inputs/environment. The preceding existing native desktop review/history case also remains unaffected by command admission. Original real GUI/event code, fixture, image path, backend sampling and resource ownership are unchanged; the earlier isolated 48-sample/two-wrap GUI evidence and audio/history/close observations remain valid for those paths. The changed command path receives the fresh injected-controller and actual native coverage above. No new GUI execution on this follow-up is claimed. See the original archived #213 report for complete logs, genuine screenshots and retained performance/runtime limits.

## Conformance mapping

- Play/seek/pause/loop: explicit typed admission, both-direction controller state observations and the fresh real native case.
- Captured identity/history: unchanged identity metadata and original exact history/GUI checks; the controller identity test reran.
- Truthful unavailable/failed playback: unchanged platform/backend fallback and native faults; desktop failure tests and real native failures reran.
- Optional-backend compilation: exhaustive common payload handling addresses both recorded production errors without warning allowances. Required exact-head platform CI must confirm the new build; source review/Linux checks do not claim macOS/Windows playback.

Local implementation/conformance checks pass with no unresolved material review findings. Specification closure and exact-head remote CI are recorded separately; no merge/deployment is authorized or performed.

## Verification and archival

Completeness: 4/4 tasks and all planning artifacts complete. Correctness: the modified requirement and its four scenarios are mapped above, with fresh coverage on the changed controller path and explicit reuse only for unchanged targets. Coherence: the exhaustive typed forwarding preserves existing behavior/platform fallback, strict warnings and ownership. The independent reviewer reports no material blockers; exact-head platform CI remains required after publication.

Pinned prearchive strict validation passed all 76 items. Moon ran 521 policy tests with zero failures and all strict items successfully, then rejected only `preserve-review-control-payloads` as active. No other policy failure was present. This expected exit 1 is not a passed protected gate. Local conformance is verified and ready for synchronization/archival.

Postarchive protected Moon gate and separate pinned strict validation passed: 521 policy tests, zero failures, all 75 living specifications valid. Logs: `/tmp/opencut-226-followup-final-protected.log` and `/tmp/opencut-226-followup-final-strict.log`. Only this follow-up was synchronized/archived; the original feature archive remains historical evidence. The locally verified correction is ready for the existing draft, with fresh exact-head platform/aggregate CI pending.
