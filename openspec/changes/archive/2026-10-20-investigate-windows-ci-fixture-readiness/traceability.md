# Conformance map

All paths below are relative to apps/agent-bridge. Focused suites passed58 normalization and12 restart cases. Final complete-suite/gate results and delivery state are recorded in verification.md.

| Requirement/scenario | Implementation and automated evidence |
| --- | --- |
| Complete readiness: concurrent publication | fixtures/normalization-lifetime-native.rs:76 publishes a closed stage by rename; tests at270/286/299 require complete final metadata and actual stage/rename failure preservation. tests/master-normalization-lifetime.test.ts:433 executes all3 native controls; all27 original actual lifecycle cases retain PID/state/output assertions. |
| Complete readiness: no publication | tests/master-normalization-lifetime.test.ts:519 retains missing-phase refusal, unchanged1500ms entry bound and actual diagnostic/owned-request settlement. |
| Complete readiness: malformed metadata | tests/master-normalization-lifetime.test.ts:205/444 proves original SyntaxError, owned real capture abort/settlement, exact PID absence, diagnostic and unchanged authoritative/completed-output bytes. |
| Bounded real crash verification: repeated crashes | tests/job-restart.test.ts:200 retains the original four sequential real processes, three cold probes, all statuses and guard/private inventory assertions under the default5s limit; queued predecessor proof remains at241. |
| Bounded real crash verification: self-contained setup | tests/job-restart.test.ts:51/145 requires successful separately bounded5000ms bun build and actual output, and invalid-source compilation fails without source fallback. Every existing cold process invokes the resulting real-production-code bundle. |
| Bounded real crash verification: failed/ended child | tests/job-restart.test.ts:75/156/174 proves incomplete output fails after actual PID absence, live teardown closes/kills only owned children, and successor admission is refused. The unchanged10000ms helper watchdog records failure and settles through the same close boundary. |

Follow-up to the failed/ended-child control: tests/job-restart.test.ts:188 captures the single owned ChildProcess PID and waits for exact file contents inside the unchanged wait before the original live/absent assertions. This closes the control's own publication observation race. Restart12/12, full unit994/15 existing skips, types and lint passed again; details in evidence.md. The repeated-crash case now starts at206 and queued predecessor case at247; their original bodies remain byte-identical.

No public/persisted contract or workflow/governance hash changes. All original nine restart test bodies remain byte-identical to base756b7b39; all original normalization phases/faults/lifetime assertions and limits remain. Independent restart review approved implementation; main agent reviewed normalization. Exact-head Windows CI is required to validate the startup optimization; Linux evidence alone does not prove its Windows effect.
