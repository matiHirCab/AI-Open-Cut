# Temporal animation fixtures

The synthetic `issue48-independent-temporal-v1` recipe is in
`crates/editor-core/tests/fixtures/temporal/recipe.json`. The generator uses
production project creation and edit operations in a fresh store. It imports no
media, evaluates no expressions supplied by a user, and changes no public schema
or reviewed golden references. Existing directories are rejected.

Generate both persistent projects for desktop inspection:

```sh
cargo run -p opencut-editor-core --example temporal_fixture -- /tmp/opencut-temporal-new
```

The output prints each project store, project ID, revision, overlay track and root
item IDs. Family A contains six editable root rectangles. Family B contains an
editable root component instance and repeater; definition-local children remain
read-only in the desktop inspector. Use the printed `--project-store` and
`--project-id` arguments when launching `opencut-desktop`. The preview remains the
existing placeholder; this generator supplies inspection/editing evidence, while
the native tests verify real rendered output.

Each project is 64×64 at 10 fps and lasts 1300 ms. Family A uses six opaque 5×5
rectangles in separate lanes. Spring and Bézier keys are `(0,8)` and `(1000,20)`.
For interior progress `p=t/1000`, the independent Bézier equation is
`B(p)=3p²−2p³`, and the spring equation is
`S(p)=1−exp(−p)*(cos(sqrt(99)*p)+sin(sqrt(99)*p)/sqrt(99))`.
Position is `8+12*B(p)` or `8+12*S(p)`. At/before zero and at/after 1000 ms the
authored values 8 and 20 are held exactly. Spring at 300 ms overshoots 20; valid
Bézier controls all remain in [0,1]. The remaining lanes compare finite two and
infinite repeat triangles `(100,8),(300,20),(500,8)` with linear segments, and
finite two and infinite ping-pong `(100,8),(300,20)` with the same Bézier. One
ping-pong iteration is a complete outward/return traversal. Both finite modes
exhaust at 900 ms and hold 8.

Family B proves clocks from authored metadata rather than injecting evaluated
facts. Its root instance has start100, trim50 and rate1.5; the outer group has
stagger100, with a hidden child retaining its authored rank. The inner instance
has start20, trim10, rate0.5 and stagger40, and another hidden leaf retains rank.
The visible leaf's composition time is
`((t−100)*1.5+50−100−20)*0.5+10−40 = 0.75*t−140`.
Its explicit retained clock adds250 once, with sourceDuration1500 and visible
duration1200. Two root repeater copies delay100 root ms per copy and move16 pixels
vertically, so copy `j=0,1,2` samples `0.75*t+110−75*j`. At root713 ms their
source times are644.75,569.75,494.75. Each uses the infinite linear triangle.
The copies start at186⅔,286⅔,386⅔ root ms and all are inactive at1300 ms. Thus
the second copy is correctly absent at300 ms; inactive lanes have no centroid.

Numeric tests exercise exact endpoints, fractional neighbors, turns, seams,
finite exhaustion and half-open visibility. Native tests compare production frame
300/500/1000/1200, audiovisual range300..1300, full export, fractional range713..913,
and a materialized draft frame1000/range900..1300. The candidate changes the spring
final key from20 to24 in Family A and repeater offset100 to120 in Family B;
one independent candidate PNG and one complete encoded candidate sequence per
family distinguish it from committed state. Export retains its global frame
grid; the713 ms range's first frame samples its exact requested time. References
use independently calculated static positions: PNG controls use a neutral zero-vignette full
workspace, while encoded sequences use explicit static affine geometry and matching codec settings.
Expected positions never derive from production animation evaluation; references still traverse
the production static renderer and canonical activity path. RGB MSE remains below20,
and lossless active-lane centroids differ by less than0.4 pixels from the
independent `(x+2,y+2)` geometric center and static control. Encoded centroids
compare against their codec-matched independent sequence with the same0.4 limit. Explicit authored
activity and color mass prevent coordinated missing-lane or wrong-color output
from passing. The two families
perform39 production renders in total, asserted against a maximum40; this does
not change any CI time budget. Observation preserves project/history/draft bytes.

Run the focused checks:

