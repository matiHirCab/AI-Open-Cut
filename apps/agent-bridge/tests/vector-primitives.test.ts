import { describe, expect, it } from "vitest";
import { z } from "zod/v4";
import CATALOG from "../../../contracts/vector-primitives-v1.json";
import {
  cornerRadiiSchema,
  gradientStopSchema,
  paintSchema,
  pathCommandSchema,
  strokeSchema,
  vectorColorSchema,
  vectorPathSchema,
  vectorPointSchema,
} from "../src/vector-primitives";

const catalogSchema = z
  .strictObject({
    activation: z.literal("core_primitives_only"),
    fixtures: z.array(
      z.strictObject({
        id: z.string().min(1),
        kind: z.enum([
          "color",
          "point",
          "paint",
          "stroke",
          "cornerRadii",
          "path",
          "gradientStop",
          "lineCap",
          "lineJoin",
          "fillRule",
          "pathCommand",
        ]),
        structuralInvalid: z.boolean().optional(),
        valid: z.boolean(),
        value: z.unknown().refine((value) => value !== undefined),
      })
    ),
    identifiers: z.strictObject({
      command: z.tuple([
        z.literal("moveTo"),
        z.literal("lineTo"),
        z.literal("quadraticTo"),
        z.literal("cubicTo"),
        z.literal("close"),
      ]),
      fillRule: z.tuple([z.literal("nonzero"), z.literal("evenodd")]),
      lineCap: z.tuple([
        z.literal("butt"),
        z.literal("round"),
        z.literal("square"),
      ]),
      lineJoin: z.tuple([
        z.literal("miter"),
        z.literal("round"),
        z.literal("bevel"),
      ]),
      paint: z.tuple([
        z.literal("solid"),
        z.literal("linearGradient"),
        z.literal("radialGradient"),
      ]),
    }),
    limits: z.strictObject({
      maxCoordinate: z.literal(1_000_000),
      maxDashEntries: z.literal(64),
      maxDimension: z.literal(16_384),
      maxGradientStops: z.literal(64),
      maxMiterLimit: z.literal(1000),
      maxPathCommands: z.literal(4096),
    }),
    version: z.literal(1),
  })
  .refine(
    (catalog) =>
      new Set(catalog.fixtures.map((fixture) => fixture.id)).size ===
        catalog.fixtures.length &&
      catalog.fixtures.every(
        (fixture) => !(fixture.structuralInvalid && fixture.valid)
      )
  );
const schemas = {
  color: vectorColorSchema,
  cornerRadii: cornerRadiiSchema,
  fillRule: vectorPathSchema.shape.fillRule,
  gradientStop: gradientStopSchema,
  lineCap: strokeSchema.shape.lineCap,
  lineJoin: strokeSchema.shape.lineJoin,
  paint: paintSchema,
  path: vectorPathSchema,
  pathCommand: pathCommandSchema,
  point: vectorPointSchema,
  stroke: strokeSchema,
};

describe("canonical vector primitives", () => {
  for (const fixture of catalogSchema.parse(CATALOG).fixtures) {
    it(fixture.id, () => {
      const parsed = schemas[fixture.kind].safeParse(fixture.value);
      if (fixture.structuralInvalid) {
        expect(fixture.valid).toBe(false);
        expect(parsed.success).toBe(false);
      }
      expect(parsed.success).toBe(fixture.valid);
      if (parsed.success) {
        expect(parsed.data).toEqual(fixture.value);
        expect(
          schemas[fixture.kind].parse(JSON.parse(JSON.stringify(parsed.data)))
        ).toEqual(parsed.data);
      }
    });
  }
  it("rejects malformed catalog wrappers and identifier drift", () => {
    expect(catalogSchema.safeParse({ ...CATALOG, extra: true }).success).toBe(
      false
    );
    for (const key of Object.keys(CATALOG)) {
      const value: Record<string, unknown> = { ...CATALOG };
      delete value[key];
      expect(catalogSchema.safeParse(value).success).toBe(false);
    }
    expect(
      catalogSchema.safeParse({
        ...CATALOG,
        fixtures: [...CATALOG.fixtures, CATALOG.fixtures[0]],
      }).success
    ).toBe(false);
    expect(
      catalogSchema.safeParse({
        ...CATALOG,
        limits: { ...CATALOG.limits, maxPathCommands: 4097 },
      }).success
    ).toBe(false);
    expect(
      catalogSchema.safeParse({
        ...CATALOG,
        identifiers: { ...CATALOG.identifiers, command: ["arcTo"] },
      }).success
    ).toBe(false);
    expect(
      catalogSchema.safeParse({
        ...CATALOG,
        fixtures: [{ ...CATALOG.fixtures[0], extra: 1 }],
      }).success
    ).toBe(false);
  });
  it("rejects positional catalog envelopes and object-form fixture kinds", () => {
    const fixture = {
      id: "record",
      kind: "color",
      valid: true,
      value: { a: 1, b: 0, g: 0, r: 1 },
    };
    const fixtureList = [
      ["record", "color", fixture.value, true],
      ["record", "color", fixture.value, true, false],
      { ...fixture, kind: { color: null } },
      { ...fixture, structuralInvalid: true },
    ];
    for (const value of fixtureList) {
      expect(
        catalogSchema.safeParse({ ...CATALOG, fixtures: [value] }).success
      ).toBe(false);
    }
    expect(
      catalogSchema.safeParse([
        CATALOG.version,
        CATALOG.activation,
        CATALOG.limits,
        CATALOG.identifiers,
        CATALOG.fixtures,
      ]).success
    ).toBe(false);
    for (const key of ["id", "kind", "value", "valid"]) {
      const value: Record<string, unknown> = { ...fixture };
      delete value[key];
      expect(
        catalogSchema.safeParse({ ...CATALOG, fixtures: [value] }).success
      ).toBe(false);
    }
  });
  it("enforces path command count inclusively", () => {
    const commands = Array.from({ length: 4096 }, () => ({
      to: { x: 0, y: 0 },
      type: "moveTo",
    }));
    expect(
      vectorPathSchema.safeParse({ commands, fillRule: "nonzero" }).success
    ).toBe(true);
    expect(
      vectorPathSchema.safeParse({
        commands: [...commands, commands[0]],
        fillRule: "nonzero",
      }).success
    ).toBe(false);
  });
  it("rejects nonfinite values at every numeric leaf", () => {
    function variants(value: unknown): unknown[] {
      if (typeof value === "number") {
        return [Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY];
      }
      if (Array.isArray(value)) {
        return value.flatMap((child, i) =>
          variants(child).map((replacement) =>
            value.map((v, j) => (i === j ? replacement : v))
          )
        );
      }
      if (value !== null && typeof value === "object") {
        return Object.entries(value).flatMap(([key, child]) =>
          variants(child).map((replacement) => ({
            ...value,
            [key]: replacement,
          }))
        );
      }
      return [];
    }
    for (const fixture of catalogSchema
      .parse(CATALOG)
      .fixtures.filter((f) => f.valid)) {
      for (const value of variants(fixture.value)) {
        expect(schemas[fixture.kind].safeParse(value).success, fixture.id).toBe(
          false
        );
      }
    }
  });
});
