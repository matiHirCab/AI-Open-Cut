"""Persistent, local-only Kokoro worker for the OpenCut agent bridge."""

from __future__ import annotations

import argparse
from contextlib import redirect_stdout
import importlib.metadata
import json
import os
from pathlib import Path
import sys
import traceback
import uuid

MODEL_ID = "hexgrad/Kokoro-82M"
MODEL_VERSION = None
PROVIDER_ID = "kokoro"
SAMPLE_RATE_HZ = 24_000
MAX_TEXT_CHARACTERS = 5_000
MIN_SPEED = 0.5
MAX_SPEED = 2.0
DEFAULT_SPEED = 1.0
DEFAULT_LANGUAGE = "en-US"
DEFAULT_VOICES = {"en-US": "af_heart", "en-GB": "bf_emma"}
READY_MARKER_VERSION = 1
RUNTIME_DEPENDENCIES = ("kokoro", "torch", "soundfile", "numpy")
VOICES = (
    "af_alloy",
    "af_aoede",
    "af_bella",
    "af_heart",
    "af_jessica",
    "af_kore",
    "af_nicole",
    "af_nova",
    "af_river",
    "af_sarah",
    "af_sky",
    "am_adam",
    "am_echo",
    "am_eric",
    "am_fenrir",
    "am_liam",
    "am_michael",
    "am_onyx",
    "am_puck",
    "am_santa",
    "bf_alice",
    "bf_emma",
    "bf_isabella",
    "bf_lily",
    "bm_daniel",
    "bm_fable",
    "bm_george",
    "bm_lewis",
)


def language_for_voice(voice: str) -> str:
    return "en-US" if voice.startswith(("af_", "am_")) else "en-GB"


def list_voices() -> list[dict[str, object]]:
    return [
        {
            "accent": "American English" if voice.startswith(("af_", "am_")) else "British English",
            "available": True,
            "id": voice,
            "label": voice.split("_", 1)[1].replace("_", " ").title(),
            "language": language_for_voice(voice),
            "locale": language_for_voice(voice),
            "modelId": MODEL_ID,
            "previewSupported": True,
            "providerId": PROVIDER_ID,
            "isDefault": DEFAULT_VOICES.get(language_for_voice(voice)) == voice,
        }
        for voice in VOICES
    ]


class WorkerError(Exception):
    def __init__(self, code: str, message: str, retryable: bool = False) -> None:
        super().__init__(message)
        self.code = code
        self.retryable = retryable


class KokoroBackend:
    """Loads the model once and reuses language pipelines across requests."""

    def __init__(self) -> None:
        self._model = None
        self._pipelines: dict[str, object] = {}

    @property
    def loaded(self) -> bool:
        return self._model is not None

    def _ensure_loaded(self) -> None:
        if self._model is not None:
            return
        from kokoro import KModel, KPipeline

        self._model = KModel().to("cpu").eval()
        self._pipelines = {
            code: KPipeline(lang_code=code, model=False) for code in ("a", "b")
        }

    def preload_voices(self) -> None:
        self._ensure_loaded()
        for voice in VOICES:
            self._pipelines[voice[0]].load_voice(voice)

    def synthesize(self, text: str, voice: str, speed: float):
        import numpy as np

        self._ensure_loaded()
        pipeline = self._pipelines[voice[0]]
        voice_pack = pipeline.load_voice(voice)
        chunks = []
        with __import__("torch").inference_mode():
            for _graphemes, phonemes, _audio in pipeline(text, voice, speed):
                if not phonemes:
                    continue
                reference = voice_pack[len(phonemes) - 1]
                audio = self._model(phonemes, reference, speed)
                chunks.append(audio.detach().cpu().numpy().reshape(-1))
        if not chunks:
            raise WorkerError("TTS_INVALID_OUTPUT", "Kokoro produced no audio")
        return np.concatenate(chunks).astype("float32", copy=False)


def _model_root() -> Path:
    configured = os.getenv("OPENCUT_KOKORO_MODEL_DIR")
    if not configured:
        raise WorkerError(
            "TTS_UNAVAILABLE", "OPENCUT_KOKORO_MODEL_DIR is required"
        )
    root = Path(configured).expanduser()
    root.mkdir(parents=True, exist_ok=True)
    return root.resolve(strict=True)


def _work_root() -> Path:
    configured = os.getenv("OPENCUT_TTS_WORK_DIR")
    if not configured:
        raise WorkerError("TTS_UNAVAILABLE", "OPENCUT_TTS_WORK_DIR is required")
    root = Path(configured).expanduser()
    root.mkdir(parents=True, exist_ok=True)
    return root.resolve(strict=True)


def _ready_marker() -> Path:
    return _model_root() / "opencut-kokoro-ready.json"