```sh
cargo test -p opencut-editor-core --lib temporal_fixture_tests -- --nocapture
cargo test -p opencut-editor-core --test animation_channels temporal_fixture_controller_channel_history_and_atomic_failure -- --nocapture
OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED=1 OPENCUT_GOLDEN_REQUIRED=1 \
OPENCUT_FFMPEG_PATH=/usr/bin/ffmpeg OPENCUT_FFPROBE_PATH=/usr/bin/ffprobe \
OPENCUT_TEST_FONT_PATH=crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf \
cargo test -p opencut-editor-core --test animation_channels native_temporal_fixtures_match_independent_frame_range_draft_export -- --nocapture
```

The native test is included in the existing required `animation_channels` target
and fails closed when required dependencies are unavailable. Record actual
FFmpeg/FFprobe versions, reviewed font identity, commands and results with the
change's conformance evidence. The lifecycle test verifies edit/undo/redo/reopen,
invalid inputs, missing items, stale revisions, locked tracks and failed alias
batch rollback without changing the canonical rules.

Encoded range/export references contain independently positioned static rectangles on every exact output sample, with empty channels and no inherited/source clocks. Encoded references use explicit constant Transform2D geometry without effects/channels to preserve fractional coordinates and the actual codec profile. They use the same production intent, encoder profile and complete temporal context; complete native libx264 encoder settings are asserted equal. Lossless images retain direct analytic geometry/color/mass checks; encoded images retain MSE20 and per-color centroid0.4 against the matching encoded independent sequence. Actual lossless713ms samples additionally verify fractional geometry directly. The39 renders comprise10 PNG references,2 additional actual713ms frames,8 encoded reference sequences and19 existing actuals. Original endpoint-atlas and lossless/CRF28 comparison failures remain in local evidence.

Issue48's original nonaligned correction covered animated rectangles, shapes, media and shaped/PAM-capable text, plus static explicit Transform2D rectangles. The subsequent independent epic6 audit verified remaining accepted SolidColor and inherited Caption timing failures, and plain static activity failures across Rectangle/SolidColor/Caption/Shape/SVG/Grid/pinned shaped Text/image Media. The separate approved `close-motion-graphics-audit-gaps` correction covers those cases with one canonical source eligibility predicate and source-equivalent bounded resources; six focused native audit cases and eight compatibility/runtime tests now pass; required protected126 (including the existing39-render oracle) and full release golden now pass; remaining transport/final acceptance are pending. The39-render recipe and tolerances remain unchanged.

The corrective design preserves ordinary Caption bottom-centered text/box/escaping and affine Caption's distinct source/placement, current pinned Text requirements, media source mapping, audio gain/placement, codecs, aligned requests and export grid. Caption-owned channels remain rejected; historical internal unshaped Text does not establish valid current-project reachability. The [readiness audit](mg-m3-readiness-audit.md) records actual findings, accepted specification scope and remaining gates without a universal passing-runtime claim.

Further native audit diagnostics reproduced direct SolidColor input timestamps600/700/900ms where the canonical global grid requires700/800/900, and an inclusive800ms split seam double-compositing adjacent SolidColor or legacy Rectangle halves. Original917 public exports and nine diagnostic exports are preserved in the external audit bundle. The bounded amendment proposes checked global frame indices, undurationed synthetic color trimmed to the exact count, an integer frame-tick shift, and half-open visibility only for those direct synthetic branches. The amendment has actual independent delegated specification approval; bounded production correction and focused complete-sequence native controls now pass; mandatory verification remains pending. Existing scalar/inherited clocks, codecs, media/audio/Caption paths, golden references and tolerances are preserved. The six new requested-origin tests are also included as a module in the protected animation_channels target; a standalone focused invocation is additional proof.

The bounded reviewed-fullscene compatibility correction also passes a focused unchanged semantic/graph/PNG/audio baseline capture and complete endpoint error/byte controls. It defers raw nonaffine root SolidColor intrinsic measurement to the strict consuming preparation stage and preserves historical fullscene graphs only under the approved direct-root/interior-intent guard. Full workspace/protected126/release golden reruns have now passed, while remaining transport/final acceptance stay pending; neither reviewed references nor comparison tolerances were changed.

All three required PR-scope rules resolutions now pass (25+25+6 operations), completing renderer/native verification together with protected126, temporal39, release golden and lifecycle/cache checks. Fresh transport/canonical and final review/policy/lifecycle acceptance remain pending.
