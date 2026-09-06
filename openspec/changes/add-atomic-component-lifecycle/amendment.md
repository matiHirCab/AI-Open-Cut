# Proposed clarification: validation publication boundary

Status: approved by the user on 2026-09-06: "Preserve the existing validation boundary (Recommended)". The clarification below is incorporated into the design and delta specifications.

Inspection found that validation.rs enforces stored graph, timing, slots and aggregate text limits. evaluated_scene.rs owns the expanded-occurrence, visual/audio/resource and composed-clock limits. Existing placement and duplication edits do not invoke scene evaluation before saving; ADR 0003 forbids a timeline-to-evaluated_scene dependency.

Proposed scope: retain that separation for component_instance_duplicate. Stored-domain violations fail before project/history publication. Expanded scene violations fail before render artifact preparation/publication, with no renderer process or output side effects. No new dependency edge or parallel expansion validator is introduced.

If approved, replace the component-evaluation delta scenario "Enforce existing inclusive bounds" with:

- WHEN duplication reaches versus exceeds existing timing or slot/text bounds, or contains non-finite numbers, unsafe resource forms or unknown closed-record fields
- THEN valid inclusive boundaries succeed and invalid candidates fail before project/history publication under existing core and structural validation rules.

Add scenario "Preserve expanded render preflight":

- WHEN duplicated content reaches versus exceeds existing expanded-occurrence or scene bounds
- THEN valid bounded content renders and excessive expansion fails with INVALID_ARGUMENT before render artifact preparation or backend execution, preserving the saved project/history.

Update design risks and verification/tasks to distinguish the two publication boundaries explicitly. The requested duplicated output semantics remain unchanged.
