## ADDED Requirements

### Requirement: Governed extended visual animation activation
Canonical checked-in catalogs MUST describe each newly active channel, strict scoped graphic/effect target shapes, compatible target kinds, finite bounds, identity defaults, interpolation, migration version, and the additive `extended_visual_animation_v1` capability. Shape targets MUST be `{kind,scope,id}` with kind `graphic_geometry`, `graphic_fill`, or `graphic_stroke`, scope equal to the owning composition, and id equal to the owning shape item ID. Effect targets MUST use kind `effect`, the same composition scope, and an ID in the owning item's effect stack. Existing targetless requests and accepted simple operations MUST retain wire meaning; the hierarchy parent-reference shape MUST remain unchanged. Other inactive catalog entries MUST remain explicitly deferred and unwritable. Governed Rust, TypeScript/Zod, headless, and MCP consumers MUST agree with accepted/rejected canonical fixtures, structural schemas, stable errors and retryability. The catalog version/digest and contract ownership map MUST be updated under designated CODEOWNER review. Capability reporting MUST advertise activation only when the complete renderer support is available; unavailable render tooling MUST preserve existing renderer readiness behavior.

#### Scenario: Compare all governed consumers
- **WHEN** canonical fixtures contain valid and malformed scalar, compound, scoped-target, crop/effect, and migration examples
- **THEN** every governed consumer agrees on typed acceptance and rejection without coercion or duplicated domain validation

#### Scenario: Discover supported and deferred behavior
- **WHEN** a client reads capability and channel metadata
- **THEN** it can distinguish the exact extended subset from particle/source/pan and other deferred channels, while all prior identifiers and error retryability remain stable
