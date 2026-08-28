import json
import sys
import time
import wave


VOICES = [
    {"accent": "American English", "available": True, "id": "test_voice", "isDefault": True, "label": "Test Voice", "language": "en-US", "locale": "en-US", "modelId": "fake/model", "previewSupported": True, "providerId": "fake-speech"},
    {"accent": "British English", "available": True, "id": "test_voice_gb", "isDefault": True, "label": "Test Voice GB", "language": "en-GB", "locale": "en-GB", "modelId": "fake/model", "previewSupported": True, "providerId": "fake-speech"},
]


for line in sys.stdin:
    request = json.loads(line)
    request_id = request.get("id")
    if request.get("operation") == "status":
        result = {
            "ready": True,
            "version": "1.0-test",
            "providerId": "fake-speech",
            "modelId": "fake/model",
            "modelVersion": "1",
            "models": [
                {"id": "fake/model", "version": "1", "sampleRateHz": 24000}
            ],
            "device": "cpu",
            "devices": ["cpu"],
            "modelCached": True,
            "modelLoaded": False,
            "sampleRateHz": 24000,
            "languages": ["en-US", "en-GB"],
            "voices": [voice["id"] for voice in VOICES],
            "defaultLanguage": "en-US",
            "defaultVoiceId": "test_voice",
            "defaultSpeed": 1.0,
            "limits": {
                "maxTextCharacters": 5000,
                "minSpeed": 0.5,
                "maxSpeed": 2.0,
            },
            "resources": {
                "execution": "local",
                "minimumLogicalCpus": 2,
                "minimumRamBytes": 2147483648,
                "recommendedLogicalCpus": 4,
                "recommendedRamBytes": 4294967296,
            },
        }
    elif request.get("operation") == "list_voices":
        result = VOICES
    elif request.get("operation") == "generate":
        if request["text"] == "hang":
            time.sleep(10)
        elif request["text"] == "delay":
            time.sleep(0.25)
        elif request["text"] == "malformed":
            print("{not-json", flush=True)
            continue
        elif request["text"] == "exit":
            sys.exit(17)
        elif request["text"] == "fail":
            print(
                json.dumps(
                    {
                        "id": request_id,
                        "type": "error",
                        "error": {
                            "code": "TTS_SYNTHESIS_FAILED",
                            "message": "requested fake failure",
                            "retryable": True,
                        },
                    }
                ),
                flush=True,
            )
            continue
        segment_count = len(request.get("segments", [request["text"]]))
        pause_frames = max(0, segment_count - 1) * round(24000 * request.get("sentencePauseMs", 0) / 1000)
        audio_frames = 2400 * segment_count + pause_frames
        with wave.open(request["outputPath"], "wb") as output:
            output.setnchannels(1)
            output.setsampwidth(2)
            output.setframerate(24000)
            output.writeframes(b"\0\0" * audio_frames)
        result = {
            "durationMs": round(audio_frames * 1000 / 24000),
            "providerId": "fake-speech",
            "modelId": "fake/model",
            "modelVersion": "1",
            "sampleRateHz": 24000,
            "language": request["language"],
            "voiceId": request["voice"],
        }
    else:
        print(
            json.dumps(
                {
                    "id": request_id,
                    "type": "error",
                    "error": {
                        "code": "INVALID_ARGUMENT",
                        "message": "unsupported",
                        "retryable": False,
                    },
                }
            ),
            flush=True,
        )
        continue
    response = json.dumps({"id": request_id, "type": "result", "result": result})
    if request.get("text") == "partial":
        midpoint = len(response) // 2
        sys.stdout.write(response[:midpoint])
        sys.stdout.flush()
        time.sleep(0.02)
        sys.stdout.write(response[midpoint:] + "\n")
        sys.stdout.flush()
    else:
        print(response, flush=True)
