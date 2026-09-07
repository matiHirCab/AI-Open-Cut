## Context

CI run 34067911715 passes the corrected recipe hash and fails shape semantic plan comparison. Comparing the two logged plans finds exactly three differences: 2.3431457505076194 versus 2.3431457505076203; 0.5358983848622461 versus 0.5358983848622465; and 3.819660112501051 versus 3.8196601125010474. All are derived contour coordinates. The maximum is 8 ULPs and less than 1e-12 absolute error. The existing shape fixture stores a raw Debug plan; other established golden owners remain untouched.

## Goals / Non-goals

Allow this measured platform variation without hiding structural, authored, affine, density or rendering changes. Preserve exact repeated evaluation checks on one runtime, recipe checksum enforcement, RGB references, SSIM/audio/timing tolerances and production behavior. No schema, migration, dependency, workflow or transport edits.

## Decisions

1. Add a test-only shape reference comparison helper. Compare the full plan structure and all non-coordinate text exactly. Only corresponding finite x/y values within generated contours' VectorPoint entries may differ, and only when BOTH absolute difference <= 1e-12 and ordered-f64 distance <= 8 ULPs. Preserve signed-zero distinctions and reject non-finite values, malformed structure, missing/extra entries, and unmatched coordinate locations. Do not normalize authored geometry VectorPoints or paint coordinates.
2. Parse the narrow generated Debug structure with explicit scope tracking rather than applying a global numeric regex. Report the failing line/location. Do not alter production Debug formatting or persisted models. Existing exact same-runtime plan checks remain in addition to portable stored-reference comparison.
3. Keep reference.json's RGB and semantic plan bytes intact. The checksum correction already committed remains. No baseline recapture, platform-specific baseline, or automatic update.
4. Add regressions first using all three observed CI pairs and the checked-in reference plan. Cover 8 ULP acceptance, 9 ULP rejection, absolute-cap rejection even within 8 ULPs, negative/zero cases, NaN/infinity, field-count changes, authored-point mutations, transform/density/paint/ordering mutations and real contour drift. Run the full native gate locally and confirm hosted Linux render/foundation parity before claiming CI repaired.

## Alternatives

Global decimal rounding could hide unrelated semantic changes and has arbitrary rounding boundaries. Linux-only recapture would simply reverse the failure on Windows. Changing production trigonometry or adding a math dependency expands scope. A narrowly bounded test-only reference comparison addresses the observed portability issue directly.

## Risks / Trade-offs

The Debug format can evolve: scope validation and negative tests must fail closed rather than silently treating more fields as tolerant. Eight ULPs is based on observed evidence; larger future differences must fail for review. The absolute cap prevents large-coordinate ULP spacing from allowing meaningful movement.

## Migration and rollback

No project migration or API change. Revert only the comparison helper and its tests if necessary; existing references remain usable. Obtain artifact approval, implement and validate, verify, sync/archive, run Moon, push to PR #114 and inspect CI. The active artifacts stay local until archival.

## Open questions

No technical decisions remain open; the user approved the concrete artifacts on 2026-09-06.
