# Exact byte-to-linear transfer: fixed reference evaluation

Issue 203 uses one private process-wide runtime-generated `LazyLock<[f64;256]>`. Payload 2048 bytes, total singleton including standard header 2056 bytes on this reference build, below the 4096-byte approved bound and within the existing 1048576-byte fixed-overhead reserve. Existing owner certificates and limits are unchanged. No raster, source, project or cache ownership is retained by this table.

All 65,536 channel×alpha combinations independently cover each asymmetric RGB lane and match original expression bits, including f64 premultiplication followed by the final f32 cast. Alpha output, zero/empty/malformed/PAM/excessive-dimension/overflow ordering, typed InvalidArgument and nonretryability are preserved. Eight workers prove shared immutable identity and exhaustive concurrent reuse; this is concurrent reuse, not a simultaneous-first-initialization stress test.

## Fixed paired method and result

One standalone BEFORE (420.78 s, exit 0) and one standalone AFTER (354.04 s, exit 0), each 14 cells/140 requests, same frozen driver, fixture recipe hashes, named reference host/actual quotas, settings, fonts, tools and build toolchain. Fresh means a new Renderer; OS caches were not flushed. AFTER first singleton initialization is included in the first fresh render of its separate process. Later fresh Renderers reuse the process-wide singleton. No concurrent repository suite ran with either capture. Full reports are retained byte-for-byte in before.json and after.json; comparison.json attributes their hashes.

The automated comparator passed output/artifact hashes, request/frame/scene counts, actual cache work and owner charge maxima for fresh, warmup, repeated and raster-warm/encoded-cold controls. Generated project identities need not match across independently authored fixtures; each capture verifies unchanged authoritative/preexisting state.

All 14 AFTER fresh maxima are below 5000 ms (frame)/8000 ms (range). Predeclared row4/frame facade maximum gain: **11.27%** (required ≥5%); native_visual_preparation median gain: **30.74%** (required ≥10%). Preparation is directly observed inclusive timing, not inferred by summing/subtracting stages or an isolated powf measurement. This finite fixed-host observation makes no portable, statistical or tail-latency claim. Grid1/frame was slower in the fixed AFTER capture (3718.160ms versus2004.880ms), while remaining below5000ms. The accepted result does not claim improvement in every cell.

| Workload | Intent | BEFORE fresh maximum ms | AFTER fresh maximum ms |
|---|---|---:|---:|
| baseline | frame | 3407.509 | 3074.164 |
| baseline | range | 4365.922 | 3539.969 |
| hero | frame | 2544.224 | 2101.202 |
| hero | range | 3882.235 | 2990.574 |
| motif_gallery | frame | 1775.445 | 1883.827 |
| motif_gallery | range | 2371.747 | 2295.438 |
| row1 | frame | 2531.900 | 2031.350 |
| row1 | range | 2582.874 | 2190.693 |
| row4 | frame | 3087.754 | 2739.854 |
| row4 | range | 6637.476 | 3870.891 |
| grid1 | frame | 2004.880 | 3718.160 |
| grid1 | range | 3798.007 | 2435.105 |
| grid4 | frame | 3616.206 | 2616.909 |
| grid4 | range | 5385.673 | 3937.525 |

Row4 preparation medians: BEFORE 1016.182302ms, AFTER 703.771614ms. Resident-process-tree samples remain observational, separate from conservative owner certificates.

## Provenance and retained failures

BEFORE report SHA256 `4db8bb38ce063a5d7614e97e1ca3b6371e8ca1d396c71c86f34a896d7d55a8e6`; AFTER `05526b3ac626d1578d81a87d32806ec9930a7a43b534e3550be2ccc203badcd5`. Driver unchanged `e8433ea5fa7588aa3fd040d725f39223592b799c87807029a6d064d99feb9ab9`. Compiled decoder identities BEFORE `476e59ecc812c03aa5614409970982dc35f9d6520a25b291e417e59e8d2bb81e`, AFTER `2bd511e607669ed109a0e5dca4ec5c6f53e802142d976151c69c34bd0cdbd9bf`. Test binary identities BEFORE `3694012909b68ccd07579d29563d3bd9be08013e981ac81c1d22f883e1435849`, AFTER `302403adab50f578fe9fe3b776c18bde46a4bcc91d628b52e41b22ae5442ed0a`. Captures execute this 203 worktree at cumulative predecessor HEAD `de024ee5a04a8ac398a02108458562aeca0e0d89` with explicitly recorded tracked diff hashes/dirty source; these are implementation measurements before final delivery commit. Build-toolchain.txt retains Rust 1.97/Cargo/toolchain-file provenance; report rustc is a runtime toolchain observation.

The original issue 187 **5414.023545ms** row4/frame counterexample remains unchanged in ../bounded-dense-scene-strategy/reference-observation.json, SHA256 `01194abacd1fdb644d9a6dda0d0922e6f751b1b9d2b16a6072dc317b9326bd7c`. Both new captures happened to meet the absolute targets before the lookup; the separate required relative gains also passed. No sample was replaced or used to erase the original failure.

Caught missing-glyph negative-control panic messages appear in capture logs; the standalone commands completed 1 passed/0 failed/0 ignored. Controls reject invalid/missing or matching-wrong host/source evidence, matrix/count/output/owner/cache/stage drift, absolute target failure, insufficient facade improvement below the absolute limit and insufficient preparation improvement independently.

## Delivery checks

Focused release 4 passed/0 failed; paired comparison 1 passed/0 failed; independent specification, corrected controls and production source review ACCEPT. Full repository checks, actual configured default-platform/desktop/native golden and complete motion release passed; verification.json retains command/log/report attribution. Final independent conformance ACCEPT; accepted delta synced and issue203 archived. Postarchive protected Moon and strict OpenSpec gates passed (73/0). Exact-head CI remains pending after the delivery commit.

After the fixed captures, strict Clippy rejected a redundant conjunction in the storage test. It was split into two assertions with both bounds preserved. The frozen measurement driver and all production code remain unchanged. post-capture-test-correction.json records final decoder identity and byte-exact reconstruction of the captured source by reversing only this test assertion edit. Later compatibility checks validate the corrected final source; they do not replace either performance dataset.
