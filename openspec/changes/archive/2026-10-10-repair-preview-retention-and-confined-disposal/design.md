## Context

Current main `64335ba1` has bounded completion accounting but starts preview producers without reserving preview slots or considering failed-disposal debt. Node path checks and unlink are separated by filesystem mutation windows. The audit harnesses and results are preserved in the Library report `libfile_82a158f94dd881919e923819d4deae7a` and `/tmp/opencut-review-evidence/`.

## Goals / Non-Goals

**Goals:** prevent the original repeated-producer retention growth; safely account for in-flight previews and every unsuccessfully disposed output; preserve exact inclusive successful-retention budgets; make failed close retryable and observable; prevent ancestor replacement from redirecting deletion on all three supported platforms.

**Non-Goals:** public/persisted contract changes, renderer/media semantic changes, durable handles, predecessor-process reclamation, fifth package role, broad filesystem refactor, merge/deployment.

## Decisions

### 1. Reserve preview slots and settle disposal debt before producer dispatch

JobRegistry owns admission. Reserve each admitted preview producer against the inclusive count budget before its task starts. Evict oldest settled eligible owned previews when necessary before dispatch, and await disposal before releasing their charge. Failed eviction leaves both ownership and charge intact and refuses the new producer with retryable JOB_REGISTRY_FULL. Process-local disposal debt blocks further preview producers; bounded retries attempt only owned debt under the retention serializer and permit recovery only after successful disposal. Nonpreview work retains independent admission.

Continue serialized byte admission with checked arithmetic and the unchanged 64MiB limit. When bytes are already exhausted or disposal debt is outstanding, do not start another producer. Successful retained outputs must satisfy count/byte limits; any failed/oversized/cancelled output that cannot be deleted remains explicitly charged as cleanup debt and blocks further production, instead of accumulating through retries. A physical I/O failure can prevent reclamation of an already produced oversized artifact; it must be disclosed as debt rather than claimed as successful bounded retention. Tests distinguish successful-retention budgets from the unavoidable failed-output cleanup obligation.

Alternatives: merely removing failed entries loses ownership/accounting; repeated completion-only rejection still produces more files; reserving the entire unknown byte budget for every request would evict all prior review artifacts on every preview. None meets the requested behavior. Count reservation plus fail-closed debt admission preserves ordinary retention while preventing repeated fault-path growth.

### 2. Native handle-anchored deletion inside the existing headless role

Editor-core owns the filesystem primitive and lexical validation. A private bounded stdin-only headless cleanup mode delegates to that primitive; it is not part of the public JSON-lines Request union, MCP tool set or public catalog. The bridge passes a closed owned descriptor and its existing immutable configured root, observes process termination and safe result/error, and does not implement domain/project mutation rules.

Unix: open the configured root and each project/previews directory without following links, then perform relative child operations through the held directory handles. A renamed/replaced pathname cannot redirect traversal or unlink to an external directory. Missing files remain successful idempotent disposal; unsafe links/nonregular targets are refused. The final basename is a canonical renderer-generated UUID PNG/MP4 name, never an arbitrary path.

Windows: use held native directory/file handles, refuse reparse points and retain sharing restrictions that prevent ancestor replacement while opening the target. Delete the opened file using handle-based disposition, not a subsequently resolved absolute pathname. Required native tests exercise actual replacement behavior or expected sharing refusal, Unicode/trusted-parent aliases, missing outputs and static reparse/link attacks.

Alternatives: another realpath/lstat check still races; moving by an unchecked absolute pathname can move the wrong file; adding Python/runtime/helper dependencies changes packaging; a transport-level arbitrary deletion operation expands the public attack surface. Ship the native primitive in the existing core/headless binary and preserve exactly four runtime roles.

### 3. Retryable observable graceful cleanup

Close first closes admission and settles producers; it then attempts each owned cleanup. Failures return existing safe validation/error semantics without private paths. Retain pending ownership and allow a subsequent close attempt to retry disposal, while never reopening task admission. A successful close means owned preview cleanup has settled; it cannot silently resolve while debt remains.

Alternatives: swallowing all errors conceals leaked files; forgetting debt after close prevents recovery; rejecting close before producers settle allows late outputs to escape ownership.

## Risks / Trade-offs

- Native filesystem APIs differ: actual Linux/macOS/Windows source and default-package tests are required; cross-compilation alone is insufficient.
- Closing/retry and count reservations can race completion/cancellation: one retention serializer owns admission/reclamation, and explicit unsettled reservations cannot be reclaimed on visible cancellation or TTL expiry.
- Byte size is unknown before encoding: preserve ordinary completion admission, but explicitly charge exceptional failed-output debt and block further producers. Do not claim that arbitrary failed I/O can guarantee physical deletion.
- Private cleanup transport might be mistaken for a public endpoint: keep it outside public enums/catalogs, bound its input/output and test normal protocol discovery unchanged.
- Toolchain/bootstrap or native setup may fail: preserve complete failure logs and report blocked checks without weakening gates.

## Migration Plan

No migration, schema/capability/catalog version or media oracle changes. Verify against both original reproductions, then sync/archive the approved delta after required conformance. Publish a draft PR and require terminal exact-head standard/native/release workflows. Rollback reverts only these scoped process-local/native cleanup changes.

## Open Questions

None requiring product input; concrete syscall and private adapter details must retain these invariants and receive independent conformance review. If public/persisted surfaces or the ownership boundary must change, update artifacts and seek approval before that implementation.
