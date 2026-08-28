from pathlib import Path
from types import SimpleNamespace
import json
import subprocess
import sys
from worker import Worker


def test_status_is_model_free(tmp_path: Path) -> None:
    status = Worker("small", tmp_path).status()
    assert status["providerId"] == "faster-whisper"
    assert status["computeType"] == "int8"
    assert status["ready"] is False


def test_invalid_input_is_rejected(tmp_path: Path) -> None:
    worker = Worker("small", tmp_path)
    try:
        worker.transcribe({"path": str(tmp_path / "missing.wav")})
    except ValueError as error:
        assert "unavailable" in str(error)
    else:
        raise AssertionError("missing media should fail")


def test_transcription_preserves_detected_language_and_word_timestamps(tmp_path: Path) -> None:
    media = tmp_path / "input.wav"
    media.write_bytes(b"fixture")
    model = SimpleNamespace(
        transcribe=lambda *_args, **_kwargs: (
            [SimpleNamespace(start=0.1, end=0.9, text=" Hello ", words=[SimpleNamespace(word=" Hello", start=0.1, end=0.5, probability=0.8)])],
            SimpleNamespace(language="en"),
        )
    )
    worker = Worker("small", tmp_path)
    worker._model = model
    result = worker.transcribe({"path": str(media), "durationMs": 1000})
    assert result["language"] == "en"
    assert result["segments"][0]["startMs"] == 100
    assert result["segments"][0]["words"][0]["confidence"] == 0.8


def test_duration_limit_is_enforced(tmp_path: Path) -> None:
    media = tmp_path / "input.wav"
    media.write_bytes(b"fixture")
    try:
        Worker("small", tmp_path).transcribe({"path": str(media), "durationMs": 14_400_001})
    except ValueError as error:
        assert "duration" in str(error)
    else:
        raise AssertionError("oversized media should fail")


def test_malformed_line_returns_sanitized_error_and_worker_continues(tmp_path: Path) -> None:
    completed = subprocess.run(
        [sys.executable, str(Path(__file__).with_name("worker.py")), "--model-dir", str(tmp_path)],
        input='{"broken"\n{"id":"status-1","operation":"status"}\n',
        text=True,
        capture_output=True,
        check=True,
    )
    responses = [json.loads(line) for line in completed.stdout.splitlines()]
    assert responses[0]["error"]["message"] == "Transcription provider failed"
    assert responses[1]["id"] == "status-1"
    assert responses[1]["ok"] is True
