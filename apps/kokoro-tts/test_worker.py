import json
from io import StringIO
import os
from pathlib import Path
import tempfile
import unittest
import uuid
from unittest.mock import patch

import worker


class FakeBackend:
    loaded = True

    def synthesize(self, text, voice, speed):
        import numpy as np

        return np.array([0.0, 0.25, -0.25, 0.0], dtype="float32")


class FailingBackend:
    loaded = True

    def synthesize(self, text, voice, speed):
        raise RuntimeError("private model failure")


class WorkerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.model = self.root / "model"
        self.work = self.root / "work"
        self.model.mkdir()
        self.work.mkdir()
        self.environment = patch.dict(
            os.environ,
            {
                "OPENCUT_KOKORO_MODEL_DIR": str(self.model),
                "OPENCUT_TTS_WORK_DIR": str(self.work),
            },
        )
        self.environment.start()

    def tearDown(self):
        self.environment.stop()
        self.temporary.cleanup()

    def test_validates_request_bounds_and_voice(self):
        output = self.work / f"{uuid.uuid4()}.wav"
        with self.assertRaises(worker.WorkerError):
            worker.validate_generate(
                {"text": "", "voice": "af_heart", "speed": 1, "outputPath": str(output)}
            )
        with self.assertRaises(worker.WorkerError):
            worker.validate_generate(
                {"text": "hello", "voice": "ef_dora", "speed": 1, "outputPath": str(output)}
            )
        with self.assertRaises(worker.WorkerError):
            worker.validate_generate(
                {"text": "hello", "voice": "af_heart", "speed": 2.1, "outputPath": str(output)}
            )

    def test_confines_output_to_work_root(self):
        outside = self.root / f"{uuid.uuid4()}.wav"
        with self.assertRaises(worker.WorkerError) as raised:
            worker.validate_output_path(str(outside))
        self.assertEqual(raised.exception.code, "PATH_NOT_ALLOWED")

    def test_writes_pcm16_wav_atomically(self):
        import soundfile as sf

        output = self.work / f"{uuid.uuid4()}.wav"
        result = worker.generate(
            {
                "text": "hello",
                "language": "en-US",
                "voice": "af_heart",
                "speed": 1,
                "outputPath": str(output),
            },
            FakeBackend(),
        )
        info = sf.info(output)
        self.assertEqual(info.samplerate, 24_000)
        self.assertEqual(info.channels, 1)
        self.assertEqual(info.subtype, "PCM_16")
        self.assertEqual(result["durationMs"], 1)
        self.assertEqual(result["providerId"], worker.PROVIDER_ID)
        self.assertEqual(result["modelId"], worker.MODEL_ID)
        self.assertEqual(result["sampleRateHz"], worker.SAMPLE_RATE_HZ)
        self.assertEqual(result["voiceId"], "af_heart")
        self.assertEqual(list(self.work.glob("*.tmp")), [])

    def test_sentence_segments_and_pause_produce_one_wav(self):
        import soundfile as sf

        output = self.work / f"{uuid.uuid4()}.wav"
        result = worker.generate(
            {
                "text": "One. Two.",
                "segments": ["One.", "Two."],
                "sentencePauseMs": 100,
                "language": "en-US",
                "voice": "af_heart",
                "speed": 1,
                "outputPath": str(output),
            },
            FakeBackend(),
        )
        audio, sample_rate = sf.read(output)
        self.assertEqual(sample_rate, worker.SAMPLE_RATE_HZ)
        self.assertEqual(len(audio), 8 + 2_400)
        self.assertEqual(result["durationMs"], 100)
        self.assertEqual(list(self.work.glob("*.wav")), [output])

    def test_reports_synthesis_failure_without_leaking_details(self):
        output = self.work / f"{uuid.uuid4()}.wav"
        request = json.dumps(
            {
                "id": "failure",
                "operation": "generate",
                "text": "hello",
                "language": "en-US",
                "voice": "af_heart",
                "speed": 1,
                "outputPath": str(output),
            }
        )
        stdout = StringIO()
        with (
            patch.object(worker.sys, "stdin", StringIO(request + "\n")),
            patch.object(worker.sys, "stdout", stdout),
            patch.object(worker.sys, "stderr", StringIO()),
        ):
            worker.serve(FailingBackend())
        response = json.loads(stdout.getvalue())
        self.assertEqual(response["error"]["code"], "TTS_SYNTHESIS_FAILED")
        self.assertNotIn("private model failure", response["error"]["message"])
        self.assertFalse(output.exists())

    def test_capabilities_are_the_source_of_provider_contract_values(self):
        capabilities = worker.status(FakeBackend())
        voices = worker.list_voices()
        self.assertEqual(capabilities["providerId"], worker.PROVIDER_ID)
        self.assertEqual(capabilities["modelId"], worker.MODEL_ID)
        self.assertEqual(capabilities["sampleRateHz"], worker.SAMPLE_RATE_HZ)
        self.assertEqual(capabilities["voices"], [voice["id"] for voice in voices])
        active_model = next(
            model
            for model in capabilities["models"]
            if model["id"] == capabilities["modelId"]
        )
        self.assertEqual(active_model["sampleRateHz"], capabilities["sampleRateHz"])
        self.assertTrue(
            any(
                voice["id"] == capabilities["defaultVoiceId"]
                and voice["isDefault"]
                for voice in voices
            )
        )
        self.assertEqual(capabilities["resources"]["minimumLogicalCpus"], 2)
        self.assertEqual(capabilities["resources"]["recommendedLogicalCpus"], 4)
        for voice in voices:
            for field in (
                "accent",
                "available",
                "label",
                "locale",
                "modelId",
                "previewSupported",
                "providerId",
            ):
                self.assertIn(field, voice)

    def test_readiness_marker_requires_exact_runtime_contract(self):
        versions = {
            "kokoro": "0.9.4",
            "torch": "2.8.0",
            "soundfile": "0.13.1",
            "numpy": "2.4.6",
        }
        marker = {
            "markerVersion": worker.READY_MARKER_VERSION,
            "providerId": worker.PROVIDER_ID,
            "modelId": worker.MODEL_ID,
            "modelVersion": worker.MODEL_VERSION,
            "sampleRateHz": worker.SAMPLE_RATE_HZ,
            "voices": list(worker.VOICES),
            "dependencies": versions,
        }
        worker._ready_marker().write_text(json.dumps(marker), encoding="utf-8")
        self.assertTrue(worker.readiness_marker_valid(versions))

        marker["sampleRateHz"] = 16_000
        worker._ready_marker().write_text(json.dumps(marker), encoding="utf-8")
        self.assertFalse(worker.readiness_marker_valid(versions))

    def test_readiness_marker_rejects_corrupt_and_dependency_drift(self):
        versions = {
            "kokoro": "0.9.4",
            "torch": "2.8.0",
            "soundfile": "0.13.1",
            "numpy": "2.4.6",
        }
        worker._ready_marker().write_text("not-json", encoding="utf-8")
        self.assertFalse(worker.readiness_marker_valid(versions))
        worker._ready_marker().write_text(
            json.dumps(
                {
                    "markerVersion": worker.READY_MARKER_VERSION,
                    "providerId": worker.PROVIDER_ID,
                    "modelId": worker.MODEL_ID,
                    "modelVersion": worker.MODEL_VERSION,
                    "sampleRateHz": worker.SAMPLE_RATE_HZ,
                    "voices": list(worker.VOICES),
                    "dependencies": {**versions, "torch": "old"},
                }
            ),
            encoding="utf-8",
        )
        self.assertFalse(worker.readiness_marker_valid(versions))

    def test_shared_provider_contract_has_consistent_wire_values(self):
        contract_path = (
            Path(__file__).resolve().parents[2]
            / "contracts"
            / "speech-provider-v1.json"
        )
        contract = json.loads(contract_path.read_text(encoding="utf-8"))
        status = contract["status"]
        synthesis = contract["synthesis"]
        origin = contract["origin"]["generation"]
        self.assertEqual(
            status["voices"], [voice["id"] for voice in contract["voices"]]
        )
        for field in ("providerId", "modelId", "sampleRateHz"):
            self.assertEqual(synthesis[field], status[field])
            self.assertEqual(origin[field], status[field])
        self.assertEqual(synthesis["voiceId"], origin["request"]["voiceId"])

    def test_worker_error_codes_exist_in_shared_catalog(self):
        catalog_path = Path(__file__).resolve().parents[2] / "contracts" / "error-codes-v1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))["codes"]
        expected_retryability = {
            "INVALID_ARGUMENT": False,
            "PATH_NOT_ALLOWED": False,
            "TTS_INVALID_OUTPUT": False,
            "TTS_SYNTHESIS_FAILED": True,
            "TTS_UNAVAILABLE": False,
        }
        for code, retryable in expected_retryability.items():
            self.assertIn(code, catalog)
            self.assertEqual(catalog[code]["retryable"], retryable)
            self.assertTrue(catalog[code]["description"])


if __name__ == "__main__":
    unittest.main()
