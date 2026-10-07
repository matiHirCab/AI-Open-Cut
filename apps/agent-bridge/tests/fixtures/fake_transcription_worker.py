import json
import sys
import time

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
            "knownTextAlignment": {"maxDurationMs": 30000, "maxTextBytes": 4096, "phoneme": False,
                                   "sentence": False, "supported": True, "word": True},
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
    elif request.get("operation") == "align":
        if request.get("knownText") == "__slow_alignment__":
            time.sleep(0.2)
        if request.get("knownText") == "__timeout_alignment__":
            time.sleep(2)
        duration = request["durationMs"]
        text = request["knownText"]
        word = {"text": text, "startMs": 0, "endMs": duration}
        result = {"language": request.get("language") or "en", "durationMs": duration,
                  "segments": [{"text": text, "startMs": 0, "endMs": duration,
                                "words": [{"word": text, "startMs": 0, "endMs": duration}]}],
                  "alignment": {"sentences": [], "words": [word], "phonemes": [], "quality": "forced",
                                "providerId": "fake-transcriber", "modelId": "small", "modelVersion": "test"}}
        if text == "__malformed_alignment__":
            result["alignment"] = None
    else:
        print(json.dumps({"id": request.get("id"), "ok": False, "error": {"code": "TRANSCRIPTION_PROVIDER_FAILED", "message": "unsupported"}}), flush=True)
        continue
    print(json.dumps({"id": request.get("id"), "ok": True, "result": result}), flush=True)
