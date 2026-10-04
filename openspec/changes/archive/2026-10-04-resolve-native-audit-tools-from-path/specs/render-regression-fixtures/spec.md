## ADDED Requirements

### Requirement: Usable explicit requested-origin native tools
The requested-origin native audit gate MUST preserve explicit configured FFmpeg and FFprobe executable values, accepting both usable executable paths and command names resolved through the process PATH by the existing Renderer readiness contract. It MUST retain readable actual-file deterministic font validation and unchanged required-mode missing/partial configuration behavior. Missing or unusable tools or fonts MUST fail the selected gate before native rendering; no dependency substitution or required-test skipping is permitted. All existing independent requested-origin expectations, equations, tolerances, render operations and budgets MUST remain unchanged.

#### Scenario: Execute with configured command names
- **WHEN** both required flags select the full native animation suite with usable ffmpeg and ffprobe PATH command names and the deterministic font
- **THEN** all requested-origin native controls execute and pass alongside the existing animation cases without a working-directory executable-file requirement

#### Scenario: Reject unusable supplied tools or font
- **WHEN** supplied configuration names a nonexistent FFmpeg executable, nonexistent FFprobe executable, or missing/unreadable font
- **THEN** readiness fails before native rendering with a concrete dependency diagnostic, preserving configured values and the required gate instead of substituting or skipping
