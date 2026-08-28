"""OpenCut faster-whisper JSON-lines worker (Python 3.11, CPU int8)."""
from __future__ import annotations

import argparse
import json
import os
import sys
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
from typing import Any

PROVIDER_ID = "faster-whisper"
CONTRACT_VERSION = "transcription-provider-v1"
DEFAULT_MODEL = "small"
MAX_DURATION_MS = 14_400_000
READY_MARKER = ".opencut-ready.json"


class Worker:
    def __init__(self, model_id: str, model_dir: Path) -> None:
        self.model_id = model_id
        self.model_dir = model_dir
        self._model: Any | None = None

    def _load(self) -> Any:
        if self._model is None:
            from faster_whisper import WhisperModel
            self._model = WhisperModel(
                self.model_id,
                device="cpu",
                compute_type="int8",
                download_root=str(self.model_dir),
                local_files_only=True,
            )
        return self._model

    def status(self) -> dict[str, Any]:
        cached = (self.model_dir / READY_MARKER).is_file()
        try:
            model_version = version("faster-whisper")
        except PackageNotFoundError:
            model_version = None
        return {
            "ready": cached,
            "providerId": PROVIDER_ID,
            "modelId": self.model_id,
            "modelVersion": model_version,
            "device": "cpu",
            "computeType": "int8",
            "modelCached": cached,
            "modelLoaded": self._model is not None,
            "maxDurationMs": MAX_DURATION_MS,
            "version": CONTRACT_VERSION,
        }

    def transcribe(self, request: dict[str, Any]) -> dict[str, Any]:
        path = request.get("path")
        if not isinstance(path, str) or not Path(path).is_file():
            raise ValueError("input media is unavailable")
        language = request.get("language")
        if language is not None and not isinstance(language, str):
            raise ValueError("language must be a string")
        duration_ms = request.get("durationMs")
        if not isinstance(duration_ms, int) or duration_ms <= 0 or duration_ms > MAX_DURATION_MS:
            raise ValueError("input duration is invalid")
        segments, info = self._load().transcribe(
            path,
            language=language,
            word_timestamps=True,
            vad_filter=True,
        )
        output = []
        duration_ms = 0
        for segment in segments:
            start_ms = max(0, round(segment.start * 1000))
            end_ms = max(start_ms + 1, round(segment.end * 1000))
            duration_ms = max(duration_ms, end_ms)
            words = [
                {
                    "word": word.word.strip() or word.word,
                    "startMs": max(0, round(word.start * 1000)),
                    "endMs": max(1, round(word.end * 1000)),
                    **({"confidence": word.probability} if word.probability is not None else {}),
                }
                for word in (segment.words or [])
            ]
            output.append({"text": segment.text.strip(), "startMs": start_ms, "endMs": end_ms, "words": words})
        return {"language": info.language, "durationMs": max(1, duration_ms), "segments": output}


def prepare(model_id: str, model_dir: Path) -> None:
    from faster_whisper import WhisperModel
    WhisperModel(model_id, device="cpu", compute_type="int8", download_root=str(model_dir))
    (model_dir / READY_MARKER).write_text(json.dumps({"modelId": model_id, "version": CONTRACT_VERSION}), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--prepare", action="store_true")
    parser.add_argument("--model", default=os.environ.get("OPENCUT_TRANSCRIPTION_MODEL", DEFAULT_MODEL))
    parser.add_argument("--model-dir", default=os.environ.get("OPENCUT_TRANSCRIPTION_MODEL_DIR", "local-data/transcription/model"))
    args = parser.parse_args()
    model_dir = Path(args.model_dir).resolve()
    model_dir.mkdir(parents=True, exist_ok=True)
    if args.prepare:
        prepare(args.model, model_dir)
        return 0
    worker = Worker(args.model, model_dir)
    for line in sys.stdin:
        request: Any = None
        try:
            request = json.loads(line)
            operation = request.get("operation")
            result = worker.status() if operation == "status" else worker.transcribe(request) if operation == "transcribe" else (_ for _ in ()).throw(ValueError("unsupported operation"))
            response = {"id": request.get("id"), "ok": True, "result": result}
        except Exception:  # Provider details and paths must not cross the bridge.
            response = {"id": request.get("id") if isinstance(request, dict) else None, "ok": False, "error": {"code": "TRANSCRIPTION_PROVIDER_FAILED", "message": "Transcription provider failed"}}
        print(json.dumps(response, ensure_ascii=False), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
