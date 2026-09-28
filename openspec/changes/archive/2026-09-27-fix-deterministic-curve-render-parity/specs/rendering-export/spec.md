## ADDED Requirements

### Requirement: Supported FFmpeg parameterized curve parity

Supported FFmpeg 6 and 8 configurations MUST render valid evaluated cubic Bézier and spring curves consistently from the same immutable scene across frame preview, audiovisual range preview, materialized draft preview, and final export. Equivalent timestamps and output settings MUST yield matching semantic plans and decoded output within the existing visual, audio, and timing tolerances. Correct rendering MUST include visual position, scale, opacity, and audio gain. Invalid or non-finite evaluated work MUST keep its established typed preflight error and publish no artifact. Legacy-only scenes MUST retain existing output.

#### Scenario: Render a Bézier visual through every intent
- **WHEN** a rectangle moves from position X 0 to 20 over 500 ms using a valid cubic Bézier and is rendered at 250 ms with supported FFmpeg 6 or 8
- **THEN** draft frame, committed frame, range, and export place it according to the same canonical sample within the existing decoded visual tolerance

#### Scenario: Preserve render concurrency for scenes without Bézier
- **WHEN** a scene uses only spring, simple, or legacy curves
- **THEN** the renderer retains its existing FFmpeg filter-graph thread policy and output behavior

#### Scenario: Render spring visuals and gain through every intent
- **WHEN** an immutable scene contains a valid spring visual channel and spring audio gain channel at matching fixed timestamps
- **THEN** frame, audiovisual range, materialized draft, and export plans agree and decoded visual and audio results satisfy the existing tolerances

#### Scenario: Preserve failure and legacy output
- **WHEN** a scene contains invalid parameterized work or only existing legacy curves
- **THEN** invalid work fails with its stable typed error before output side effects, while legacy-only output remains equivalent to pre-fix output
