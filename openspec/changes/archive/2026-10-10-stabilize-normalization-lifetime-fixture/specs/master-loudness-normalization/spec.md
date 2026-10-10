## MODIFIED Requirements

### Requirement: Isolated bounded normalization lifetime
Every original-capture/measurement/measured-processing/verification/optional-correction child SHALL inherit existing request-owned groups, bounded concurrent diagnostic drainage, exact finite frames, local kill/wait and owned workspace lifetime. At most one correction pair SHALL occur; no unbounded retry/pass/file/result work is allowed. Cancellation, deadline or shutdown MUST kill/reap all owned phases, clean only owned temporary/partial work, preserve overlapping jobs/existing completed outputs and leave project/history unchanged. Failed normalization MUST not publish a completed summary/artifact.

#### Scenario: Cancel every owned phase
- **WHEN** a request is cancelled, times out or shuts down during each active normalization phase
- **THEN** its actual descendants disappear, owned private/partial files are cleaned and unrelated jobs/outputs/state/history are preserved

#### Scenario: Preserve reusable and overlapping requests
- **WHEN** sequential and overlapping normalized preview/export/analysis work is dispatched
- **THEN** existing reusable/one-shot isolation remains valid and one cancellation cannot terminate another request

#### Scenario: Fail workspace stream process and publication safely
- **WHEN** injected workspace/filter/PCM/metric/exit/metadata/publication faults occur
- **THEN** the existing safe error boundary publishes no partial final result and retains every unrelated byte and canonical transaction

#### Scenario: Preserve cold completion and actual completed outputs during controlled lifecycle faults
- **WHEN** a lifecycle case first completes one real cold normalization request and then arms each owned phase for cancellation, deadline or shutdown
- **THEN** the cold request proves successful finite frame completion without changing current/history, controlled work proves actual backend/descendant entry and exact PID absence under the unchanged1500ms phase wait and2000ms deadline, and the previously completed analysis JSON plus unrelated outputs remain byte-identical with all original nine-phase and twenty-seven-mode cases retained

#### Scenario: Separately bound cold preparation and controlled lifecycle execution
- **WHEN** each of the twenty-seven lifecycle cases creates its fresh project and proves cold completion before exercising its owned phase
- **THEN** preparation has its own explicit5000ms hook limit and any setup failure fails the corresponding case, while controlled execution retains the original5000ms test limit,1500ms entry wait,2000ms deadline, exact PID absence, immutable prior outputs, actual overlap and successful reuse without retries or shared fixtures

#### Scenario: Direct native lifecycle fixture preserves bounded proof
- **WHEN** mandatory lifecycle verification prepares its private compiled backend and executes controlled normalization phases
- **THEN** actual directly invoked backend and native descendant PIDs are alive before cancellation, all original54 success/lifetime/fault cases and existing1500ms/2000ms/5000ms bounds remain required, compilation failure refuses setup without skips/fallback, and readiness failure retains actual phase/outcome diagnostics without converting failure to success or claiming native media correctness

