import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { z } from "zod/v4";
import errors from "../../../contracts/error-codes-v1.json";
import protocol from "../../../contracts/headless-protocol-v1.json";

export const documentationRoot = resolve(import.meta.dirname, "../../..");
export const documentationGuide = resolve(
  documentationRoot,
  "docs/motion-graphics-release.md"
);
const failureIds = [
  "definition_cycle",
  "missing_definition",
  "stale_revision",
  "late_batch_rollback",
] as const;
const argumentsSchema = z.record(z.string(), z.unknown());
const exampleSchema = z
  .object({
    batch: z
      .object({
        arguments: argumentsSchema,
        tool: z.literal("timeline_batch_edit"),
      })
      .strict(),
    exampleVersion: z.literal(1),
    failures: z
      .array(
        z
          .object({
            arguments: argumentsSchema,
            expectedError: z
              .object({
                code: z.enum([
                  "INVALID_ARGUMENT",
                  "ITEM_NOT_FOUND",
                  "REVISION_CONFLICT",
                ]),
                retryable: z.boolean(),
              })
              .strict(),
            id: z.enum(failureIds),
            tool: z.string().min(1),
          })
          .strict()
      )
      .length(4),
    projectCreate: argumentsSchema,
    projectSchemaVersion: z.literal(44),
    protocolVersion: z.literal(1),
    requiredEditorCapabilities: z.array(z.string()).length(4),
    standalone: z
      .object({
        arguments: argumentsSchema,
        tool: z.literal("component_create"),
      })
      .strict(),
  })
  .strict();

// Only private teaching metadata is checked here. Actual MCP/core owns all
// project, component, timing, slot, reference and persistence validation.
export const parseDocumentationExample = (input: unknown) => {
  const example = exampleSchema.parse(input);
  if (
    new Set(example.failures.map((entry) => entry.id)).size !==
    failureIds.length
  ) {
    throw new Error("incomplete or duplicate documentation refusal corpus");
  }
  if (
    new Set(example.requiredEditorCapabilities).size !== 4 ||
    !example.requiredEditorCapabilities.every((name) =>
      protocol.status.editorCapabilities.includes(name)
    )
  ) {
    throw new Error("unknown or duplicate documented editor capability");
  }
  for (const entry of example.failures) {
    if (
      errors.codes[entry.expectedError.code].retryable !==
      entry.expectedError.retryable
    ) {
      throw new Error(
        "documented error retryability disagrees with canonical catalog"
      );
    }
  }
  return example;
};
export const readDocumentationExample = () =>
  parseDocumentationExample(
    JSON.parse(
      readFileSync(
        resolve(
          documentationRoot,
          "docs/examples/motion-graphics-release-v1.json"
        ),
        "utf8"
      )
    )
  );

export const bindDocumentationArguments = (
  input: Record<string, unknown>,
  bindings: Record<string, string>
): Record<string, unknown> => {
  const bind = (value: unknown): unknown => {
    if (typeof value === "string" && value.startsWith("$runtime:")) {
      const key = value.slice("$runtime:".length);
      if (!Object.hasOwn(bindings, key)) {
        throw new Error(`unknown documentation runtime binding: ${key}`);
      }
      return bindings[key];
    }
    if (Array.isArray(value)) {
      return value.map(bind);
    }
    if (value && typeof value === "object") {
      return Object.fromEntries(
        Object.entries(value).map(([key, child]) => [key, bind(child)])
      );
    }
    return value;
  };
  return bind(input) as Record<string, unknown>;
};

const markdownLink = /\[[^\]]+\]\(([^)]+)\)/g;
const externalLink = /^(?:https?:|#)/;

export const assertDocumentationLinks = (
  path: string,
  text = readFileSync(path, "utf8")
) => {
  let checked = 0;
  for (const match of text.matchAll(markdownLink)) {
    const [, href] = match;
    if (!href || externalLink.test(href)) {
      continue;
    }
    const target = resolve(dirname(path), href.split("#")[0] ?? "");
    const within = relative(documentationRoot, target);
    if (
      within.startsWith("..") ||
      !existsSync(target) ||
      !statSync(target).isFile()
    ) {
      throw new Error(`missing or external documentation link: ${href}`);
    }
    checked += 1;
  }
  if (!checked) {
    throw new Error("documentation has no canonical local links");
  }
  return checked;
};
