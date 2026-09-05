## Why

Issue #20 requires editable parent transforms and bounded hierarchy validation. The current schema-9 timeline has common visual properties, Transform2D, and stacking, but no GroupItem or runtime parent reference; the foundation catalog supplies only fixture examples.

## What Changes

- Add non-drawing GroupItem nodes and optional typed, composition-scoped parent references, with editor-core enforcing reference integrity, cycles, and an inclusive 32-edge ancestor limit.
- Add group creation and parenting operations, standalone and alias-aware in atomic batches; reuse common transform and visibility updates.
- Evaluate ancestor affine transforms, opacity, visibility, and active intervals once for all render intents while retaining flat stacking and existing audio semantics.
- Migrate schema 9 to 10, including all retained history, with unparented defaults and unchanged output for existing projects.
- Govern runtime group contracts, typed consumers, capability reporting, deterministic fixtures, and documentation together.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `timeline-editing`: Groups, scoped parenting, graph validation, lifecycle and transactional rules.
- `project-persistence`: Atomic schema-10 migration and hierarchy validation across retained history.
- `motion-graphics-architecture`: Canonical ancestor evaluation without nested compositing or a second child list.
- `motion-graphics-contracts`: Governed runtime group vocabulary, typed operations, discovery, and parity.
- `rendering-export`: Shared grouped visual output and fail-before-side-effect evidence.

## Impact

Core model, validation, timeline mutations, migrations, evaluation, and render preparation; typed headless and MCP adapters; canonical catalogs and Rust/TypeScript parity; affected desktop exhaustive matches; documentation and tests. No new dependency edge or provider behavior is needed. Issues #17 and #18 are represented by the living common-visual and Transform2D requirements already present in this checkout.

Public operations and optional fields are additive and preserve existing simple requests and protocol major version. Schema 10 is a versioned persisted upgrade: older readers must reject it. The new group variant requires capability-aware clients to understand grouped projects; it does not claim that an old closed-union decoder can read new content.

## Non-goals

Components/instances, masks/effects, group raster isolation, animation of groups, audio parenting, automatic keep-world reparenting, recursive group duplication/deletion, and desktop group-authoring UI. Remaining motion-graphics vocabulary stays fixture-only.

## Approval

The user explicitly approved these artifacts with “Approve” in the task conversation on 2026-09-04. Implementation may proceed through tasks.md; implementation verification and governed-contract review remain required.

The user explicitly approved the three-fix implementation plan in this conversation on 2026-09-05. The following artifact refinements record that approved scope; public contracts and schema version remain unchanged.

## Designated contract review approval

On 2026-09-05, the user replied "Approve" to the explicit request to approve the governed contract implementation as designated owner @matiHirCab, after the final correction verification report and passing contract parity results were presented. This approval covers the canonical group, headless/MCP and ownership catalogs, governed native and TypeScript consumers, and their parity evidence.
