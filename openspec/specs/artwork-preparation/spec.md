# Artwork Preparation Specification

## Purpose

Specify bounded, deterministic PNG preparation, explicit review packages and source-preserving managed artwork commits.

## Requirements

### Requirement: Bounded deterministic artwork transformation
Core SHALL preserve ordinary legacy first-open migration; its preview SHALL introduce no artwork mutation and preserve already-normalized current project bytes. Supported preparation SHALL require matching probed source dimensions; missing legacy facts SHALL retain ASSET_INTEGRITY_FAILED with explicit reimport guidance without declaring damaged bytes. Core SHALL prepare only supported managed static PNG Image sources within 4MiB and 512 by512 decoded bounds, normalize PNG to RGBA8 without scaling, and validate integer cleanup/outline controls before allocation/publication. Optional border-connected cleanup SHALL use four-neighbor reachability and inclusive max-channel RGB tolerance 0..255; disabled cleanup SHALL preserve original alpha. Outline radius0..16 SHALL use square alpha dilation, explicit color/alpha and radius padding with disclosed content offset, preserving transparent holes/fine detail according to the selected operation. Color math and limits SHALL follow the approved design. Unsupported, corrupt, oversized, animated or invalid requests MUST fail deterministically without project changes.

#### Scenario: Inspect exact alpha and edges
- **WHEN** synthetic transparent, partial-alpha, opaque white subject, interior-hole, fine-line and border-touching PNG fixtures are prepared with disabled/enabled cleanup, endpoint tolerances and outline radii
- **THEN** exact independent RGBA/mask oracles establish retained source geometry, matching reachability, padded outline and source-over arithmetic

#### Scenario: Reject unsupported or bounded input
- **WHEN** source format, animation, bytes, dimensions or parameters exceed supported limits or PNG decoding fails
- **THEN** existing typed errors reject before managed asset/history publication and source bytes remain unchanged

### Requirement: Explicit inspect apply and discard
The core SHALL remain platform-neutral; the initial CLI file-package backend SHALL require Linux descriptor-anchored directory operations and fail DEPENDENCY_UNAVAILABLE on unsupported platforms before dispatch or mutation. A thin local CLI SHALL expose core-generated result, cleaned alpha mask and opaque light/dark review PNGs with exact source/project/revision/parameter/result identity in a bounded versioned manifest. Preview SHALL require a new explicit allowed output directory, forbid overwrite/links and emit at most8MiB total PNG data. Applying SHALL require explicit approval, intact reviewed output and the captured manifest; discarding SHALL remove only owned review files and never change project state or foreign files. Existing provider, path, privacy and overwrite permissions MUST remain authoritative.

#### Scenario: Review and reject ambiguous cleanup
- **WHEN** a caller prepares a package, inspects mask/light/dark files and discards it
- **THEN** all project/history/source bytes remain unchanged and only the owned review package is removed

#### Scenario: Reject unavailable backend and cancellation while reading
- **WHEN** the CLI runs on an unsupported platform, is cancelled while waiting for input, or encounters a nonregular review file
- **THEN** it fails promptly without dispatch or mutation; Linux file handling rejects parent swaps/links and FIFO/device inputs without blocking

#### Scenario: Reject altered or unsafe review files
- **WHEN** reviewed files/manifest are modified, output already exists, paths escape policy, links occur or foreign files are present during discard
- **THEN** the operation fails without project mutation, unauthorized overwrite or foreign deletion

### Requirement: Atomic source-preserving derivative commit
Core SHALL revalidate captured project revision, source ID/hash, parameters, result hash/dimensions and managed source integrity under its existing lock, then atomically publish one new independent Image asset with immutable derivation metadata and one normal history revision. It MUST NOT overwrite source bytes, mutate timeline placement, rerun a provider, silently rebase stale intent or treat manifest metadata as a revived process capability. Repeat preparation SHALL be deterministic. Cancellation/failure before the durable commit point MUST NOT publish a partial asset; uncertainty after commit MUST require inspection before retry.

#### Scenario: Apply reopen undo and redo
- **WHEN** a reviewed synthetic derivative is explicitly committed, reopened, undone and redone
- **THEN** its exact pixels, parameters, source hash and padded geometry survive, and source/timeline remain intact

#### Scenario: Conflict duplicate cancellation and injected failure
- **WHEN** captured revision is stale, commit repeats, source/result identity differs, preparation is cancelled or persistence faults occur
- **THEN** typed failure preserves atomic boundaries, no duplicate or partial asset appears, and successful authoritative commits remain inspectable rather than automatically replayed