def dependency_versions() -> dict[str, str] | None:
    try:
        return {
            dependency: importlib.metadata.version(dependency)
            for dependency in RUNTIME_DEPENDENCIES
        }
    except importlib.metadata.PackageNotFoundError:
        return None


def readiness_marker_valid(versions: dict[str, str]) -> bool:
    try:
        marker = json.loads(_ready_marker().read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    return marker == {
        "markerVersion": READY_MARKER_VERSION,
        "providerId": PROVIDER_ID,
        "modelId": MODEL_ID,
        "modelVersion": MODEL_VERSION,
        "sampleRateHz": SAMPLE_RATE_HZ,
        "voices": list(VOICES),
        "dependencies": versions,
    }


def validate_output_path(raw_path: object) -> Path:
    if not isinstance(raw_path, str) or not raw_path:
        raise WorkerError("INVALID_ARGUMENT", "outputPath is required")
    root = _work_root()
    candidate = Path(raw_path).expanduser()
    if not candidate.is_absolute():
        raise WorkerError("PATH_NOT_ALLOWED", "outputPath must be absolute")
    if candidate.suffix.lower() != ".wav" or candidate.name != candidate.name.lower():
        raise WorkerError("PATH_NOT_ALLOWED", "outputPath must be a lowercase WAV file")
    try:
        uuid.UUID(candidate.stem)
    except ValueError as error:
        raise WorkerError(
            "PATH_NOT_ALLOWED", "outputPath must use a UUID file name"
        ) from error
    if candidate.parent.resolve(strict=True) != root:
        raise WorkerError("PATH_NOT_ALLOWED", "outputPath is outside the TTS work root")
    return candidate


def validate_generate(
    request: dict[str, object],
) -> tuple[str, str, str, float, Path]:
    text = request.get("text")
    language = request.get("language")
    voice = request.get("voice")
    speed = request.get("speed")
    if not isinstance(text, str) or not text.strip():
        raise WorkerError("INVALID_ARGUMENT", "text cannot be empty")
    if len(text) > MAX_TEXT_CHARACTERS:
        raise WorkerError("INVALID_ARGUMENT", "text exceeds 5000 characters")
    if voice not in VOICES:
        raise WorkerError("INVALID_ARGUMENT", "voice is not supported")
    if not isinstance(language, str) or language != language_for_voice(str(voice)):
        raise WorkerError(
            "INVALID_ARGUMENT", "voice does not support the requested language"
        )
    if isinstance(speed, bool) or not isinstance(speed, (int, float)):
        raise WorkerError("INVALID_ARGUMENT", "speed must be a number")
    speed_value = float(speed)
    if not MIN_SPEED <= speed_value <= MAX_SPEED:
        raise WorkerError("INVALID_ARGUMENT", "speed must be between 0.5 and 2.0")
    return (
        text.strip(),
        language,
        str(voice),
        speed_value,
        validate_output_path(request.get("outputPath")),
    )


def status(backend: KokoroBackend) -> dict[str, object]:
    versions = dependency_versions()
    version = versions["kokoro"] if versions else "unavailable"
    dependencies_ready = versions is not None
    cached = versions is not None and readiness_marker_valid(versions)
    return {
        "ready": dependencies_ready and cached,
        "version": version,
        "providerId": PROVIDER_ID,
        "modelId": MODEL_ID,
        "modelVersion": MODEL_VERSION,
        "models": [
            {
                "id": MODEL_ID,
                "version": MODEL_VERSION,
                "sampleRateHz": SAMPLE_RATE_HZ,
            }
        ],
        "device": "cpu",
        "devices": ["cpu"],
        "modelCached": cached,
        "modelLoaded": backend.loaded,
        "sampleRateHz": SAMPLE_RATE_HZ,
        "languages": sorted(DEFAULT_VOICES),
        "voices": list(VOICES),
        "defaultLanguage": DEFAULT_LANGUAGE,
        "defaultVoiceId": DEFAULT_VOICES[DEFAULT_LANGUAGE],
        "defaultSpeed": DEFAULT_SPEED,
        "limits": {
            "maxTextCharacters": MAX_TEXT_CHARACTERS,
            "minSpeed": MIN_SPEED,
            "maxSpeed": MAX_SPEED,
        },
        "resources": {
            "execution": "local",
            "minimumLogicalCpus": 2,
            "recommendedLogicalCpus": 4,
            "minimumRamBytes": 2 * 1024**3,
            "recommendedRamBytes": 4 * 1024**3,
        },
    }


def generate(request: dict[str, object], backend: KokoroBackend) -> dict[str, object]:
    import numpy as np
    import soundfile as sf

    text, language, voice, speed, output = validate_generate(request)
    # Third-party model code may print download or pipeline notices. MCP owns
    # stdout, so keep every non-protocol message on stderr.
    raw_segments = request.get("segments", [text])
    if not isinstance(raw_segments, list) or not raw_segments or not all(isinstance(segment, str) and segment.strip() for segment in raw_segments):
        raise WorkerError("INVALID_ARGUMENT", "segments must contain non-empty text")
    pause_ms = request.get("sentencePauseMs", 0)
    if isinstance(pause_ms, bool) or not isinstance(pause_ms, int) or not 0 <= pause_ms <= 5_000:
        raise WorkerError("INVALID_ARGUMENT", "sentencePauseMs is invalid")
    rendered = []
    with redirect_stdout(sys.stderr):
        for index, segment in enumerate(raw_segments):
            rendered.append(backend.synthesize(segment.strip(), voice, speed))
            if index + 1 < len(raw_segments) and pause_ms:
                rendered.append(np.zeros(round(SAMPLE_RATE_HZ * pause_ms / 1000), dtype="float32"))
    synthesized = np.concatenate(rendered)
    audio = np.asarray(synthesized, dtype="float32").reshape(-1)
    if audio.size == 0 or not np.isfinite(audio).all():
        raise WorkerError("TTS_INVALID_OUTPUT", "Kokoro produced invalid audio")
    temporary = output.with_name(f".{output.name}.{uuid.uuid4().hex}.tmp")
    try:
        sf.write(temporary, audio, SAMPLE_RATE_HZ, format="WAV", subtype="PCM_16")
        os.replace(temporary, output)
    finally:
        temporary.unlink(missing_ok=True)
    return {
        "durationMs": max(1, round(audio.size * 1000 / SAMPLE_RATE_HZ)),
        "providerId": PROVIDER_ID,
        "modelId": MODEL_ID,
        "modelVersion": MODEL_VERSION,
        "sampleRateHz": SAMPLE_RATE_HZ,
        "language": language,
        "voiceId": voice,
    }


def emit(message: dict[str, object]) -> None:
    sys.stdout.write(json.dumps(message, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def serve(backend: KokoroBackend | None = None) -> None:
    active_backend = backend or KokoroBackend()
    for raw_line in sys.stdin:
        if not raw_line.strip():
            continue
        request_id: object = None
        try:
            request = json.loads(raw_line)
            if not isinstance(request, dict):
                raise WorkerError("INVALID_ARGUMENT", "request must be an object")
            request_id = request.get("id")
            operation = request.get("operation")
            if operation == "status":
                result = status(active_backend)
            elif operation == "list_voices":
                result = list_voices()
            elif operation == "generate":
                result = generate(request, active_backend)
            else:
                raise WorkerError("INVALID_ARGUMENT", "operation is not supported")
            emit({"id": request_id, "type": "result", "result": result})
        except WorkerError as error:
            emit(
                {
                    "id": request_id,
                    "type": "error",
                    "error": {
                        "code": error.code,
                        "message": str(error),
                        "retryable": error.retryable,
                    },
                }
            )
        except Exception:
            traceback.print_exc(file=sys.stderr)
            emit(
                {
                    "id": request_id,
                    "type": "error",
                    "error": {
                        "code": "TTS_SYNTHESIS_FAILED",
                        "message": "Kokoro synthesis failed",
                        "retryable": True,
                    },
                }
            )


def prepare() -> None:
    os.environ.pop("HF_HUB_OFFLINE", None)
    root = _model_root()
    os.environ["HF_HOME"] = str(root)
    os.environ["CUDA_VISIBLE_DEVICES"] = ""
    backend = KokoroBackend()
    with redirect_stdout(sys.stderr):
        backend.preload_voices()
    output = _work_root() / f"{uuid.uuid4()}.wav"
    try:
        generate(
            {
                "text": "OpenCut local speech is ready.",
                "language": DEFAULT_LANGUAGE,
                "voice": "af_heart",
                "speed": 1.0,
                "outputPath": str(output),
            },
            backend,
        )
    finally:
        output.unlink(missing_ok=True)
    versions = dependency_versions()
    if versions is None:
        raise WorkerError("TTS_UNAVAILABLE", "Kokoro dependencies are incomplete")
    _ready_marker().write_text(
        json.dumps(
            {
                "markerVersion": READY_MARKER_VERSION,
                "providerId": PROVIDER_ID,
                "modelId": MODEL_ID,
                "modelVersion": MODEL_VERSION,
                "sampleRateHz": SAMPLE_RATE_HZ,
                "voices": list(VOICES),
                "dependencies": versions,
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    print("Kokoro model, English voices, and CPU synthesis are ready.")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--prepare", action="store_true")
    arguments = parser.parse_args()
    if arguments.prepare:
        prepare()
    else:
        serve()


if __name__ == "__main__":
    main()
