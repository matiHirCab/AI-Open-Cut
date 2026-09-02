## ADDED Requirements

### Requirement: EvaluatedScene-only production planner
The production render planner MUST accept only editor-core evaluated instructions, process-local resource bindings, prepared resources, output dimensions and timing, and render intent, and repository architecture checks MUST reject planner or renderer-facade code that enumerates or pattern-matches persisted project, track, timeline-item, transition, text-item, or asset records to reconstruct scene semantics.

#### Scenario: Detect reintroduced project traversal
- **WHEN** production render-planning or renderer-facade code imports or pattern-matches persisted timeline record types or directly reads project tracks or assets for semantic planning
- **THEN** the editor-core architecture test fails with the forbidden owner and source file

#### Scenario: Permit canonical evaluation input
- **WHEN** the renderer facade submits an immutable project or materialized draft snapshot to the canonical evaluator and passes only its scene-and-binding result downstream
- **THEN** architecture enforcement accepts the boundary and the planner can be tested without filesystem or process execution

#### Scenario: Preserve infrastructure boundaries
- **WHEN** a frame, range, draft, or export render is prepared and executed
- **THEN** scene evaluation, path-safe resource preparation, backend planning, process execution, and artifact publication remain separately testable through their existing inward dependency direction
