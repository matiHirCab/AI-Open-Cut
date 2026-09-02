## MODIFIED Requirements

### Requirement: Report-only timing and memory baseline
An explicit baseline capture MUST perform exactly one discarded warm-up and three measured renders, require deterministic conformance across the measured renders, and report the fixture and environment identity, warm-up and sample counts, median scene-evaluation time, median filter-graph construction time, median frame rendering time, median audiovisual range rendering time, median export time, median total elapsed time, and maximum sampled resident memory for the test process tree using finite non-negative values and declared units and aggregation metadata. Process-tree memory MUST include recursively discovered FFmpeg and FFprobe descendants. Required CI MUST resolve the report destination independently of Cargo's test working directory, create its parent directory before capture, and use that same workspace artifact for schema validation and upload. This change MUST NOT treat captured timing or memory values as universal pass/fail budgets.

#### Scenario: Capture comparable observations
- **WHEN** a caller runs baseline capture in a fully declared environment
- **THEN** it emits a machine-readable report declaring one warm-up, three measured samples, median timing aggregation, maximum memory aggregation, process-tree memory scope, and all required fixture, tool, font, and platform metadata

#### Scenario: Publish the required Linux observation
- **WHEN** Linux CI invokes golden conformance through Cargo from the repository workspace
- **THEN** capture writes schema 2 to an absolute workspace destination whose parent exists, and the subsequent validation and artifact upload consume that same file

#### Scenario: Compare unlike environments
- **WHEN** two reports have different operating-system, architecture, tool, font, fixture, sampling, scope, or aggregation identity
- **THEN** they remain separate observations and MUST NOT be presented as a like-for-like regression comparison
