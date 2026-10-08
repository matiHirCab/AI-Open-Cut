import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import contract from "../../../contracts/speech-alignment-markers-v1.json";
import {
  headlessEditSchema,
  schemas,
  speechMarkerPolicySchema,
} from "../src/schemas";
import {
  removeAudioBusHeadlessAdditions,
  removeAudioBusOwnershipAddition,
} from "./fixtures/audio-buses-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";
import pins from "./fixtures/speech-markers-predecessor-pins.json";
import { projectSpeechMarkerMcpPredecessor } from "./fixtures/speech-markers-projection";

it("keeps every canonical closed policy exact and rejects structural negatives", () => {
  for (const policy of contract.policies) {
    expect(speechMarkerPolicySchema.parse(policy)).toEqual(policy);
    expect(
      headlessEditSchema.parse({
        alignment: contract.alignment,
        assetId: "asset",
        markerPolicy: policy,
        operation: contract.operation,
        scope: "root",
        startMs: 1000,
      })
    ).toMatchObject({ markerPolicy: policy });
  }
  for (const policy of contract.invalidPolicies) {
    expect(speechMarkerPolicySchema.safeParse(policy).success).toBe(false);
  }
  const { operation, ...input } = headless.requests.speechMarkersGenerate.edit;
  expect(operation).toBe(contract.operation);
  expect(
    schemas.speechMarkersGenerate.parse({
      expectedRevision: 1,
      projectId: "project",
      ...input,
    })
  ).toMatchObject({ markerPolicy: contract.policies[2] });
  expect(
    headlessEditSchema.safeParse({
      ...headless.requests.speechMarkersGenerate.edit,
      alignment: null,
    }).success
  ).toBe(false);
  expect(
    schemas.speechCommitPreview.safeParse({
      expectedRevision: 1,
      placement: {
        itemId: "item",
        markerPolicy: { type: "none" },
        type: "replace",
      },
      projectId: "project",
      token: "token",
    }).success
  ).toBe(false);
});

it("preserves the exact frozen issue61 catalog raw bytes and additive projections", () => {
  for (const [name, pin] of Object.entries(pins.catalogs)) {
    const raw = readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures",
        `speech-markers-${name.replace(".json", "")}-predecessor.raw`
      )
    );
    expect(createHash("sha256").update(raw).digest("hex")).toBe(pin.rawSha256);
    const prior: unknown = JSON.parse(raw.toString());
    if (name === "headless-protocol-v1.json") {
      const current = removeAudioBusHeadlessAdditions(headless);
      expect(current.requests.speechMarkersGenerate.edit).toEqual({
        alignment: contract.alignment,
        assetId: "asset-1",
        markerPolicy: contract.policies[2],
        operation: contract.operation,
        scope: "root",
        startMs: 1000,
      });
      Reflect.deleteProperty(current.requests, "speechMarkersGenerate");
      expect(
        current.status.editorCapabilities.filter(
          (value) => value === contract.capability
        )
      ).toHaveLength(1);
      current.status.editorCapabilities =
        current.status.editorCapabilities.filter(
          (value) => value !== contract.capability
        );
      expect(current).toEqual(prior);
    } else {
      const current = removeAudioBusOwnershipAddition(ownership);
      expect(current.categories.speechAlignmentMarkers.canonical).toBe(
        "contracts/speech-alignment-markers-v1.json"
      );
      Reflect.deleteProperty(current.categories, "speechAlignmentMarkers");
      expect(current).toEqual(prior);
    }
  }
  const prior = projectSpeechMarkerMcpPredecessor(mcp);
  expect(
    createHash("sha256")
      .update(JSON.stringify(expandMcpSurfaceCatalog(prior as typeof mcp)))
      .digest("hex")
  ).toBe(pins.mcpExpandedSha256);
  expect(
    createHash("sha256")
      .update(
        readFileSync(
          resolve(
            import.meta.dirname,
            "fixtures/speech-markers-mcp-surface-v1-predecessor.raw"
          )
        )
      )
      .digest("hex")
  ).toBe(pins.mcpRawSha256);
  expect(() =>
    projectSpeechMarkerMcpPredecessor({ ...mcp, unrelated: true })
  ).toThrow("Unrelated");
  const changed = structuredClone(mcp);
  changed.toolDefinitions.speech_markers_generate.inputSchema.properties.startMs.minimum = 1;
  expect(() => projectSpeechMarkerMcpPredecessor(changed)).toThrow("Incorrect");
});
