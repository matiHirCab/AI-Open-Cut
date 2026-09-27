## ADDED Requirements

### Requirement: Lossless canonical MCP catalog representation
The canonical MCP catalog MUST remain a single manually governed, version-1 checked-in artifact. Its stored representation MAY share exact repeated schema subtrees, but the contract parity gate MUST first expand the representation deterministically and compare every registered tool's complete structural input schema, complete structural output schema, and client-visible annotations, as well as the exact tool, prompt, resource, and capability identifier sets. Schema `description` copy alone SHALL remain excluded from structural comparison. A storage-only rewrite MUST preserve the expanded pre-change catalog exactly and MUST NOT change the public MCP surface or protocol version.

#### Scenario: Expand the compact catalog without changing the public contract
- **WHEN** the compact catalog is loaded and expanded twice
- **THEN** both expansions are identical to the approved pre-change canonical catalog, including every schema, annotation, identifier, resource template, and version

#### Scenario: Detect live MCP structural drift
- **WHEN** a registered tool's structural input or output schema or annotation differs from its expanded canonical definition, including a difference in a shared schema subtree
- **THEN** the standalone contract parity gate fails and identifies the affected tool

#### Scenario: Detect supporting-surface drift
- **WHEN** a registered tool, prompt, resource name or template, or capability identifier differs from the expanded canonical catalog
- **THEN** the standalone contract parity gate fails for the mismatched surface

#### Scenario: Reject an invalid compact representation
- **WHEN** a reference is missing, cyclic, malformed, or resolves outside declared local definitions
- **THEN** catalog expansion fails before any parity comparison and cannot silently omit or weaken a tool definition

#### Scenario: Preserve existing client compatibility
- **WHEN** a client discovers or invokes the version-1 MCP surface after the catalog storage change
- **THEN** the same tools, schemas, annotations, prompts, resources, capabilities, typed failures, and revision-conflict behavior remain available with unchanged identifiers and meanings
