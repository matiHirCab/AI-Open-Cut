## ADDED Requirements

### Requirement: Frame-derived static image geometry
For transformed static images, core MUST derive oriented source dimensions from one usable decoded image frame inspected by the configured FFprobe through the private process adapter. Inspection MUST select the first video stream and bound packet inspection to one packet and metadata output to 64 KiB. Frame dimensions and the frame display matrix MUST determine source extent using the existing backend-compatible rotation logic; absent frame orientation means identity. Core MUST NOT substitute encoded stream geometry for unusable frame metadata. Video probing and all public and persisted shapes SHALL remain unchanged.

#### Scenario: Preserve EXIF image extent
- **WHEN** an asymmetric static JPEG with absent orientation or EXIF orientation 1 through 8 uses identity Transform2D
- **THEN** its displayed extent and content match legacy rendering, including all 800 colored pixels of the 20x40 displayed orientation-6 fixture

#### Scenario: Reject unusable image inspection
- **WHEN** image inspection fails, exceeds the metadata cap, returns no unique usable frame, or supplies invalid dimensions or display-matrix metadata
- **THEN** rendering returns UNSUPPORTED_MEDIA before destination inspection or writes without an encoded-geometry fallback; inability to start FFprobe returns DEPENDENCY_UNAVAILABLE

#### Scenario: Reuse inspected images
- **WHEN** multiple transformed items share a static image
- **THEN** preparation probes that asset once on its canonical managed path, materialization reuses its typed dimensions without another probe, and project, revision, and history remain unchanged
