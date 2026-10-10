"""Local synthetic PCM/alignment fixture; no speech inference or provider changes."""
import json
import math
import pathlib
import struct
import sys
import wave

RECIPE = json.loads((pathlib.Path(__file__).resolve().parents[4] / "contracts/complete-reference-scene-v1.json").read_text())


def source(path, frequency, amplitude, duration, voice=False):
    with wave.open(str(path), "wb") as output:
        output.setnchannels(2)
        output.setsampwidth(2)
        output.setframerate(48000)
        pcm = bytearray()
        for frame in range(duration * 48):
            ms = frame // 48
            active = not voice or any(cue["startMs"] <= ms < cue["endMs"] for cue in RECIPE["alignment"]["sentences"])
            value = round(amplitude * math.sin(math.tau * frequency * frame / 48000) * 32767) if active else 0
            pcm.extend(struct.pack("<hh", value, value))
        output.writeframes(pcm)


if len(sys.argv) == 3 and sys.argv[1] == "--sources":
    root = pathlib.Path(sys.argv[2])
    root.mkdir(parents=True, exist_ok=True)
    for name, frequency, amplitude, duration in [("music", 211, .08, 6000), ("event0", 659, .1, 300), ("event1", 877, .1, 300)]:
        source(root / (name + ".wav"), frequency, amplitude, duration)
    sys.exit(0)

VOICE = {"id": "fixture", "label": "Synthetic reference", "providerId": "reference-synthetic", "modelId": "oscillator", "language": "en-US", "locale": "en-US", "accent": "Synthetic", "available": True, "isDefault": True, "previewSupported": False}
for line in sys.stdin:
    request = json.loads(line)
    operation = request.get("operation")
    if operation == "status":
        result = {"ready": True, "version": "1-test", "providerId": "reference-synthetic", "modelId": "oscillator", "modelVersion": "1", "models": [{"id": "oscillator", "version": "1", "sampleRateHz": 48000}], "device": "cpu", "devices": ["cpu"], "modelCached": True, "modelLoaded": False, "sampleRateHz": 48000, "languages": ["en-US"], "voices": ["fixture"], "defaultLanguage": "en-US", "defaultVoiceId": "fixture", "defaultSpeed": 1.0, "limits": {"maxTextCharacters": 5000, "minSpeed": .5, "maxSpeed": 2}, "resources": {"execution": "local", "minimumLogicalCpus": 2, "minimumRamBytes": 2147483648, "recommendedLogicalCpus": 4, "recommendedRamBytes": 4294967296}, "timestampSupport": {"sentence": False, "word": False, "phoneme": False}}
    elif operation == "list_voices":
        result = [VOICE]
    elif operation == "generate":
        source(request["outputPath"], 437, .1, 6000, True)
        result = {"durationMs": 6000, "providerId": "reference-synthetic", "modelId": "oscillator", "modelVersion": "1", "sampleRateHz": 48000, "language": request["language"], "voiceId": request["voice"]}
    else:
        raise ValueError("unsupported synthetic fixture operation")
    print(json.dumps({"id": request["id"], "type": "result", "result": result}), flush=True)
