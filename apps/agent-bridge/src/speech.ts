import { randomUUID } from "node:crypto";

import type { z } from "zod/v4";
import { BridgeError, type HeadlessClient } from "./headless";
import type { HeadlessRequest } from "./headless-contract";
import type { JobTaskContext } from "./jobs";
import { type Logger, NOOP_LOGGER } from "./logger";
import {
  commitGeneratedAssetResultSchema,
  projectStateSchema,
  replaceGeneratedAssetResultSchema,
  type schemas,
  speechEstimateSchema,
  speechPreviewResultSchema,
  speechRegenerateResultSchema,
  type speechSourceSchema,
  speechTextOptionsSchema,
  speechVoiceListSchema,
  ttsResultSchema,
  ttsStatusSchema,
} from "./schemas";

const WHITESPACE_PATTERN = /\s+/gu;

export interface SpeechSynthesisRequest {
  language: string;
  speed: number;
  text: string;
  textOptions?: z.infer<typeof speechTextOptionsSchema>;
  voiceId: string;
}

export interface SpeechProviderEstimate {
  chunks: number;
  expectedDurationMs: number;
  maximumDurationMs: number;
  minimumDurationMs: number;
}

export interface SynthesizedSpeech {
  durationMs: number;
  modelId: string;
  modelVersion: string | null;
  outputPath: string;
  providerId: string;
  request: SpeechSynthesisRequest;
  sampleRateHz: number;
}

export interface SpeechSynthesizer {
  cancel: () => void;
  cleanup: (path: string) => Promise<void>;
  close: () => Promise<void>;
  estimate?: (
    request: SpeechSynthesisRequest
  ) => Promise<SpeechProviderEstimate>;
  listVoices: () => Promise<z.infer<typeof speechVoiceListSchema>>;
  queueStatus: () => {
    active: number;
    concurrency: 1;
    fairness: "fifo";
    maxQueued: number;
    queued: number;
  };
  status: () => Promise<z.infer<typeof ttsStatusSchema>>;
  synthesize: (
    request: SpeechSynthesisRequest,
    signal?: AbortSignal
  ) => Promise<SynthesizedSpeech>;
}

type GenerateAndInsertInput = z.infer<(typeof schemas)["ttsGenerateAndInsert"]>;
type SpeechSource = z.infer<typeof speechSourceSchema>;
type CommitGeneratedAsset = (
  request: Extract<HeadlessRequest, { operation: "commit_generated_asset" }>
) => Promise<z.infer<typeof commitGeneratedAssetResultSchema>>;
type ReplaceGeneratedAsset = (
  request: Extract<HeadlessRequest, { operation: "replace_generated_asset" }>
) => Promise<z.infer<typeof replaceGeneratedAssetResultSchema>>;

type RetainedCommit =
  | { input: GenerateAndInsertInput; type: "insert" }
  | {
      expectedRevision: number;
      itemId: string;
      projectId: string;
      type: "replace";
    };

interface RetainedArtifact {
  commit?: RetainedCommit;
  expiresAtMs: number;
  generated: SynthesizedSpeech;
  timer: ReturnType<typeof setTimeout>;
}

export class GeneratedArtifactConflictError extends BridgeError {
  readonly generatedArtifact: { expiresAtMs: number; token: string };

  constructor(
    token: string,
    expiresAtMs: number,
    message: string,
    cause: unknown
  ) {
    super("REVISION_CONFLICT", message, true, { cause });
    this.generatedArtifact = { expiresAtMs, token };
  }
}

export class SpeechApplicationService {
  readonly #artifacts = new Map<string, RetainedArtifact>();
  readonly #artifactTtlMs: number;
  readonly #commitGeneratedAsset: CommitGeneratedAsset;
  readonly #headless: HeadlessClient | undefined;
  readonly #logger: Logger;
  readonly #now: () => number;
  readonly #provider: SpeechSynthesizer;
  readonly #replaceGeneratedAsset: ReplaceGeneratedAsset | undefined;
  #closed = false;

