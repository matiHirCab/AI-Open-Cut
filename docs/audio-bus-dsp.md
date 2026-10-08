# Audio bus DSP

Schema42 adds an optional nonnull `dsp` block to the four built-in audio buses.
`audio_bus_set_dsp` accepts `projectId`, `expectedRevision`, `busId`, and a closed
`dsp` record. Batch and prepared draft operations use the same typed operation.
Omission means identity. An explicitly authored identity block remains inspectable.
Migration preserves old routing, sound definitions, placed-event snapshots,
undo/redo and source-matched drafts; pre42 documents cannot contain DSP fields.

The canonical bounds and examples are in `contracts/audio-bus-dsp-v1.json`.
All parameters must be finite. Gain is −120 to24dB, pan is −1 to1, and each bus
allows up to eight ordered peaking EQ bands (20–20000Hz, Q0.1–10, gain ±24dB).
The nullable compressor accepts threshold −60 to0dB, ratio1–20, attack0.01–2000ms,
release0.01–9000ms and makeup0–24dB. It uses downward peak detection, maximum
channel linking, a hard knee and a fully wet signal. No raw FFmpeg expressions
are accepted.

Item gain, automation, fades, mute, instance clocks and legacy role ducking run
before bus summing. A placed event's captured bus takes precedence over explicit
track routing and role fallback. Each reachable bus sums its direct inputs and
already processed child buses, then applies gain, ordered EQ, compression and
stereo balance before its output bus. Master runs once. Active DSP uses48kHz
planar float stereo with standard FFmpeg channel conversion; negative pan
attenuates the right channel by1+pan and positive pan attenuates the left by1−pan.
Center leaves both channels unchanged. Neutral or unreachable DSP retains the
legacy scene, plan, graph and audio output exactly. A zero item gain stays silent
even with automation and cannot activate DSP. An audible upstream route through
that bus can activate DSP; all selected streams remain connected.

DSP editing remains available independently of render readiness. Status advertises
`audio_bus_dsp_v1` only when the base renderer and all five required DSP filters
are available. A render using active DSP checks those dependencies after canonical
model/resource admission and before destination inspection or artifact creation.
Missing dependencies return `DEPENDENCY_UNAVAILABLE` without partial artifacts.
