import json
import sys

for line in sys.stdin:
    request = json.loads(line)
    if request.get("operation") == "status":
        result = {
            "ready": True,
            "providerId": "fake-transcriber",
            "modelId": "small",
            "modelVersion": "test",
            "device": "cpu",
            "computeType": "int8",
            "modelCached": True,
            "modelLoaded": True,
            "maxDurationMs": 60000,
            "version": "transcription-provider-v1",
        }
    elif request.get("operation") == "transcribe":
        result = {
            "language": request.get("language") or "en",
            "durationMs": 1000,
            "segments": [{
                "text": "Packaged caption",
                "startMs": 0,
                "endMs": 1000,
                "words": [{"word": "Packaged", "startMs": 0, "endMs": 500}],
            }],
        }
    else:
        print(json.dumps({"id": request.get("id"), "ok": False, "error": {"code": "TRANSCRIPTION_PROVIDER_FAILED", "message": "unsupported"}}), flush=True)
        continue
    print(json.dumps({"id": request.get("id"), "ok": True, "result": result}), flush=True)