  constructor(
    provider: SpeechSynthesizer,
    headlessOrCommit: HeadlessClient | CommitGeneratedAsset,
    artifactTtlMs = 600_000,
    now: () => number = Date.now,
    logger: Logger = NOOP_LOGGER
  ) {
    this.#provider = provider;
    this.#artifactTtlMs = artifactTtlMs;
    this.#now = now;
    this.#logger = logger;
    this.#headless =
      typeof headlessOrCommit === "function" ? undefined : headlessOrCommit;
    this.#commitGeneratedAsset =
      typeof headlessOrCommit === "function"
        ? headlessOrCommit
        : async (request) =>
            await headlessOrCommit.call(
              request,
              commitGeneratedAssetResultSchema
            );
    this.#replaceGeneratedAsset =
      typeof headlessOrCommit === "function"
        ? undefined
        : async (request) =>
            await headlessOrCommit.call(
              request,
              replaceGeneratedAssetResultSchema
            );
  }

  async status() {
    this.#cleanupExpired();
    const status = ttsStatusSchema.parse(await this.#provider.status());
    validateStatus(status);
    return { ...status, queue: this.#provider.queueStatus() };
  }

  async listVoices(language?: string) {
    const status = await this.status();
    const voices = speechVoiceListSchema.parse(
      await this.#provider.listVoices()
    );
    validateVoiceCatalog(status, voices);
    return language
      ? voices.filter((voice) => voice.language === language)
      : voices;
  }

  async estimate(source: SpeechSource) {
    this.#assertOpen();
    const status = await this.status();
    const request = await this.#resolveSource(source, status);
    const estimate = this.#provider.estimate
      ? await this.#provider.estimate(request)
      : defaultEstimate(request);
    return speechEstimateSchema.parse({
      characters: Array.from(request.text).length,
      chunks: estimate.chunks,
      cost: { amount: 0, billing: "local", currency: null },
      estimatedDurationMs: {
        expected: estimate.expectedDurationMs,
        maximum: estimate.maximumDurationMs,
        minimum: estimate.minimumDurationMs,
      },
      language: request.language,
      modelCached: status.modelCached,
      modelId: status.modelId,
      modelLoaded: status.modelLoaded,
      providerId: status.providerId,
      queue: this.#provider.queueStatus(),
      resources: status.resources,
      voice: request.voiceId,
    });
  }

  async preview(source: SpeechSource, context: JobTaskContext) {
    this.#assertOpen();
    const status = await this.status();
    const request = await this.#resolveSource(source, status);
    context.onProgress(0.05);
    const generated = await this.#synthesize(request, context);
    try {
      validateSynthesis(status, request, generated);
      const retained = this.#retain(generated);
      context.onProgress(1);
      return speechPreviewResultSchema.parse({
        durationMs: generated.durationMs,
        expiresAtMs: retained.expiresAtMs,
        language: request.language,
        modelId: generated.modelId,
        modelVersion: generated.modelVersion,
        providerId: generated.providerId,
        sampleRateHz: generated.sampleRateHz,
        token: retained.token,
        voice: request.voiceId,
      });
    } catch (error) {
      await this.#cleanupWithoutMasking(generated.outputPath);
      throw error;
    }
  }

  previewAudio(token: string) {
    const artifact = this.#artifact(token);
    return { mimeType: "audio/wav", path: artifact.generated.outputPath };
  }

  async discardPreview(token: string) {
    const artifact = this.#artifact(token);
    await this.#discard(token, artifact);
    return { discarded: true, token };
  }

  async commitPreview(
    token: string,
    projectId: string,
    expectedRevision: number,
    placement:
      | { startMs: number; trackId: string; type: "insert" }
      | { itemId: string; type: "replace" }
  ) {
    const artifact = this.#artifact(token);
    artifact.commit =
      placement.type === "insert"
        ? {
            input: {
              expectedRevision,
              projectId,
              startMs: placement.startMs,
              text: artifact.generated.request.text,
              textOptions: artifact.generated.request.textOptions,
              trackId: placement.trackId,
            },
            type: "insert",
          }
        : {
            expectedRevision,
            itemId: placement.itemId,
            projectId,
            type: "replace",
          };
    return await this.#commitRetained(token, artifact, expectedRevision);
  }

  async regenerate(
    input: z.infer<(typeof schemas)["speechRegenerate"]>,
    context: JobTaskContext
  ) {
    const status = await this.status();
    const request = await this.#resolveSource(
      { ...input, type: "item" },
      status
    );
    context.onProgress(0.05);
    const generated = await this.#synthesize(request, context);
    try {
      validateSynthesis(status, request, generated);
      context.markNonCancellable();
      const result = await this.#commitReplace(
        input.projectId,
        input.expectedRevision,
        input.itemId,
        generated
      );
      const cleanupWarning = await this.#cleanupWarning(generated.outputPath);
      return speechRegenerateResultSchema.parse({
        ...this.#speechResult(result, generated),
        replacedAssetId: result.replacedAssetId,
        warnings: cleanupWarning
          ? [...result.warnings, cleanupWarning]
          : result.warnings,
      });
    } catch (error) {
      if (error instanceof BridgeError && error.code === "REVISION_CONFLICT") {
        const retained = this.#retain(generated, {
          expectedRevision: input.expectedRevision,
          itemId: input.itemId,
          projectId: input.projectId,
          type: "replace",
        });
        // biome-ignore lint/style/useErrorCause: the domain constructor stores the cause.
        throw new GeneratedArtifactConflictError(
          retained.token,
          retained.expiresAtMs,
          error.message,
          error
        );
      }
      await this.#cleanupWithoutMasking(generated.outputPath);
      throw error;
    }
  }

  async generateAndInsert(
    input: GenerateAndInsertInput,
    context: JobTaskContext
  ) {
    this.#assertOpen();
    const status = await this.status();
    if (!status.ready) {
      throw new BridgeError(
        "TTS_UNAVAILABLE",
        `Speech provider ${status.providerId} is not ready`
      );
    }
    const voices = speechVoiceListSchema.parse(
      await this.#provider.listVoices()
    );
    validateVoiceCatalog(status, voices);
    const request = resolveRequest(input, status, voices);

    context.onProgress(0.05);
    const generated = await this.#synthesize(request, context);
    try {
      validateSynthesis(status, request, generated);
      context.onProgress(0.7);
      if (context.signal.aborted) {
        throw new BridgeError(
          "JOB_CANCELLED",
          "Speech synthesis was cancelled",
          true
        );
      }
      context.markNonCancellable();
      const result = await this.#commit(input, generated);
      context.onProgress(0.95);
      const cleanupWarning = await this.#cleanupWarning(generated.outputPath);
      return cleanupWarning
        ? { ...result, warnings: [...result.warnings, cleanupWarning] }
        : result;
    } catch (error) {
      if (error instanceof BridgeError && error.code === "REVISION_CONFLICT") {
        const retained = this.#retain(generated, { input, type: "insert" });
        // biome-ignore lint/style/useErrorCause: the domain constructor stores the cause.
        throw new GeneratedArtifactConflictError(
          retained.token,
          retained.expiresAtMs,
          error.message,
          error
        );
      }
      await this.#cleanupWithoutMasking(generated.outputPath);
      throw error;
    }
  }

  async commitGeneratedArtifact(token: string, expectedRevision: number) {
    this.#assertOpen();
    this.#cleanupExpired();
    const retained = this.#artifacts.get(token);
    if (!retained) {
      throw new BridgeError(
        "GENERATED_ARTIFACT_NOT_FOUND",
        "Generated speech artifact was not found or has expired"
      );
    }
    try {
      return await this.#commitRetained(token, retained, expectedRevision);
    } catch (error) {
      if (error instanceof BridgeError && error.code === "REVISION_CONFLICT") {
        throw error;
      }
      await this.#discard(token, retained);
      throw error;
    }
  }

  cancel() {
    this.#provider.cancel();
  }

  async close() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#isClosed()) {
      return;
    }
    this.#closed = true;
    await Promise.allSettled(
      [...this.#artifacts].map(async ([token, artifact]) =>
        this.#discard(token, artifact)
      )
    );
    await this.#provider.close();
  }

  async #resolveSource(
    source: SpeechSource,
    status: z.infer<typeof ttsStatusSchema>
  ) {
    const voices = speechVoiceListSchema.parse(
      await this.#provider.listVoices()
    );
    validateVoiceCatalog(status, voices);
    if (source.type === "request") {
      return resolveRequest(source, status, voices);
    }
    if (!this.#headless) {
      throw new BridgeError(
        "INTERNAL_ERROR",
        "Project state reader is unavailable"
      );
    }
    const state = await this.#headless.call(
      { operation: "get_state", projectId: source.projectId },
      projectStateSchema
    );
    const item = state.project.tracks
      .flatMap((track) => track.items)
      .find((candidate) => candidate.id === source.itemId);
    if (item?.type !== "media") {
      throw new BridgeError("ITEM_NOT_FOUND", "Speech item was not found");
    }
    const asset = state.project.assets.find(
      (candidate) => candidate.id === item.assetId
    );
    if (asset?.origin?.type !== "speech_synthesis") {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Item does not contain persisted speech intent"
      );
    }
    const persisted = asset.origin.generation.request;
    return resolveRequest(
      {
        language: source.language ?? persisted.language,
        speed: source.speed ?? persisted.speed,
        text: source.text ?? persisted.text,
        textOptions: source.textOptions ?? persisted.textOptions,
        voice: source.voice ?? persisted.voiceId,
      },
      status,
      voices
    );
  }

  #artifact(token: string) {
    this.#cleanupExpired();
    const artifact = this.#artifacts.get(token);
    if (!artifact) {
      throw new BridgeError(
        "GENERATED_ARTIFACT_NOT_FOUND",
        "Generated speech artifact was not found or has expired"
      );
    }
    return artifact;
  }

  async #commitRetained(
    token: string,
    retained: RetainedArtifact,
    expectedRevision: number
  ) {
    if (!retained.commit) {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Speech preview requires an insertion or replacement placement"
      );
    }
    try {
      const result =
        retained.commit.type === "insert"
          ? await this.#commit(
              { ...retained.commit.input, expectedRevision },
              retained.generated
            )
          : await this.#commitReplace(
              retained.commit.projectId,
              expectedRevision,
              retained.commit.itemId,
              retained.generated
            );
      const cleanupWarning = await this.#discardWithWarning(token, retained);
      return cleanupWarning
        ? { ...result, warnings: [...result.warnings, cleanupWarning] }
        : result;
    } catch (error) {
      if (error instanceof BridgeError && error.code === "REVISION_CONFLICT") {
        throw error;
      }
      await this.#discard(token, retained);
      throw error;
    }
  }

  async #commit(input: GenerateAndInsertInput, generated: SynthesizedSpeech) {
    const { request } = generated;
    const committed = await this.#commitGeneratedAsset({
      displayName: `speech-${request.voiceId}.wav`,
      expectedRevision: input.expectedRevision,
      operation: "commit_generated_asset",
      origin: this.#origin(generated),
      path: generated.outputPath,
      projectId: input.projectId,
      startMs: input.startMs,
      trackId: input.trackId,
    });
    return ttsResultSchema.parse(this.#speechResult(committed, generated));
  }

  async #commitReplace(
    projectId: string,
    expectedRevision: number,
    itemId: string,
    generated: SynthesizedSpeech
  ) {
    if (!this.#replaceGeneratedAsset) {
      throw new BridgeError(
        "INTERNAL_ERROR",
        "Speech replacement is unavailable"
      );
    }
    return await this.#replaceGeneratedAsset({
      expectedRevision,
      itemId,
      operation: "replace_generated_asset",
      origin: this.#origin(generated),
      path: generated.outputPath,
      projectId,
    });
  }

  #origin(generated: SynthesizedSpeech) {
    return {
      generation: {
        generatedAtMs: this.#now(),
        modelId: generated.modelId,
        modelVersion: generated.modelVersion,
        providerId: generated.providerId,
        request: {
          ...generated.request,
          textOptions: speechTextOptionsSchema.parse(
            generated.request.textOptions ?? {}
          ),
        },
        sampleRateHz: generated.sampleRateHz,
      },
      type: "speech_synthesis" as const,
    };
  }

  #speechResult(
    committed: {
      assetId: string;
      itemId: string;
      revision: number;
      warnings: string[];
    },
    generated: SynthesizedSpeech
  ) {
    const { request } = generated;
    return {
      assetId: committed.assetId,
      durationMs: generated.durationMs,
      itemId: committed.itemId,
      language: request.language,
      modelId: generated.modelId,
      modelVersion: generated.modelVersion,
      providerId: generated.providerId,
      revision: committed.revision,
      voice: request.voiceId,
      warnings: committed.warnings,
    };
  }

  async #cleanupWarning(path: string): Promise<string | null> {
    try {
      await this.#provider.cleanup(path);
      this.#logger.debug("speech.cleanup.completed", {
        cleanupOutcome: "removed",
      });
      return null;
    } catch {
      this.#logger.warn("speech.cleanup.failed", {
        cleanupOutcome: "retry_on_shutdown",
        code: "TEMP_FILE_CLEANUP_FAILED",
      });
      return "TEMP_FILE_CLEANUP_FAILED";
    }
  }

  async #cleanupWithoutMasking(path: string) {
    try {
      await this.#provider.cleanup(path);
      this.#logger.debug("speech.cleanup.completed", {
        cleanupOutcome: "removed",
      });
    } catch {
      this.#logger.warn("speech.cleanup.failed", {
        cleanupOutcome: "retry_on_shutdown",
        code: "TEMP_FILE_CLEANUP_FAILED",
      });
      // The original operation error is authoritative; provider shutdown retries cleanup.
    }
  }

  async #synthesize(request: SpeechSynthesisRequest, context: JobTaskContext) {
    const startedAt = performance.now();
    const chunks = prepareSpeechSegments(request).length;
    this.#logger.info("speech.synthesis.started", {
      characters: Array.from(request.text).length,
      chunks,
      jobId: context.jobId,
      providerId: (await this.#provider.status()).providerId,
    });
    try {
      const generated = await this.#provider.synthesize(
        request,
        context.signal
      );
      this.#logger.info("speech.synthesis.completed", {
        characters: Array.from(request.text).length,
        chunks,
        jobId: context.jobId,
        providerId: generated.providerId,
        status: "completed",
        synthesisDurationMs: Math.round(performance.now() - startedAt),
      });
      return generated;
    } catch (error) {
      this.#logger.error("speech.synthesis.failed", {
        code: error instanceof BridgeError ? error.code : "TTS_PROVIDER_FAILED",
        jobId: context.jobId,
        status: "failed",
        synthesisDurationMs: Math.round(performance.now() - startedAt),
      });
      throw error;
    }
  }

  async #discardWithWarning(token: string, artifact: RetainedArtifact) {
    if (this.#artifacts.get(token) !== artifact) {
      return null;
    }
    const warning = await this.#cleanupWarning(artifact.generated.outputPath);
    if (!warning) {
      this.#artifacts.delete(token);
      clearTimeout(artifact.timer);
    }
    return warning;
  }

  #retain(generated: SynthesizedSpeech, commit?: RetainedCommit) {
    const token = randomUUID();
    const expiresAtMs = this.#now() + this.#artifactTtlMs;
    const timer = setTimeout(() => {
      const artifact = this.#artifacts.get(token);
      if (artifact) {
        this.#discard(token, artifact).catch(() => undefined);
      }
    }, this.#artifactTtlMs);
    timer.unref();
    this.#artifacts.set(token, {
      ...(commit ? { commit } : {}),
      expiresAtMs,
      generated,
      timer,
    });
    return { expiresAtMs, token };
  }

  async #discard(token: string, artifact: RetainedArtifact) {
    if (this.#artifacts.get(token) !== artifact) {
      return;
    }
    this.#artifacts.delete(token);
    clearTimeout(artifact.timer);
    await this.#provider.cleanup(artifact.generated.outputPath);
  }

  #cleanupExpired() {
    const now = this.#now();
    for (const [token, artifact] of this.#artifacts) {
      if (artifact.expiresAtMs <= now) {
        this.#discard(token, artifact).catch(() => undefined);
      }
    }
  }

  #assertOpen() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#isClosed()) {
      throw new BridgeError(
        "BRIDGE_SHUTTING_DOWN",
        "Speech service is shutting down",
        true
      );
    }
  }

  #isClosed() {
    return this.#closed;
  }
}

