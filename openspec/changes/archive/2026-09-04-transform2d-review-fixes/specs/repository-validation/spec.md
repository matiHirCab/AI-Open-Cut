## ADDED Requirements

### Requirement: Portable Transform2D correctness and required native coverage
Font-metric unit tests MUST use checked-in licensed fixtures without host font dependencies. Native Transform2D tests MUST use explicitly configured FFmpeg, FFprobe, and font paths for every subprocess; absent optional configuration SHALL skip only native cases, while partial configuration and missing required dependencies MUST fail. The protected Linux native parity job MUST execute the Transform2D integration target alongside existing golden and headless lifecycle tests, with matching policy validation.

#### Scenario: Run ordinary correctness on each platform
- **WHEN** Windows, Linux, or macOS correctness runs without native configuration
- **THEN** font and non-native tests run without installed rendering tools or system fonts

#### Scenario: Honor explicit native configuration
- **WHEN** valid absolute tools are configured but unavailable on PATH
- **THEN** native Transform2D tests execute successfully using those paths

#### Scenario: Reject incomplete required execution
- **WHEN** native configuration is partial or required mode lacks usable dependencies
- **THEN** the suite fails rather than silently skipping

#### Scenario: Protect native coverage
- **WHEN** the required Transform2D CI command is missing, altered, or neutralized
- **THEN** repository policy validation fails
