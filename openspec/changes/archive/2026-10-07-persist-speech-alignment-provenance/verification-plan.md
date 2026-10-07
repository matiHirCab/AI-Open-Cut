# Issue60 planned scenario evidence

The table retains the approved plan. Concrete implementation evidence and original failed runs are mapped in `implementation-review.md`; full required checks and exact-head CI remain pending until their recorded receipts pass. Unchecked tasks remain authoritative.

| Requirement/scenario | Planned independent evidence |
| --- | --- |
| Durable truthful speech alignment / each quality | Canonical fixed native/forced/estimated records consumed by Rust and Zod; exercise each of seven nonempty independent granularity combinations without inferring absent levels; distinct synthesis/alignment identity retained |
| Legacy unaligned synthesis | Exact old provenance fixture, no alignment emitted; existing speech commit/regeneration unchanged; no extra synthesis invocation |
| Closed timing shape | Reject unknown fields at alignment/timed-text levels, unknown quality, missing required arrays/producer metadata, null alignment, null arrays, floats/negative/unsafe JSON integers and duplicate serialized members where raw transport supports detection |
| Canonical bounded rejection | Fixed fixtures for all-empty arrays, blank and oversized UTF-8 text/identifiers, exact boundary acceptance, one-over count/total-byte rejection, zero/reversed spans, overlap, descending order, beyond-duration timing and missing/zero duration; include multibyte text to distinguish bytes from codepoints |
| Independent granularities | Allow gaps and independent array partitions; reject overlaps only within their own granularity; no forced sentence/word/phoneme nesting assumption |
| Atomic invalid input | Snapshot project.json/history.json/drafts/managed media before failing commit and replacement on current and genuine legacy projects; compare complete bytes after rejection |
| Revision and references | Stale commit/replacement and missing track/item failures preserve exact existing codes and complete authoritative resources; aligner metadata has no new asset reference and changes no GC policy |
| Exact lifecycle | Native core commit/replacement and fresh headless read roundtrip; inspect new asset IDs, stable replacement item ID, one revision, undo/redo and no-rewrite second reopen |
| Atomic migration | Genuine schema37 current/undo/redo with old speech provenance; mixed supported sources; current/undo/redo alignment presence below38 including null, invalid aligned38 retained history and future versions reject before staged publication |
| Journal ownership | Existing fault injection at preparation/journal/publication phases yields complete old or committed new generation and established warning semantics; no additional persistence mechanism |
| Bridge retained artifact | Preview commit conflict retry preserves exact audio and alignment with fake synthesizer invocation count1; valid regeneration publishes only returned new provenance; unaligned current provider remains unaligned |
| Cross-language parity | Canonical manually authored alignment catalog, actual Rust/Serde and Zod shapes plus fresh headless/MCP serialization; update declared ownership and designated reviewer |
| Complete predecessor proof | Compare unmodified main17 catalog raw pins, then independently pin expanded semantic predecessors; approved38→37 projection removes only enumerated additions/markers; all historical pins and unrelated-drift negatives retained |
| Renderer preservation | Provenance-only metadata produces identical evaluated preview/export behavior with unchanged media; no new render capability, expression evaluation or network/path interpretation |

The Windows-only baseline descendant test failure is a prerequisite gate issue and must not be counted as an alignment scenario failure or hidden by Linux-only passing evidence. No tests may be disabled, deadlines weakened, or expected failures reclassified as successful acceptance.

Exact-head initial CI37668406035 passes standard23-test integration and20-test packaged smoke, preserving local OOM diagnostics separately. Ubuntu112953523416 and Windows112953523431 fail the new headless fixture because correctness jobs deliberately do not install FFprobe. The reviewed correction runs the complete47-case protocol suite hermetically everywhere and additionally with real FFprobe under the unchanged mandatory native gate. Full original job logs are retained at `/tmp/opencut-issue60-initial-job-112953523416.log` and `/tmp/opencut-issue60-initial-job-112953523431.log`. No passing check is inferred from these failures.
