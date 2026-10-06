## Why

Issue #54 requests effect order that is persisted, observable, validated, editable and undoable across public edits and shared rendering. Merged #43 already implements the typed ordered stack, stable effect IDs/targets, limits, migration and shared runtime, and living visual-effects requirements already mandate those semantics. Existing native helpers distinguish ordered effects, but the actual MCP smoke workflow currently uses one zero-radius blur and does not prove public reorder→undo/redo→reopen with independently expected noncommuting pixels. This change supplies that concrete conformance/documentation gap; it does not reimplement an existing feature.

## What Changes

- Add a canonical noncommutative vignette/tint fixture with independent local-coordinate/premultiplied-linear expected pixels and stable effect ID animation targets.
- Prove standalone and creation-alias batch effects assignment/reordering, omitted/empty semantics, draft materialization/commit, undo/redo and process/core reopen through actual public APIs/protocols and native render output.
- Extend exact current/retained history and genuine already-supported source migration conformance using existing adoption, including invalid retained generations and full atomic byte/resource preservation.
- Add actual configured native frame/range/draft/export evidence with shape asymmetry, noncentral anchor, inherited/component occurrence, unchanged audio/time behavior, existing bounds/errors and unchanged identity/default output.
- Document existing array-order and stable-ID semantics, fixture math and reproducible native commands; preserve public API/schema/capabilities/catalog digest/algorithms/budgets exactly.

## Capabilities

### New Capabilities

- None. No runtime/model/capability or operation is introduced.

### Modified Capabilities

- `visual-effects`: explicit independent public/native order evidence for the already implemented stack.
- `rendering-export`: all-intent noncommuting effect-order lifecycle proof.
- `project-persistence`: targeted existing-generation/history/adoption conformance without a new migration.
- `contract-governance`: canonical shared conformance witness and exact unchanged public catalog evidence.

## Impact

Tests/fixture ownership/docs and accepted specification evidence, plus the independently approved narrow atomic-adoption transaction routing correction. Existing contracts/extended-visual-animation-v1.json owns effects; append conformance cases without changing current marker/version/field/operation behavior. Candidate owners are new core integration ordered_effect_stacks.rs, current headless protocol tests, bridge extended-visual workflow/tests, canonical ownership/CODEOWNER and effect/render-regression docs. No production model/validator/compositor/migration algorithm changes are authorized; only the reviewed existing transaction staging selectors may change. Any detected runtime semantic defect requires a separate explicitly approved scope amendment before fixes. Reconciled against exact verified #53 and approved independently before implementation; approval.md records scope and predecessor proof.

Additional modified capability: repository-validation, preserving verified matte/blend mandatory conformance and adding actual ordered-effect core/public native commands with exact fail-closed guards. This fifth delta only strengthens test execution; no runtime/public/schema semantics are changed.

Approved scope amendment: atomic ordered-effect edit adoption uses the existing staged transaction selector only, as detailed in design.md. Independent Sol Medium reviewer explicitly approved this amendment before production edits; approval.md records the authorization.
