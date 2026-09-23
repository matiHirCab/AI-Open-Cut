// Deterministic source for the reviewed static rules-screen edit recipe.
// Run: bun run scripts/generate-rules-screen-recipe.ts
import { writeFileSync } from "node:fs";

const destination = new URL(
  "../crates/editor-core/tests/fixtures/rules-screen/recipe.json",
  import.meta.url
);
const DURATION = 1_000;
const asColor = (hex: string, a = 1) => ({
  r: Number.parseInt(hex.slice(1, 3), 16) / 255,
  g: Number.parseInt(hex.slice(3, 5), 16) / 255,
  b: Number.parseInt(hex.slice(5, 7), 16) / 255,
  a,
});
const paint = (hex: string, a = 1) => ({ type: "solid", color: asColor(hex, a) });
const stroke = (hex: string, width: number, a = 1) => ({
  paint: paint(hex, a), width, dash: [], dashOffset: 0,
  lineCap: "butt", lineJoin: "miter", miterLimit: 4,
});
const transform2d = (x: number, y: number) => ({
  position: { x, y, unit: "pixels" }, anchor: { x: 0, y: 0 },
  scaleX: 1, scaleY: 1, rotationDeg: 0, skewXDeg: 0, skewYDeg: 0, opacity: 1,
});
const operations: Record<string, unknown>[] = [];
const shape = (
  alias: string, geometry: object, x: number, y: number,
  fill: object | null, edge: object | null
) => operations.push({
  operation: "add_shape", trackId: "$overlay", startMs: 0,
  durationMs: DURATION, resultAlias: alias, geometry, fill,
  stroke: edge, transform2d: transform2d(x, y),
});

// Full-frame native vector background and a subdued diagonal grid.
shape("background", { type: "rectangle", width: 1920, height: 1080 }, 0, 0, paint("#101925"), null);
operations.push({
  operation: "add_grid", trackId: "$overlay", startMs: 0, durationMs: DURATION,
  resultAlias: "grid", transform2d: transform2d(0, 0),
  grid: { width: 1920, height: 1080, pattern: {
    type: "diagonal", spacing: 72, stroke: stroke("#6E8DA6", 2, 0.2),
  } },
});

// Three outlined cards with a vertical accent and visible corner brackets.
const cardX = [96, 710, 1324];
const bracketCommands: object[] = [];
for (const [index, x] of cardX.entries()) {
  const n = index + 1;
  shape(`card${n}`, { type: "roundedRectangle", width: 500, height: 270,
    radii: { topLeft: 18, topRight: 18, bottomRight: 18, bottomLeft: 18 } },
  x, 84, paint("#192B3C", 0.84), stroke("#DCE9F2", 5));
  shape(`accent${n}`, { type: "rectangle", width: 14, height: 188 },
    x + 30, 126, paint(index === 1 ? "#F6BB5C" : "#46D4CC"), null);
  for (const [dx, dy, sx, sy] of [
    [-16, -16, 1, 1], [516, -16, -1, 1],
    [-16, 286, 1, -1], [516, 286, -1, -1],
  ] as const) {
    // Path commands are local to the crop origin; Transform2D places that origin on canvas.
    const point = { x: x + dx - 80, y: 84 + dy - 68 };
    bracketCommands.push(
      { type: "moveTo", to: point },
      { type: "lineTo", to: { x: point.x + 42 * sx, y: point.y } },
      { type: "moveTo", to: point },
      { type: "lineTo", to: { x: point.x, y: point.y + 42 * sy } },
    );
  }
}
shape("brackets", { type: "path", path: { fillRule: "nonzero", commands: bracketCommands } },
  80, 68, null, stroke("#46D4CC", 6));

// Circular outlines sit behind the three-word impact stack.
for (const [index, diameter] of [440, 320, 200].entries()) {
  shape(`circle${index + 1}`, { type: "ellipse", width: diameter, height: diameter },
    1450 + (440 - diameter) / 2, 545 + (440 - diameter) / 2,
    null, stroke(index === 0 ? "#46D4CC" : "#DCE9F2", 4, 0.7));
}
for (const [index, word] of ["EVERY.", "SINGLE.", "ONE."].entries()) {
  const y = 475 + index * 170;
  for (const [layer, dx, dy, color] of [
    ["shadow", 10, 9, "#235767"],
    ["face", 0, 0, "#FFFFFF"],
  ] as const) {
    operations.push({
      operation: "add_text", trackId: "$overlay", startMs: 0,
      durationMs: DURATION, resultAlias: `word${index + 1}_${layer}`,
      text: word, fontSize: 128, color, fontFamily: null, fontPath: null,
      transform: { positionX: 105 + dx, positionY: y + dy, scale: 1, opacity: 1 },
    });
  }
}
// Alias resolution is exercised inside the same atomic batch.
operations.push({ operation: "update_item", itemId: "@word1_face", fontSize: 132 });
operations.push({
  operation: "add_media", trackId: "$audio", assetId: "$tone",
  startMs: 0, durationMs: DURATION, sourceInMs: 0, resultAlias: "tone",
});

const recipe = {
  schemaVersion: 1,
  canvas: { width: 1920, height: 1080, fps: 10 },
  durationMs: DURATION,
  timestampsMs: [0, 500, 900],
  fontSha256: "ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280",
  syntheticAudio: { sampleRateHz: 48000, frequencyHz: 440, channels: 1 },
  operations,
};
writeFileSync(destination, `${JSON.stringify(recipe, null, 2)}\n`);
