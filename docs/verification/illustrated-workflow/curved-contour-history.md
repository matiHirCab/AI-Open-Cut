# Historical curved contour input

This original test-authored input is retained as historical evidence. It does not
satisfy replacement fixture acceptance. Localized by origin (560,140), its bounds
were 800×800; the complete sticker inventory contained 80 path commands.

Original target builder SHA-256: `35ce339be033bc9c1f9e942bb006903f4678962bba0ffe7fed8c8cbf4bb6367e`.
See [curved-observations.json](curved-observations.json) and raw source snapshots
at `/workspace/issue191-platform-seventh-input-snapshot`.

```typescript
    const contour = [
      { to: { x: 640, y: 140 }, type: "moveTo" },
      { to: { x: 1280, y: 140 }, type: "lineTo" },
      {
        control1: { x: 1360, y: 140 },
        control2: { x: 1360, y: 160 },
        to: { x: 1360, y: 220 },
        type: "cubicTo",
      },
      { to: { x: 1360, y: 860 }, type: "lineTo" },
      {
        control1: { x: 1360, y: 940 },
        control2: { x: 1340, y: 940 },
        to: { x: 1280, y: 940 },
        type: "cubicTo",
      },
      { to: { x: 640, y: 940 }, type: "lineTo" },
      {
        control1: { x: 560, y: 940 },
        control2: { x: 560, y: 920 },
        to: { x: 560, y: 860 },
        type: "cubicTo",
      },
      { to: { x: 560, y: 220 }, type: "lineTo" },
      {
        control1: { x: 560, y: 140 },
        control2: { x: 580, y: 140 },
        to: { x: 640, y: 140 },
        type: "cubicTo",
      },
      { type: "close" },
    ];
```
