import {
  PLANNING_LIMITS,
  PLANNING_WARNINGS,
  PlanningError,
  type PlanningEvidence,
  type PlanningWarning,
  type ShortPlanningInput,
  type ShortPlanningSource,
  type ShortPlanningSuccess,
  shortPlanningInputSchema,
  shortPlanningSourceSchema,
  shortPlanningSuccessSchema,
} from "./contract";

export const parsePlanningInput = (value: unknown): ShortPlanningInput => {
  let encoded: string | undefined;
  try {
    encoded = JSON.stringify(value);
  } catch (cause) {
    throw new PlanningError("input", { cause });
  }
  if (encoded === undefined) {
    throw new PlanningError("input");
  }
  if (Buffer.byteLength(encoded, "utf8") > PLANNING_LIMITS.inputBytes) {
    throw new PlanningError("limit");
  }
  const parsed = shortPlanningInputSchema.safeParse(value);
  if (!parsed.success) {
    throw new PlanningError("input");
  }
  return parsed.data;
};

type Segment = Omit<PlanningEvidence, "fullyIncluded">;
interface Range {
  endMs: number;
  startMs: number;
}
const overlaps = (a: Range, b: Range) =>
  a.startMs < b.endMs && b.startMs < a.endMs;
const fail = (): never => {
  throw new PlanningError("input");
};
const validRange = (range: Range, limit: number) =>
  range.startMs < range.endMs && range.endMs <= limit;

type Chunk = ShortPlanningInput["chunks"][number];
type SourceSegment = Chunk["segments"][number];
const absoluteWords = (segment: SourceSegment, offsetMs: number) => {
  let previous = segment.startMs;
  return segment.words?.map((word) => {
    if (word.startMs < previous || !validRange(word, segment.endMs)) {
      fail();
    }
    previous = word.endMs;
    return {
      endMs: offsetMs + word.endMs,
      startMs: offsetMs + word.startMs,
      word: word.word,
    };
  });
};
const flattenChunk = (chunk: Chunk): Segment[] => {
  const identifiers = new Set<string>();
  let previous = 0;
  return chunk.segments.map((segment) => {
    if (
      identifiers.has(segment.id) ||
      segment.startMs < previous ||
      !validRange(segment, chunk.durationMs)
    ) {
      fail();
    }
    identifiers.add(segment.id);
    previous = segment.endMs;
    const words = absoluteWords(segment, chunk.offsetMs);
    return {
      chunkId: chunk.id,
      endMs: chunk.offsetMs + segment.endMs,
      segmentId: segment.id,
      startMs: chunk.offsetMs + segment.startMs,
      text: segment.text,
      ...(words === undefined ? {} : { words }),
    };
  });
};
const flatten = (input: ShortPlanningInput, durationMs: number): Segment[] => {
  const segments: Segment[] = [];
  const chunks = new Set<string>();
  let cursor = 0;
  let words = 0;
  for (const chunk of input.chunks) {
    if (
      chunks.has(chunk.id) ||
      chunk.offsetMs !== cursor ||
      chunk.durationMs > chunk.maxDurationMs
    ) {
      fail();
    }
    chunks.add(chunk.id);
    cursor += chunk.durationMs;
    if (!Number.isSafeInteger(cursor) || cursor > durationMs) {
      fail();
    }
    const entries = flattenChunk(chunk);
    words += entries.reduce(
      (count, entry) => count + (entry.words?.length ?? 0),
      0
    );
    if (
      words > PLANNING_LIMITS.words ||
      segments.length + entries.length > PLANNING_LIMITS.segments
    ) {
      throw new PlanningError("limit");
    }
    segments.push(...entries);
  }
  if (cursor !== durationMs) {
    fail();
  }
  return segments;
};

const evidence = (segment: Segment, range: Range): PlanningEvidence => ({
  ...segment,
  fullyIncluded:
    segment.startMs >= range.startMs && segment.endMs <= range.endMs,
});

type Candidate = ShortPlanningSuccess["candidates"][number];
const candidateWarnings = (
  input: ShortPlanningInput,
  candidate: Omit<Candidate, "warnings">
): PlanningWarning[] => {
  const warnings = new Set<PlanningWarning>(["context_review"]);
  if (Object.values(candidate.editorial).some((text) => text.length > 0)) {
    warnings.add("editorial_claim_review");
  }
  const partial = candidate.evidence.some((quote) => !quote.fullyIncluded);
  if (partial) {
    warnings.add("cut_boundary");
  }
  if (partial || candidate.context.before || candidate.context.after) {
    warnings.add("context_excluded");
  }
  if (
    candidate.durationMs * 5 < input.targetDurationMs * 4 ||
    candidate.durationMs * 5 > input.targetDurationMs * 6
  ) {
    warnings.add("duration_mismatch");
  }
  const chunks = input.chunks.filter((chunk) =>
    overlaps(
      { endMs: chunk.offsetMs + chunk.durationMs, startMs: chunk.offsetMs },
      candidate
    )
  );
  if (chunks.length > 1) {
    warnings.add("chunk_boundary");
  }
  return PLANNING_WARNINGS.filter((warning) => warnings.has(warning));
};