const resolveRequest = (
  input: {
    language?: string | undefined;
    speed?: number | undefined;
    text: string;
    textOptions?: z.input<typeof speechTextOptionsSchema> | undefined;
    voice?: string | undefined;
  },
  status: z.infer<typeof ttsStatusSchema>,
  voices: z.infer<typeof speechVoiceListSchema>
): SpeechSynthesisRequest => {
  const requestedLanguage = input.language;
  let voice: (typeof voices)[number] | undefined;
  if (input.voice) {
    voice = voices.find(
      (candidate) => candidate.id === input.voice && candidate.available
    );
  } else if (requestedLanguage) {
    voice = voices.find(
      (candidate) =>
        candidate.language === requestedLanguage &&
        candidate.isDefault &&
        candidate.available
    );
  } else {
    voice = voices.find(
      (candidate) =>
        candidate.id === status.defaultVoiceId && candidate.available
    );
  }
  if (!voice) {
    throw new BridgeError(
      "VALIDATION_FAILED",
      input.voice
        ? "Speech voice is not supported"
        : "Speech provider has no default voice for the requested language"
    );
  }
  const language = requestedLanguage ?? voice.language;
  if (voice.language !== language) {
    throw new BridgeError(
      "VALIDATION_FAILED",
      "Speech voice does not support the requested language"
    );
  }
  const speed = input.speed ?? status.defaultSpeed;
  const text = input.text.trim();
  if (Array.from(text).length > status.limits.maxTextCharacters) {
    throw new BridgeError(
      "VALIDATION_FAILED",
      `Speech text exceeds the provider limit of ${status.limits.maxTextCharacters} characters`
    );
  }
  if (speed < status.limits.minSpeed || speed > status.limits.maxSpeed) {
    throw new BridgeError(
      "VALIDATION_FAILED",
      `Speech speed must be between ${status.limits.minSpeed} and ${status.limits.maxSpeed}`
    );
  }
  return {
    language,
    speed,
    text,
    textOptions: speechTextOptionsSchema.parse(input.textOptions ?? {}),
    voiceId: voice.id,
  };
};

