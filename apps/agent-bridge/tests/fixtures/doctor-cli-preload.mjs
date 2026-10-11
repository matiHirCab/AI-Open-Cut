import { mock } from "bun:test";
import { BridgeError, callHeadless, errorBody } from "../../src/headless.ts";
import {
  GeneratedArtifactConflictError,
  prepareSpeechSegments,
} from "../../src/speech.ts";
import { FasterWhisperTranscriber } from "../../src/transcription.ts";

const mode = process.env.OPENCUT_TEST_DOCTOR_MODE;
const close = (service) => {
  if (mode === `${service}-cleanup`) {
    throw new BridgeError("VALIDATION_FAILED", "Owned cleanup failed");
  }
};

mock.module("../../src/headless.ts", () => ({
  BridgeError,
  callHeadless,
  errorBody,
  HeadlessClient: class {
    close() {
      return close("headless");
    }
  },
}));
mock.module("../../src/speech.ts", () => ({
  GeneratedArtifactConflictError,
  prepareSpeechSegments,
  SpeechApplicationService: class {
    close() {
      return close("speech");
    }
  },
}));
mock.module("../../src/transcription.ts", () => ({
  FasterWhisperTranscriber,
  TranscriptionApplicationService: class {
    close() {
      return close("transcription");
    }
  },
}));
mock.module("../../src/doctor.ts", () => ({
  runDoctor: () => {
    if (mode === "doctor-cleanup") {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Doctor artifact cleanup failed"
      );
    }
    return { checks: [], ready: mode !== "unready", version: 1 };
  },
}));
