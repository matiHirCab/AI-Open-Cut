# Master loudness normalization

`audio_master_set_normalization` authors optional root `masterNormalization` in
project schema 44. Its required controls are `enabled`, `targetIntegratedLufs`
(-70 to -5), `targetLoudnessRangeLu` (1 to 50), and `targetTruePeakDbtp` (-9 to 0).
Disabled controls still validate. Omitted or disabled controls retain the
previous rendering path. Editing works without FFmpeg; active rendering requires
the reported `audio_master_normalization_v1` capability.

The renderer measures and processes the complete root mix after source/item/
component/event timing, routing, bus DSP, ducking and master balance. It applies
one measured normalizer before selected output cropping and codec encoding.
Preview, export, draft rendering and audio analysis consume the same prepared
root controls. Active roots must last 1 to 600,000 milliseconds; preparation
streams 48 kHz stereo finite float PCM with bounded buffers and at most three
private files. Measured coefficients are never persisted.

The integrated target is verified on delivered complete-root precodec PCM with
EBU R128 integrated metadata at 0.001 LU resolution, within 0.1 LU of the target.
A separate fine true-peak measurement admits at most 0.01 dB report rounding
above the configured ceiling. The legacy audio-analysis `integratedLufs` uses
the fixed loudnorm input meter; that value can differ from the independent EBU
measurement and does not drive normalization corrections. LRA is processing
guidance rather than a promise of exact output LRA.

Exact silence remains unchanged. Short or very quiet audio with unmeasurable
integrated loudness uses limiting-only attenuation, never an invented integrated
target result. Infeasible measurable targets or failed processing/verification
return `FFMPEG_FAILED` and publish no final artifact. Selected crops can have a
different integrated loudness; AAC and other codec quantization can also change
decoded peaks. The target guarantee applies to the complete root before encoding.

Source schemas 1 through 43 and retained undo/redo history adopt schema 44 under
the existing atomic project transaction. Old documents do not acquire invented
controls. Premature, null, malformed and unknown future settings are rejected.
Standalone edits, batches and drafts retain optimistic revisions, rollback,
undo/redo and reopening semantics. This root setter creates no ID or result alias.
