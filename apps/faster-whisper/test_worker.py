from pathlib import Path
from types import SimpleNamespace
import json
import subprocess
import sys
import pytest
from worker import Worker


def test_known_text_support_is_truthful_and_model_free(tmp_path: Path) -> None:
    support = Worker("small", tmp_path).status()["knownTextAlignment"]
    canonical = json.loads((Path(__file__).resolve().parents[2] / "contracts" / "known-text-alignment-v1.json").read_text(encoding="utf-8"))
    assert support == canonical["localSupport"]


@pytest.mark.parametrize("text,duration", [("", 1000), (" \n", 1000), ("é" * 2049, 1000), ("Hello", 30001), ("Hello", True)])
def test_known_text_bounds_reject_before_model_load(tmp_path: Path, text: str, duration: int) -> None:
    worker = Worker("small", tmp_path)
    with pytest.raises(ValueError):
        worker.align({"path": str(tmp_path / "missing"), "knownText": text, "durationMs": duration})
    assert worker._model is None


def test_known_tokens_use_audio_alignment_not_recognition(tmp_path: Path, monkeypatch) -> None:
    path = tmp_path / "source.wav"
    path.write_bytes(b"fixture")
    calls = []
    class Features:
        shape = (80, 101)
        def __getitem__(self, key):
            return self
    class Extractor:
        sampling_rate = 16000
        nb_max_frames = 3000
        def __call__(self, audio):
            return Features()
    class Tokens:
        sot_sequence = [1, 2]
        def __init__(self, *_args, **kwargs):
            assert kwargs == {"task": "transcribe", "language": "en"}
        def encode(self, text):
            calls.append(text)
            return [10, 11]
    monkeypatch.setitem(sys.modules, "faster_whisper.audio", SimpleNamespace(
        decode_audio=lambda *_args, **kwargs: [0] * 16000,
        pad_or_trim=lambda value: value))
    monkeypatch.setitem(sys.modules, "faster_whisper.tokenizer", SimpleNamespace(Tokenizer=Tokens))
    model = SimpleNamespace(hf_tokenizer=None, model=SimpleNamespace(is_multilingual=True),
                            max_length=448, feature_extractor=Extractor(), encode=lambda features: "encoded")
    def align(_tokenizer, tokens, encoded, frames):
        assert (tokens, encoded, frames) == ([[10, 11]], "encoded", 100)
        return [[{"word": "Hello", "start": 0.1, "end": 0.4}, {"word": " world", "start": 0.5, "end": 0.9}]]
    model.find_alignment = align
    worker = Worker("small", tmp_path); worker._model = model
    request = {"path": str(path), "knownText": "Hello world", "durationMs": 1000}
    result = worker.align(request)
    assert calls == ["Hello world"]
    assert result["alignment"]["quality"] == "forced"
    canonical = json.loads((Path(__file__).resolve().parents[2] / "contracts" / "known-text-alignment-v1.json").read_text(encoding="utf-8"))
    assert result["alignment"]["words"] == canonical["alignment"]["words"]
    assert result["alignment"]["words"][1] == {"text": " world", "startMs": 500, "endMs": 900}
    assert result["segments"][0]["text"] == "Hello world"
    model.find_alignment = lambda *_args: [[{"word": "Hello", "start": float("nan"), "end": 0.4}]]
    with pytest.raises(ValueError, match="finite"):
        worker.align(request)
    model.find_alignment = lambda *_args: [[{"word": "Hello", "start": 0.1, "end": 0.1}]]
    with pytest.raises(ValueError, match="degenerate"):
        worker.align(request)
    for timings in [[], [{"word": "Hello", "start": 0.1, "end": 1.1}],
                    [{"word": "Hello", "start": 0.1, "end": 0.5}, {"word": " world", "start": 0.4, "end": 0.9}]]:
        model.find_alignment = lambda *_args: [timings]
        with pytest.raises(ValueError):
            worker.align(request)
    monkeypatch.setitem(sys.modules, "faster_whisper.audio", SimpleNamespace(
        decode_audio=lambda *_args, **kwargs: [0] * 16032,
        pad_or_trim=lambda value: value))
    with pytest.raises(ValueError, match="decoded"):
        worker.align(request)
    model.max_length = 4
    with pytest.raises(ValueError, match="token context"):
        worker.align(request)


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
