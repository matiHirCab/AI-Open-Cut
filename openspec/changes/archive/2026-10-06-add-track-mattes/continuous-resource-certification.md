# Exact continuous request certification and pinned provider admission

This is an implementation-mechanism clarification of the approved Preflight matte
work and live memory / Shared evaluated track matte semantics requirements. It
changes no authoring scope, wire fields, schema34, pin, numerical tolerance,
request/work/memory/node limit, or complete default/no-matte behavior. Exact
limits and weighted shutter semantics remain authoritative. Implementation waits
for independent and parent approval of this manifested clarification.

## Continuous clocks and simultaneous exact materializations

MotionBlur::sample_times already computes an exact root-independent INTEGER delta
for each sample, then saturating addition and clamp to[0,duration-1]. Reuse that
owner to derive deltas using an interior anchor with a timeline large enough to
avoid clamps (validated FPS>=1 bounds all deltas within500ms); do not duplicate
its binary64 arithmetic or round inherited Fractional/Split clocks early.
Represent composed root clocks as F(root)=clamp(root+offset,low,high), using i128
for checked offset arithmetic. Composing delta d adds d to offset and clamps
low+d/high+d to[0,duration-1]. Normalize bounds to F(0),F(duration-1); constant
functions use a canonical zero offset and equal bounds. Unique provider requests
key by typed owning group/occurrence and EXACT root-clock function, and copy
requests by exact owning copy/function. Preserve ordered duplicate sample weights
separately from unique live provider/copy reuse.

Identical normalized functions may merge. Distinct functions may still collide
at some integer roots, so function-class counts are only an early upper bound.
When upper bounds fit every existing limit they may certify safety. An excessive
upper bound MUST NOT be reported as genuine4096/materialization or work excess
without resolving collisions: perform bounded exact integer-domain partitioning
at plateau transitions and linear/constant equality points, then evaluate exact
group/root and copy/root keys on representative integer regions. It is sufficient
to use the full conservative geometry/effect/mask envelopes with the exact maximum
simultaneous visit counts; animation keyframes alone never partition this proof.
Charge expansion, normalization and unresolved partition analysis to the SAME
existing65536 left-first candidate-analysis owner, preserving its canonical
analysis-exhaustion failure when proof cannot complete. Admit descriptor actual
capacities/temporary overlaps before expansion under the SAME1GiB policy; no
exponential provenance tree or new independent node counter is allowed.

Use these per-layer actual uncached visit bounds in the existing continuous
effect/mask/ordinary-source work owner. The existing complete authored/inherited
geometric/mask envelopes remain shared across hidden/unused retained contexts,
cover all transitive clocks and retain the same nodes owner. Operate on already
bounded immutable occurrence/projection facts; an empty role table in a
materialize=false intermediate is not conformance evidence, nor permission to
force extra unbounded source clones. Final retained safety must see the complete
logical dependencies. The actual per-frame schedule/runtime remains the exact
materialization/work/live-memory enforcement owner.

Required controls include duration1/depth32 with enabled maximum16 samples
(32 unique provider groups, duplicate averaging weights retained), tiny/subnormal
angles, left/right clamps, different provenance paths with identical actual keys,
exact4096/4097 uncached requests, and a noncolliding transitive masked/effect case
exceeding shared work. Known valid collisions fitting actual limits stay accepted.
Existing safe-endpoint/unsafe-interior and shared near-exhausted-node controls
remain mandatory. No blanket provenance-product rejection substitutes for proof.

## Pinned provider media integrity before cache/decoder/publication

Actual native evidence shows an imported, hash-pinned provider-only MKV still
renders after a payload-byte mutation preserving its format header and size.
An active matte scene must enforce the already stored contentHash/sizeBytes before
source-cache/memo acceptance or raster decoding/publication. Preserve canonical
path/format/geometry validation and existing error precedence, and fail hash/size
mismatch with existing nonretryable ASSET_INTEGRITY_FAILED.

Main owns a narrowly optional active-matte integrity sidecar in existing immutable
SceneResourceBindings carrying canonical asset ID, managed relative path, optional
expected SHA256 and size from existing Asset metadata. The complete no-matte
default uses an empty sidecar and adds no new hash allocations/behavior. Include
participating pinned media and referenced hidden/inactive providers; retain model
validation for unused definitions. Reuse canonical eligibility/path records, never
add parallel DAG resolution. Deduplicate deterministic asset identities and charge
all sidecar outer/nested/String/map capacities before cloning/admission. Legacy
unpinned records keep existing canonical resource checks, never invent a stored
hash or silently pin changed bytes as expected.

Render-artifact owns a private ArtifactIo bounded streaming fingerprint port,
implemented by FileSystemArtifactIo with a fixed64KiB read buffer (and size),
using the existing SHA256 library. Injected test ports fail closed or explicitly
provide their owned fingerprint; production cannot fall back to a whole-file Vec.
Validate resolved paths and pinned expected metadata before readiness/cache/decoder
and RenderWorkspace creation, with existing format/geometry ordering retained.
Charge the64KiB transient and simultaneously live sidecar/facts/cache capacities
under existing1GiB admission. No renderer→assets edge, direct filesystem bypass,
public API/schema/hash marker, or default/no-matte algorithm change is authorized.

The native warm-cache witness must assert pinned metadata, valid Matroska signature
and identical64-byte header/size, change only a payload byte, then require the
exact nonretryable integrity error and unchanged complete recursive bytes AFTER
the deliberate corruption and BEFORE the rejected render. Preserve all four
original native intents/geometry/clocks/audio/tolerances and fresh cache controls.