/** Planning input validation only; managed-source validation remains in core. */
export const planShortCandidates = (
  value: unknown,
  resolvedSource: ShortPlanningSource
): ShortPlanningSuccess => {
  const input = parsePlanningInput(value);
  const parsedSource = shortPlanningSourceSchema.safeParse(resolvedSource);
  if (!parsedSource.success) {
    throw new PlanningError("source");
  }
  const source = parsedSource.data;
  if (
    source.assetId !== input.assetId ||
    source.projectId !== input.projectId
  ) {
    throw new PlanningError("source");
  }
  if (source.revision !== input.expectedRevision) {
    throw new PlanningError("stale");
  }
  const segments = flatten(input, source.durationMs);
  const selection = input.selection ?? { endMs: source.durationMs, startMs: 0 };
  if (!validRange(selection, source.durationMs)) {
    fail();
  }
  const included = segments.filter((segment) => overlaps(segment, selection));
  if (!included.length) {
    fail();
  }
  const ranges: Range[] = [];
  let start: number | undefined;
  let end = 0;
  for (const segment of included) {
    start ??= Math.max(segment.startMs, selection.startMs);
    end = Math.min(segment.endMs, selection.endMs);
    if (end - start >= input.targetDurationMs) {
      ranges.push({ endMs: end, startMs: start });
      start = undefined;
    }
  }
  if (start !== undefined) {
    ranges.push({ endMs: end, startMs: start });
  }
  if (ranges.length > input.maxCandidates) {
    throw new PlanningError("limit");
  }
  const edits = new Map<
    string,
    NonNullable<ShortPlanningInput["edits"]>[number]
  >();
  for (const edit of input.edits ?? []) {
    const index = ranges.findIndex(
      (_, i) => edit.candidateId === `candidate-${i + 1}`
    );
    if (
      index < 0 ||
      edits.has(edit.candidateId) ||
      (edit.startMs === undefined) !== (edit.endMs === undefined)
    ) {
      fail();
    }
    edits.set(edit.candidateId, edit);
  }
  const candidates = ranges.map(
    (range, index): ShortPlanningSuccess["candidates"][number] => {
      const id = `candidate-${index + 1}`;
      const edit = edits.get(id);
      const selected =
        edit?.startMs === undefined
          ? range
          : {
              endMs: edit.endMs ?? fail(),
              startMs: edit.startMs,
            };
      if (
        !validRange(selected, selection.endMs) ||
        selected.startMs < selection.startMs
      ) {
        fail();
      }
      const first = segments.findIndex((segment) =>
        overlaps(segment, selected)
      );
      if (first < 0) {
        fail();
      }
      let last = first;
      while (
        last + 1 < segments.length &&
        overlaps(segments[last + 1] ?? fail(), selected)
      ) {
        last += 1;
      }
      const quotes = segments
        .slice(first, last + 1)
        .map((segment) => evidence(segment, selected));
      const before = segments[first - 1];
      const after = segments[last + 1];
      const editorial = {
        cta: edit?.cta ?? "",
        hook: edit?.hook ?? "",
        rationale: edit?.rationale ?? "",
      };
      const durationMs = selected.endMs - selected.startMs;
      const candidate = {
        id,
        ...selected,
        context: {
          after: after ? evidence(after, selected) : null,
          before: before ? evidence(before, selected) : null,
        },
        durationMs,
        editorial,
        evidence: quotes,
      };
      return { ...candidate, warnings: candidateWarnings(input, candidate) };
    }
  );
  for (const candidate of candidates) {
    if (
      candidates.some(
        (other) => candidate !== other && overlaps(candidate, other)
      )
    ) {
      candidate.warnings.push("candidate_overlap");
    }
  }
  return shortPlanningSuccessSchema.parse({
    candidates,
    ok: true,
    selection,
    source: {
      ...source,
      chunks: input.chunks.map(({ segments: _segments, ...chunk }) => chunk),
      transcriptApproved: true,
      transcriptionProvenance: "supplied",
    },
    targetDurationMs: input.targetDurationMs,
    version: 1,
  });
};