export const prepareSpeechSegments = (request: SpeechSynthesisRequest) => {
  const options = speechTextOptionsSchema.parse(request.textOptions ?? {});
  const { text: originalText } = request;
  let text = originalText;
  if (options.normalization === "basic") {
    text = text.normalize("NFC").replace(WHITESPACE_PATTERN, " ").trim();
  }
  for (const pronunciation of options.pronunciations) {
    text = text.split(pronunciation.term).join(pronunciation.spoken);
  }
  if (options.chunking === "none") {
    return [text];
  }
  const segments = [
    ...new Intl.Segmenter(request.language, {
      granularity: "sentence",
    }).segment(text),
  ]
    .map((segment) => segment.segment.trim())
    .filter(Boolean);
  return segments.length > 0 ? segments : [text];
};

const defaultEstimate = (
  request: SpeechSynthesisRequest
): SpeechProviderEstimate => {
  const segments = prepareSpeechSegments(request);
  const options = speechTextOptionsSchema.parse(request.textOptions ?? {});
  const characters = segments.reduce(
    (total, segment) => total + Array.from(segment).length,
    0
  );
  const expectedDurationMs = Math.max(
    1,
    Math.round(
      (characters / (15 * request.speed)) * 1000 +
        Math.max(0, segments.length - 1) * options.sentencePauseMs
    )
  );
  return {
    chunks: segments.length,
    expectedDurationMs,
    maximumDurationMs: Math.round(expectedDurationMs * 1.35),
    minimumDurationMs: Math.max(1, Math.round(expectedDurationMs * 0.75)),
  };
};

