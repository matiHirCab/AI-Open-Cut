"""Hermetic protocol/source checks for the test-only complete-scene worker."""
import json
import pathlib
import struct
import subprocess
import sys
import tempfile
import unittest
import wave


WORKER = pathlib.Path(__file__).with_name("reference_scene_worker.py")


class ReferenceWorkerTest(unittest.TestCase):
    def test_status_preserves_worker_contract_and_declares_no_native_timestamps(self):
        output = subprocess.run(
            [sys.executable, str(WORKER)],
            input=json.dumps({"id": "status", "operation": "status"}) + "\n",
            text=True, capture_output=True, check=True,
        )
        event = json.loads(output.stdout)
        self.assertEqual(set(event), {"id", "type", "result"})
        self.assertEqual(event["id"], "status")
        self.assertEqual(event["type"], "result")
        self.assertEqual(event["result"]["sampleRateHz"], 48000)
        self.assertEqual(event["result"]["timestampSupport"], {
            "sentence": False, "word": False, "phoneme": False,
        })

    def test_generate_has_independent_voice_windows_and_unchanged_metadata(self):
        with tempfile.TemporaryDirectory() as root:
            path = pathlib.Path(root) / "voice.wav"
            output = subprocess.run(
                [sys.executable, str(WORKER)],
                input=json.dumps({"id": "voice", "operation": "generate", "outputPath": str(path), "voice": "fixture", "language": "en-US"}) + "\n",
                text=True, capture_output=True, check=True,
            )
            result = json.loads(output.stdout)["result"]
            self.assertEqual(set(result), {"durationMs", "providerId", "modelId", "modelVersion", "sampleRateHz", "language", "voiceId"})
            self.assertEqual(result["durationMs"], 6000)
            with wave.open(str(path)) as source:
                self.assertEqual((source.getnchannels(), source.getsampwidth(), source.getframerate(), source.getnframes()), (2, 2, 48000, 288000))
                data = source.readframes(source.getnframes())
            for start, end in [(0, 500), (900, 1000), (1400, 1500), (1900, 2400), (2900, 3200), (3900, 4300), (5000, 6000)]:
                self.assertFalse(any(data[start * 48 * 4:end * 48 * 4]))
            for start, end in [(500, 900), (1000, 1400), (1500, 1900), (2400, 2900), (3200, 3900), (4300, 5000)]:
                self.assertTrue(any(data[start * 48 * 4:end * 48 * 4]))
            for offset in range(0, len(data), 4):
                left, right = struct.unpack_from("<hh", data, offset)
                self.assertEqual(left, right)
                self.assertLessEqual(abs(left), 3277)

    def test_sources_have_literal_bounded_native_media_facts(self):
        with tempfile.TemporaryDirectory() as root:
            subprocess.run([sys.executable, str(WORKER), "--sources", root], check=True)
            for name, frames in [("music", 288000), ("event0", 14400), ("event1", 14400)]:
                with wave.open(str(pathlib.Path(root) / (name + ".wav"))) as source:
                    self.assertEqual((source.getnchannels(), source.getsampwidth(), source.getframerate(), source.getnframes()), (2, 2, 48000, frames))
                    self.assertTrue(any(source.readframes(frames)))


if __name__ == "__main__":
    unittest.main()
