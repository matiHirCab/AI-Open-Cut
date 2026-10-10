## MODIFIED Requirements

### Requirement: Genuine reference audiovisual evidence
Configured native reference checks MUST use actual FFmpeg/FFprobe and a fixed font, demonstrate visible frame output and audio-bearing540p/720p range review with repeated revision-keyed cache reuse where the existing core backend identity is eligible and explicit deterministic uncached fallback where identity is unavailable, final export and fresh reopened/history states. Equivalent requested settings MUST share canonical evaluated semantics and retain SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame timing tolerances. Literal structural/timing/output witnesses and negative controls MUST supplement cross-intent comparisons; the latter MUST NOT be presented as an independent newly generated full-frame golden. Analysis MUST report actual LUFS/true peak through existing core behavior. Required missing tools/font/media checks MUST block acceptance, while unexecuted supported-platform checks remain explicit limits pending their own issues.

#### Scenario: F4 Compare native review and export
- **WHEN** the full scene is rendered through frame/range/export with cache reuse where eligible or explicit existing identity fallback at matched settings, then reopened or restored through history
- **THEN** stream and sampled decoded output evidence meets retained tolerances, cached and uncached published artifacts keep immutable identity and independent witness expectations remain true

#### Scenario: F5 Fail closed on missing or drifting evidence
- **WHEN** native dependencies are missing or an independent witness/timing/output negative control is altered
- **THEN** the required check fails and no optional skip or cross-intent agreement substitutes for genuine complete-scene evidence