const validateStatus = (status: z.infer<typeof ttsStatusSchema>) => {
  const activeModel = status.models.find(
    (model) => model.id === status.modelId
  );
  if (
    !activeModel ||
    activeModel.sampleRateHz !== status.sampleRateHz ||
    activeModel.version !== status.modelVersion ||
    !status.devices.includes(status.device) ||
    !status.languages.includes(status.defaultLanguage) ||
    !status.voices.includes(status.defaultVoiceId)
  ) {
    throw new BridgeError(
      "TTS_INVALID_CAPABILITIES",
      "Speech provider returned inconsistent capabilities"
    );
  }
};

const validateVoiceCatalog = (
  status: z.infer<typeof ttsStatusSchema>,
  voices: z.infer<typeof speechVoiceListSchema>
) => {
  const advertised = [...status.voices].sort();
  const listed = voices.map((voice) => voice.id).sort();
  if (
    advertised.length !== listed.length ||
    advertised.some((voice, index) => voice !== listed[index]) ||
    voices.some(
      (voice) =>
        !status.languages.includes(voice.language) ||
        voice.providerId !== status.providerId ||
        voice.modelId !== status.modelId
    ) ||
    !voices.some(
      (voice) => voice.id === status.defaultVoiceId && voice.isDefault
    )
  ) {
    throw new BridgeError(
      "TTS_INVALID_CAPABILITIES",
      "Speech provider voice catalog does not match its status"
    );
  }
};

const validateSynthesis = (
  status: z.infer<typeof ttsStatusSchema>,
  request: SpeechSynthesisRequest,
  generated: SynthesizedSpeech
) => {
  if (
    generated.durationMs <= 0 ||
    generated.providerId !== status.providerId ||
    generated.modelId !== status.modelId ||
    generated.modelVersion !== status.modelVersion ||
    generated.sampleRateHz !== status.sampleRateHz ||
    generated.request.text !== request.text ||
    generated.request.language !== request.language ||
    generated.request.voiceId !== request.voiceId ||
    generated.request.speed !== request.speed
  ) {
    throw new BridgeError(
      "TTS_INVALID_OUTPUT",
      "Speech synthesis result does not match provider capabilities or request"
    );
  }
};
