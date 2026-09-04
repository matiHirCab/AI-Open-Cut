## ADDED Requirements

### Requirement: Oriented media source geometry
Transform2D media source bounds MUST match the local backend's automatically oriented raster. Core SHALL finalize affine facts from typed dimensions obtained through a private bounded read-only metadata probe on canonical managed paths, at most once per transformed asset per preparation. FFmpeg SHALL retain responsibility for applying display rotation and flips exactly once. No public or persisted shape SHALL change.

#### Scenario: Resolve display orientation
- **WHEN** media has absent, quarter-turn, mirrored, or non-quarter-turn display metadata
- **THEN** typed source dimensions match the backend's source extent, including swapped axes for quarter turns, and identity Transform2D preserves displayed content

#### Scenario: Fail an unusable probe
- **WHEN** metadata probing fails or returns invalid dimensions or orientation
- **THEN** rendering fails with an existing typed error before workspace creation or raster execution, without an unrotated fallback

#### Scenario: Reuse source measurements
- **WHEN** multiple transformed items share an asset
- **THEN** preparation probes it once and reuses the typed dimensions without mutating project or history
