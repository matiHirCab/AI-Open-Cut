# Raster cache verification

Approval: user explicitly replied “Approve” to the four planning artifacts in this task.

## Requirement and scenario traceability

| Requirement / scenarios | Automated evidence |
| --- | --- |
| Complete identity: C1, C2 | `vector_dependencies_invalidate_and_preserve_pixels`, `text_dependencies_and_scope_are_complete`, `native_raster_cache_all_routes_conformance` |
| Complete identity: C1, C3 | `occurrence_and_composition_changes_reuse_rasters`, `warm_cache_drafts_revisions_history_and_reopen` |
| Bounded retention: B1 | `inclusive_budgets_lru_and_oversize_bypass` |
| Bounded retention: B2 | `concurrent_misses_are_immutable_and_accounted_once`, `failed_rasters_retry_and_poison_bypasses` |
| Validation: V1, V2 | `warm_cache_preserves_preflight_errors_and_cleanup`, existing font-resolution / SVG / evaluated-scene safety suites |
| Deterministic lifecycle: L1 | `native_raster_cache_all_routes_conformance`, existing native golden conformance |
| Deterministic lifecycle: L2, L3 | `warm_cache_drafts_revisions_history_and_reopen`, native alias batch fixture, existing contracts and migration suites |

## Key audit

- Scope: evaluator-produced project ID and revision, requested output width/height, raster implementation version and raster kind. No persisted traversal occurs in artifact preparation.
- Text content: plain text, effective runs/spans, authored size/color. Fonts: verified SHA-256 bindings for selected faces and profile; the current core contract admits only face index zero. Font files are still loaded and verified before lookup, including retained fonts.
- Text pixels: actual positioned glyph ID/face/cluster/coordinates/advance/color/paint layers, shaped extents, line geometry and advanced layout box/overflow, raster dimensions and offsets, effective paint/background/shadow/padding/layout styles. Alignment is represented by final glyph positions. Anchors, warnings, resource/occurrence IDs, paths and composition timing/opacity are intentionally excluded. Text raster density remains the existing one local pixel per unit; no text resolution behavior changes.
- Vector pixels: geometry, normalized SVG document, procedural grid descriptor, fill/stroke/fill-rule, contours and closure, analytic bounds, local origin, raster dimensions and composed sampling density, plus ordered recursive evaluated children. No float quantization occurs in key encoding.
- SHA-256 receives streamed typed JSON fields separated by NUL; field and sequence boundaries remain unambiguous. Only the fixed 32-byte digest is retained with immutable PAM payloads. No key text or pixels appear in Renderer debug output.
- Integration stays within existing evaluator, renderer and render_artifact ownership. Public contracts, fixtures and persisted schemas are unchanged; migration is not applicable.

## Check evidence

Logs are uncommitted external files under `C:/Users/matia/AppData/Local/Temp/`.
Commands run at repository root unless marked bridge (`apps/agent-bridge`).

| Command | Result | Log basename |
| --- | --- | --- |
| `cargo test -p opencut-editor-core --lib raster_cach` with native tools | Exit 0; 9 passed | `opencut-cache-native.log` |
| `cargo fmt --check --all` | Exit 0 | `opencut-cache-fmt.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 | `opencut-cache-clippy.log` |
| `cargo test --workspace` with native tools | Final serial exit 0; all workspace suites passed, including 330 library tests and 28 headless protocol tests | `opencut-cache-workspace.log`, `opencut-cache-workspace-serial.log` |
| Bridge `bun run typecheck` | Exit 0 | `opencut-cache-typecheck.log` |
| Bridge `bun run lint` | Exit 0; 70 files | `opencut-cache-lint.log` |
| Bridge `bun run test:unit` | Exit 0; 396 passed | `opencut-cache-test-unit.log` |
| Bridge `bun run contracts:check` | Exit 0; native parity suites and 330 TypeScript parity tests | `opencut-cache-contracts-check.log` |
| Bridge `bun run test:integration` | Exit 0; 11 passed | `opencut-cache-test-integration.log` |
| Bridge `bun run test:smoke` | Exit 0; 6 passed | `opencut-cache-test-smoke.log` |
| Bridge `bun run scripts/run-python-tests.ts` | Exit 0; 10 unittest and 5 pytest cases | `opencut-cache-python.log` |
| `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` | Exit 0; 28 items | `opencut-cache-specs.log` |
| `moon run root:openspec-validate` before archival | Exit 1 solely for this active change; policy tests 234 passed, 0 failed, all 28 specs valid | `opencut-cache-prearchive-gate.log`, `opencut-cache-prearchive-gate-final.log` |
| `moon run root:openspec-validate` after archival | Exit 0; protected policy and 28 specs pass | `opencut-cache-final-gate.log` |
| Strict all-spec validation after archival | Exit 0; 28 passed | `opencut-cache-final-specs.log` |

### Environment and earlier failures

The ambient FFmpeg rejects the existing `filter_complex_script` option. Native
verification uses the pre-existing FFmpeg 7.1.1 binaries in
`C:/Users/matia/AppData/Local/Temp/opencut-35-ffmpeg-7/ffmpeg-7.1.1-essentials_build/bin`.
The new cache fixture itself pins application font bytes. The existing reviewed
golden suite additionally requires `crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf`
(SHA-256 `ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`).
Using the application resource font for that suite caused its explicit identity
assertion to fail. That environment mistake was corrected in the passing serial workspace rerun.

The first parallel workspace run also failed the isolated process-memory sampler
and a native animation output publication. The animation test passed unchanged
in isolation (`opencut-cache-animation-diagnostic.log`, exit 0). The final workspace
rerun passed with `RUST_TEST_THREADS=1` and the reviewed font; no existing thresholds,
references, tests or production FFmpeg behavior were weakened.

### Final conformance and archival

Reviewed with the openspec-verify-change workflow after all required implementation
checks passed. The proposal, design, delta requirements, tasks, code and automated
evidence are consistent.

| Dimension | Assessment |
| --- | --- |
| Completeness | All implementation and pre-archive check tasks complete; 4 requirements and 10 scenarios mapped to automated evidence |
| Correctness | Identity/invalidation, bounded retention, validation/error precedence and shared deterministic lifecycle behavior conform |
| Coherence | Existing ownership/dependency direction preserved; no public/persisted schema, contract fixture or migration change |

Implementation references: `render_artifact/raster_cache.rs:35` owns retention;
`:80` implements checked retention, LRU, concurrent insertion and poison bypass;
`:155`, `:166`, `:247` and `:253` define scope/text/vector identity.
`renderer.rs:433` derives scope after preflight, and `:455`/`:463` route both
raster kinds through existing materialization. `evaluated_scene.rs:1307` carries
immutable project identity/revision without exposing persisted records downstream.
Paths above are relative to `crates/editor-core/src/`.

No unresolved critical findings, warnings or design mismatches. No normative
scenario is exempted from automation. The seven ignored library entries are
intentional subprocess/helper entry points, not omitted feature checks. The
native golden, cache conformance and headless native lifecycle checks all ran.
Earlier failures are superseded by the unchanged-code, fully passing serial run.

Synchronized the four requirements and ten scenarios into
`openspec/specs/raster-caching/spec.md` and archived this change on 2026-09-20.
All 18 tasks are complete. The post-archive protected Moon gate and strict all-spec
validation both passed (exit 0). No active OpenSpec change remains. Local completion
is verified; this report does not claim a remote CI run or a merged pull request.
