import { z } from "zod/v4";

const ID = /^[A-Za-z][A-Za-z0-9_-]{0,127}$/;
const SCOPE = /^(?:project|root|component:[A-Za-z][A-Za-z0-9_-]{0,127})$/;
const COLOR = /^#[0-9A-Fa-f]{8}$/;
const EVENT_HANDLER = /\son[a-z]+\s*=/;
const SAFE_INTEGER = z.number().int().min(0).max(Number.MAX_SAFE_INTEGER);
const SIGNED_SAFE_INTEGER = z
  .number()
  .int()
  .min(Number.MIN_SAFE_INTEGER)
  .max(Number.MAX_SAFE_INTEGER);
const ID_SCHEMA = z.string().regex(ID);
const SCOPE_SCHEMA = z.string().regex(SCOPE);
const COMPOSITION_SCOPE_SCHEMA = SCOPE_SCHEMA.refine(
  (scope) => scope === "root" || scope.startsWith("component:"),
  "composition scope required"
);
const COMPONENT_SCOPE_SCHEMA = SCOPE_SCHEMA.refine(
  (scope) => scope.startsWith("component:"),
  "component scope required"
);

const REFERENCE_KINDS = [
  "asset",
  "audio_bus",
  "audio_event",
  "component",
  "curve",
  "effect",
  "layer",
  "marker",
  "mask",
  "slot",
  "sound_definition",
  "transform",
] as const;

const referenceSchema = z
  .strictObject({
    id: ID_SCHEMA,
    kind: z.enum(REFERENCE_KINDS),
    scope: SCOPE_SCHEMA,
  })
  .superRefine((value, context) => {
    const compositionScoped = [
      "audio_event",
      "curve",
      "effect",
      "layer",
      "marker",
      "mask",
      "transform",
    ].includes(value.kind);
    const legal =
      (["asset", "audio_bus", "component", "sound_definition"].includes(
        value.kind
      ) &&
        value.scope === "project") ||
      (value.kind === "slot" && value.scope.startsWith("component:")) ||
      (compositionScoped &&
        (value.scope === "root" || value.scope.startsWith("component:")));
    if (!legal) {
      context.addIssue({ code: "custom", message: "illegal kind/scope tuple" });
    }
  });

type Reference = z.infer<typeof referenceSchema>;

const managedAssetSchema = z.strictObject({
  id: ID_SCHEMA,
  kind: z.literal("asset"),
  scope: z.literal("project"),
});

const concepts = [
  "transform",
  "layer",
  "component",
  "slot",
  "marker",
  "curve",
  "mask",
  "effect",
  "audio_event",
] as const;

const limitsSchema = z.strictObject({
  maxAudioEventsPerComposition: SAFE_INTEGER.positive(),
  maxComponentDefinitions: SAFE_INTEGER.positive(),
  maxComponentDepth: SAFE_INTEGER.positive(),
  maxEffectsPerLayer: SAFE_INTEGER.positive(),
  maxKeyframesPerChannel: SAFE_INTEGER.positive(),
  maxLayersPerComposition: SAFE_INTEGER.positive(),
  maxMarkersPerComposition: SAFE_INTEGER.positive(),
  maxMasksPerLayer: SAFE_INTEGER.positive(),
  maxParentDepth: SAFE_INTEGER.positive(),
  maxSlotsPerComponent: SAFE_INTEGER.positive(),
});

const catalogSchema = z.strictObject({
  contract: z.literal("motion-graphics-v1"),
  identifiers: z.strictObject({
    blendModes: z.tuple([
      z.literal("normal"),
      z.literal("multiply"),
      z.literal("screen"),
      z.literal("overlay"),
      z.literal("add"),
      z.literal("darken"),
      z.literal("lighten"),
    ]),
    concepts: z.tuple(
      concepts.map((concept) => z.literal(concept)) as [
        z.ZodLiteral<"transform">,
        z.ZodLiteral<"layer">,
        z.ZodLiteral<"component">,
        z.ZodLiteral<"slot">,
        z.ZodLiteral<"marker">,
        z.ZodLiteral<"curve">,
        z.ZodLiteral<"mask">,
        z.ZodLiteral<"effect">,
        z.ZodLiteral<"audio_event">,
      ]
    ),
    curveTypes: z.tuple([
      z.literal("hold"),
      z.literal("linear"),
      z.literal("cubic_bezier"),
      z.literal("spring"),
    ]),
    effectTypes: z.tuple([
      z.literal("gaussian_blur"),
      z.literal("directional_blur"),
      z.literal("glow"),
      z.literal("color_tint"),
      z.literal("vignette"),
      z.literal("color_adjustment"),
      z.literal("screen_flash"),
      z.literal("particle_overlay"),
    ]),
    failureClassifications: z.tuple([
      z.literal("invalid_input"),
      z.literal("missing_reference"),
      z.literal("reference_cycle"),
      z.literal("ambiguous_reference"),
    ]),
    maskChannels: z.tuple([z.literal("alpha"), z.literal("luma")]),
    maskOperations: z.tuple([
      z.literal("add"),
      z.literal("subtract"),
      z.literal("intersect"),
      z.literal("exclude"),
    ]),
    maskSourceTypes: z.tuple([z.literal("path"), z.literal("layer")]),
    positionUnits: z.tuple([z.literal("pixels"), z.literal("normalized")]),
    referenceKinds: z.tuple(
      REFERENCE_KINDS.map((kind) => z.literal(kind)) as [
        z.ZodLiteral<"asset">,
        z.ZodLiteral<"audio_bus">,
        z.ZodLiteral<"audio_event">,
        z.ZodLiteral<"component">,
        z.ZodLiteral<"curve">,
        z.ZodLiteral<"effect">,
        z.ZodLiteral<"layer">,
        z.ZodLiteral<"marker">,
        z.ZodLiteral<"mask">,
        z.ZodLiteral<"slot">,
        z.ZodLiteral<"sound_definition">,
        z.ZodLiteral<"transform">,
      ]
    ),
    slotKinds: z.tuple([
      z.literal("text"),
      z.literal("color"),
      z.literal("number"),
      z.literal("boolean"),
      z.literal("asset"),
      z.literal("rich_text"),
      z.literal("enum"),
      z.literal("duration"),
    ]),
    timeExpressionTypes: z.tuple([
      z.literal("milliseconds"),
      z.literal("marker"),
    ]),
  }),
  invalidFixtures: z.array(
    z.strictObject({
      classification: z.enum([
        "invalid_input",
        "missing_reference",
        "reference_cycle",
        "ambiguous_reference",
      ]),
      concept: z.enum(concepts),
      id: z.string().min(1),
      reason: z.string().min(1),
      value: z.unknown(),
    })
  ),
  limits: limitsSchema,
  managedResources: z.array(managedAssetSchema),
  semantics: z.strictObject({
    alphaMode: z.literal("premultiplied"),
    compositingLight: z.literal("linear"),
    coordinateSystem: z.strictObject({
      origin: z.literal("top_left"),
      positionUnits: z.tuple([z.literal("pixels"), z.literal("normalized")]),
      positiveX: z.literal("right"),
      positiveY: z.literal("down"),
    }),
    layerOrdering: z.tuple([
      z.literal("track_array_index_ascending"),
      z.literal("z_index_ascending"),
      z.literal("item_array_index_ascending"),
      z.literal("item_id_ascending_final_tie_break"),
    ]),
    resourcePolicy: z.literal("managed_or_content_addressed_only"),
    time: z.strictObject({
      interval: z.literal("half_open"),
      unit: z.literal("integer_milliseconds"),
    }),
    variantCase: z.literal("lower_snake_case"),
    visualPipeline: z.tuple([
      z.literal("source"),
      z.literal("crop_and_local_clip"),
      z.literal("masks_in_declared_order"),
      z.literal("effects_in_declared_order"),
      z.literal("local_anchor_scale_skew_rotation_position"),
      z.literal("ancestor_transforms_nearest_first"),
      z.literal("track_matte"),
      z.literal("inherited_opacity"),
      z.literal("destination_blend"),
    ]),
    wireFieldCase: z.literal("lower_camel_case"),
  }),
  status: z.literal("fixture_only"),
  validFixtures: z.array(
    z.strictObject({
      concept: z.enum(concepts),
      defines: z.array(referenceSchema),
      id: z.string().min(1),
      references: z.array(referenceSchema),
      value: z.unknown(),
    })
  ),
  version: z.literal(1),
});

const positionSchema = z.strictObject({
  unit: z.enum(["pixels", "normalized"]),
  x: z.number().finite(),
  y: z.number().finite(),
});

const transformSchema = z.strictObject({
  anchor: z.strictObject({
    x: z.number().min(0).max(1),
    y: z.number().min(0).max(1),
  }),
  id: ID_SCHEMA,
  opacity: z.number().min(0).max(1),
  position: positionSchema,
  rotationDeg: z.number().finite(),
  scaleX: z.number().positive().finite(),
  scaleY: z.number().positive().finite(),
  scope: SCOPE_SCHEMA,
  skewXDeg: z.number().finite(),
  skewYDeg: z.number().finite(),
});

const layerSchema = z.strictObject({
  animationChannels: z.array(z.string().min(1)),
  blendMode: z.enum([
    "normal",
    "multiply",
    "screen",
    "overlay",
    "add",
    "darken",
    "lighten",
  ]),
  clip: z.strictObject({ type: z.literal("composition_bounds") }).nullable(),
  effects: z.array(ID_SCHEMA),
  hidden: z.boolean(),
  id: ID_SCHEMA,
  masks: z.array(ID_SCHEMA),
  parentId: ID_SCHEMA.nullable(),
  stableItemIndex: SAFE_INTEGER,
  transformId: ID_SCHEMA.nullable(),
  zIndex: z.number().int().min(-2_147_483_648).max(2_147_483_647),
});

const layerSetSchema = z.strictObject({
  layers: z.array(layerSchema).min(1),
  scope: COMPOSITION_SCOPE_SCHEMA,
});

const componentSchema = z.strictObject({
  definition: z.strictObject({
    durationMs: SAFE_INTEGER.positive(),
    height: z.number().int().positive().max(16_384),
    id: ID_SCHEMA,
    layers: z.array(ID_SCHEMA).min(1),
    markerIds: z.array(ID_SCHEMA),
    name: z
      .string()
      .min(1)
      .refine((value) => Array.from(value).length <= 200),
    slotIds: z.array(ID_SCHEMA),
    trackIds: z.array(ID_SCHEMA).min(1),
    width: z.number().int().positive().max(16_384),
  }),
  instance: z.strictObject({
    componentId: ID_SCHEMA,
    durationMs: SAFE_INTEGER.positive(),
    id: ID_SCHEMA,
    slotValues: z.record(ID_SCHEMA, z.string()),
    startMs: SAFE_INTEGER,
    timeScale: z.number().positive().finite(),
    trimStartMs: SAFE_INTEGER,
  }),
});

const slotSchema = z
  .strictObject({
    binding: z.strictObject({
      property: z.literal("text.document"),
      targetLayerId: ID_SCHEMA,
    }),
    constraints: z.strictObject({
      maxLength: z.number().int().positive().max(4096),
      minLength: z.number().int().min(0).max(4096),
    }),
    defaultValue: z.string(),
    id: ID_SCHEMA,
    kind: z.literal("text"),
    name: z
      .string()
      .min(1)
      .refine((value) => Array.from(value).length <= 200),
    required: z.boolean(),
    scope: COMPONENT_SCOPE_SCHEMA,
  })
  .refine(
    (slot) => slot.constraints.minLength <= slot.constraints.maxLength,
    "slot_constraint_violation"
  )
  .refine(
    (slot) =>
      Array.from(slot.defaultValue).length >= slot.constraints.minLength &&
      Array.from(slot.defaultValue).length <= slot.constraints.maxLength,
    "slot_default_constraint_violation"
  );

const timeExpressionSchema = z.discriminatedUnion("type", [
  z.strictObject({ type: z.literal("milliseconds"), valueMs: SAFE_INTEGER }),
  z.strictObject({
    markerName: ID_SCHEMA,
    offsetMs: SIGNED_SAFE_INTEGER,
    type: z.literal("marker"),
  }),
]);

const markerSchema = z.strictObject({
  absoluteTime: timeExpressionSchema,
  marker: z.strictObject({
    id: ID_SCHEMA,
    kind: z.literal("cue"),
    name: ID_SCHEMA,
    scope: COMPOSITION_SCOPE_SCHEMA,
    timeMs: SAFE_INTEGER,
  }),
  relativeTime: timeExpressionSchema,
});

const curveSchema = z.discriminatedUnion("type", [
  z.strictObject({ type: z.literal("hold") }),
  z.strictObject({ type: z.literal("linear") }),
  z.strictObject({
    type: z.literal("cubic_bezier"),
    x1: z.number().min(0).max(1),
    x2: z.number().min(0).max(1),
    y1: z.number().finite(),
    y2: z.number().finite(),
  }),
  z.strictObject({
    damping: z.number().positive().finite(),
    initialVelocity: z.number().finite(),
    mass: z.number().positive().finite(),
    stiffness: z.number().positive().finite(),
    type: z.literal("spring"),
  }),
]);

const curveSetSchema = z.strictObject({
  curves: z.array(curveSchema).min(1),
  id: ID_SCHEMA,
  keyframes: z.array(
    z.strictObject({
      curve: curveSchema,
      time: timeExpressionSchema,
      value: z.number().finite(),
    })
  ),
  scope: COMPOSITION_SCOPE_SCHEMA,
});

const pathCommandSchema = z.discriminatedUnion("type", [
  z.strictObject({ type: z.literal("move_to"), x: z.number(), y: z.number() }),
  z.strictObject({ type: z.literal("line_to"), x: z.number(), y: z.number() }),
  z.strictObject({ type: z.literal("close") }),
]);

const maskSchema = z.strictObject({
  layerId: ID_SCHEMA,
  masks: z.array(
    z.strictObject({
      channel: z.enum(["alpha", "luma"]),
      expansionPx: z.number().finite(),
      featherPx: z.number().min(0).finite(),
      id: ID_SCHEMA,
      inverted: z.boolean(),
      operation: z.enum(["add", "subtract", "intersect", "exclude"]),
      source: z.discriminatedUnion("type", [
        z.strictObject({
          commands: z.array(pathCommandSchema).min(1),
          type: z.literal("path"),
        }),
        z.strictObject({ layerId: ID_SCHEMA, type: z.literal("layer") }),
      ]),
      transformId: ID_SCHEMA,
    })
  ),
  scope: COMPOSITION_SCOPE_SCHEMA,
});

const effectSchema = z.strictObject({
  effects: z.array(
    z.discriminatedUnion("type", [
      z.strictObject({
        id: ID_SCHEMA,
        radiusPx: z.number().min(0).max(4096),
        type: z.literal("gaussian_blur"),
      }),
      z.strictObject({
        color: z.string().regex(COLOR),
        id: ID_SCHEMA,
        intensity: z.number().min(0).max(100),
        radiusPx: z.number().min(0).max(4096),
        type: z.literal("glow"),
      }),
    ])
  ),
  layerId: ID_SCHEMA,
  scope: COMPOSITION_SCOPE_SCHEMA,
});

const audioSchema = z.strictObject({
  bus: z.strictObject({ id: ID_SCHEMA }),
  event: z.strictObject({
    at: timeExpressionSchema,
    busId: ID_SCHEMA,
    event: ID_SCHEMA,
    gainDb: z.number().min(-120).max(24),
    id: ID_SCHEMA,
    scope: COMPOSITION_SCOPE_SCHEMA,
    variantSeed: SAFE_INTEGER,
  }),
  soundDefinition: z.strictObject({
    busId: ID_SCHEMA,
    defaultGainDb: z.number().min(-120).max(24),
    event: ID_SCHEMA,
    variantAssetIds: z.array(ID_SCHEMA).min(1),
  }),
});

const dependencyScenarioSchema = z.strictObject({
  componentIds: z.array(ID_SCHEMA).min(1),
  dependencies: z.array(z.strictObject({ from: ID_SCHEMA, to: ID_SCHEMA })),
  entryId: ID_SCHEMA,
});

const taggedSlotValueSchema = z.discriminatedUnion("type", [
  z.strictObject({ type: z.literal("text"), value: z.string() }),
  z.strictObject({ type: z.literal("number"), value: z.number().finite() }),
]);

const slotScenarioSchema = z.strictObject({
  definition: z
    .strictObject({
      binding: z.strictObject({
        property: z.string().min(1),
        targetLayerId: ID_SCHEMA,
      }),
      constraints: z.strictObject({
        maxLength: SAFE_INTEGER.positive().max(4096),
        minLength: SAFE_INTEGER.max(4096),
      }),
      defaultValue: taggedSlotValueSchema,
      id: ID_SCHEMA,
      kind: z.literal("text"),
      name: z
        .string()
        .min(1)
        .refine((value) => scalarLength(value) <= 200),
      required: z.boolean(),
      scope: COMPONENT_SCOPE_SCHEMA,
    })
    .refine(
      (definition) =>
        definition.constraints.minLength <= definition.constraints.maxLength,
      "slot minimum must not exceed maximum"
    ),
  suppliedValue: taggedSlotValueSchema.optional(),
  targetLayerIds: z.array(ID_SCHEMA).min(1),
});

type SlotScenario = z.infer<typeof slotScenarioSchema>;

const markerCandidateSchema = z.strictObject({
  id: ID_SCHEMA,
  kind: z.literal("cue"),
  name: ID_SCHEMA,
  scope: COMPOSITION_SCOPE_SCHEMA,
  timeMs: SAFE_INTEGER,
});

const markerScenarioSchema = z.strictObject({
  lookupName: ID_SCHEMA,
  markers: z.array(markerCandidateSchema).min(1),
  scope: COMPOSITION_SCOPE_SCHEMA,
});

const audioScenarioSchema = z.strictObject({
  assets: z.array(ID_SCHEMA).min(1),
  buses: z.array(z.strictObject({ id: ID_SCHEMA })).min(1),
  events: z
    .array(
      z.strictObject({
        at: timeExpressionSchema,
        busId: ID_SCHEMA,
        event: ID_SCHEMA,
        gainDb: z.number().min(-120).max(24),
        id: ID_SCHEMA,
        scope: COMPOSITION_SCOPE_SCHEMA,
        variantSeed: SAFE_INTEGER,
      })
    )
    .min(1),
  markers: z.array(markerCandidateSchema).min(1),
  soundDefinitions: z
    .array(
      z.strictObject({
        busId: ID_SCHEMA,
        defaultGainDb: z.number().min(-120).max(24),
        event: ID_SCHEMA,
        variantAssetIds: z.array(z.string().min(1)).min(1),
      })
    )
    .min(1),
});

type FixtureFailure = readonly [
  (
    | "ambiguous_reference"
    | "invalid_input"
    | "missing_reference"
    | "reference_cycle"
  ),
  string,
];

const scalarLength = (value: string) => Array.from(value).length;

const ensureAtMost = (count: number, limit: number, label: string) => {
  if (count > limit) {
    throw new Error(`${label} exceeded`);
  }
};

const duplicateValue = (values: string[], target: string) =>
  values.filter((value) => value === target).length > 1;

const assertUniqueIds = (ids: string[], label: string) => {
  if (new Set(ids).size !== ids.length) {
    throw new Error(`${label} duplicate payload definition`);
  }
};

const duplicateKeys = (values: string[]) => {
  const counts = new Map<string, number>();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return [...counts.entries()]
    .filter(([, count]) => count > 1)
    .map(([value]) => value);
};

const assertOnlyDeclaredDuplicateKey = (
  duplicates: string[],
  allowedKeys: Set<string>,
  label: string
) => {
  if (duplicates.length === 0) {
    throw new Error(`${label} declared ambiguity key is not duplicated`);
  }
  if (duplicates.length !== 1 || !allowedKeys.has(duplicates[0] ?? "")) {
    throw new Error(
      `${label} duplicate payload definition outside declared ambiguity`
    );
  }
};

const assertSingleDeclaredDuplicate = (
  values: string[],
  allowedKeys: Set<string>,
  label: string
) => assertOnlyDeclaredDuplicateKey(duplicateKeys(values), allowedKeys, label);

const ensureAtMostPerOwner = <T>(
  values: T[],
  owner: (value: T) => string,
  limit: number,
  label: string
) => {
  const counts = new Map<string, number>();
  for (const value of values) {
    const key = owner(value);
    const count = (counts.get(key) ?? 0) + 1;
    counts.set(key, count);
    ensureAtMost(count, limit, label);
  }
};

type Limits = z.infer<typeof limitsSchema>;

const componentFailure = (value: unknown, limits: Limits): FixtureFailure => {
  const scenario = dependencyScenarioSchema.parse(value);
  assertUniqueIds(scenario.componentIds, "component invalid envelope");
  ensureAtMost(
    scenario.componentIds.length,
    limits.maxComponentDefinitions,
    "maxComponentDefinitions"
  );
  assertUniqueIds(
    scenario.dependencies.map((edge) => `${edge.from}\u0000${edge.to}`),
    "component invalid envelope dependency edge"
  );
  const ids = new Set(scenario.componentIds);
  if (!ids.has(scenario.entryId)) {
    return ["missing_reference", "component_not_found"];
  }
  const edges = new Map<string, string[]>();
  for (const edge of scenario.dependencies) {
    if (!(ids.has(edge.from) && ids.has(edge.to))) {
      return ["missing_reference", "component_not_found"];
    }
    const targets = edges.get(edge.from) ?? [];
    targets.push(edge.to);
    edges.set(edge.from, targets);
  }

  const active = new Set<string>();
  const memoizedDepth = new Map<string, number>();
  const visit = (componentId: string): number | "cycle" => {
    if (active.has(componentId)) {
      return "cycle";
    }
    const memoized = memoizedDepth.get(componentId);
    if (memoized !== undefined) {
      return memoized;
    }
    active.add(componentId);
    let longest = 1;
    for (const target of edges.get(componentId) ?? []) {
      const childDepth = visit(target);
      if (childDepth === "cycle") {
        return "cycle";
      }
      longest = Math.max(longest, childDepth + 1);
    }
    active.delete(componentId);
    memoizedDepth.set(componentId, longest);
    return longest;
  };
  const depth = visit(scenario.entryId);
  if (depth === "cycle") {
    return ["reference_cycle", "component_cycle"];
  }
  if (depth > limits.maxComponentDepth) {
    return ["invalid_input", "max_component_depth_exceeded"];
  }
  throw new Error("component candidate has no intended failure");
};

const crossScopeLayerFailure = (
  value: unknown,
  limits: Limits
): FixtureFailure => {
  const scenario = z
    .strictObject({
      layers: z.array(layerSchema.extend({ parentScope: SCOPE_SCHEMA })).min(1),
      scope: COMPOSITION_SCOPE_SCHEMA,
    })
    .parse(value);
  assertUniqueIds(
    scenario.layers.map((layer) => layer.id),
    "cross-scope layer invalid envelope"
  );
  ensureAtMost(
    scenario.layers.length,
    limits.maxLayersPerComposition,
    "maxLayersPerComposition"
  );
  if (scenario.layers.some((layer) => layer.parentScope !== scenario.scope)) {
    return ["invalid_input", "parent_scope_mismatch"];
  }
  throw new Error("cross-scope layer candidate has no intended failure");
};

const invalidLayerFailure = (
  id: string,
  value: unknown,
  limits: Limits
): FixtureFailure => {
  if (id === "layer.cross_scope_parent") {
    return crossScopeLayerFailure(value, limits);
  }
  const scenario = layerSetSchema.parse(value);
  assertUniqueIds(
    scenario.layers.map((layer) => layer.id),
    id
  );
  ensureAtMost(
    scenario.layers.length,
    limits.maxLayersPerComposition,
    "maxLayersPerComposition"
  );
  const parents = new Map(
    scenario.layers.map((layer) => [layer.id, layer.parentId] as const)
  );
  for (const layer of scenario.layers) {
    if (layer.parentId && !parents.has(layer.parentId)) {
      return ["missing_reference", "parent_not_found"];
    }
    if (layer.parentId === layer.id) {
      return ["reference_cycle", "direct_parent_cycle"];
    }
    let current = layer.parentId;
    const seen = new Set([layer.id]);
    while (current) {
      if (seen.has(current)) {
        return ["reference_cycle", "parent_cycle"];
      }
      seen.add(current);
      current = parents.get(current) ?? null;
    }
  }
  throw new Error(`${id} has no intended layer failure`);
};

const preflightSlotScenario = (id: string, scenario: SlotScenario) => {
  const { definition } = scenario;
  const defaultLength =
    definition.defaultValue.type === "text"
      ? scalarLength(definition.defaultValue.value)
      : undefined;
  if (
    definition.binding.property !== "text.document" &&
    id !== "slot.arbitrary_path"
  ) {
    throw new Error("unexpected unstable binding target");
  }
  if (
    definition.defaultValue.type !== "text" &&
    id !== "slot.invalid_default"
  ) {
    throw new Error("unexpected slot default type mismatch");
  }
  if (
    defaultLength !== undefined &&
    (defaultLength < definition.constraints.minLength ||
      defaultLength > definition.constraints.maxLength)
  ) {
    throw new Error("slot default constraint violation");
  }
  if (!scenario.suppliedValue && id !== "slot.required_value_missing") {
    throw new Error("unexpected missing slot value");
  }
  if (
    scenario.suppliedValue?.type === "number" &&
    id !== "slot.type_mismatch"
  ) {
    throw new Error("unexpected slot value type mismatch");
  }
  if (
    scenario.suppliedValue?.type === "text" &&
    id !== "slot.constraint_violation"
  ) {
    const suppliedLength = scalarLength(scenario.suppliedValue.value);
    if (
      suppliedLength < definition.constraints.minLength ||
      suppliedLength > definition.constraints.maxLength
    ) {
      throw new Error("unexpected slot value constraint violation");
    }
  }
};

const slotFailure = (
  id: string,
  value: unknown,
  limits: Limits
): FixtureFailure => {
  const scenario = slotScenarioSchema.parse(value);
  assertUniqueIds(
    scenario.targetLayerIds,
    "slot invalid envelope target layer"
  );
  ensureAtMost(
    scenario.targetLayerIds.length,
    limits.maxLayersPerComposition,
    "maxLayersPerComposition"
  );
  preflightSlotScenario(id, scenario);
  const { definition } = scenario;
  if (definition.binding.property !== "text.document") {
    return ["invalid_input", "unstable_binding_target"];
  }
  if (!scenario.targetLayerIds.includes(definition.binding.targetLayerId)) {
    return ["missing_reference", "slot_target_not_found"];
  }
  if (definition.defaultValue.type !== definition.kind) {
    return ["invalid_input", "slot_default_type_mismatch"];
  }
  if (!scenario.suppliedValue) {
    if (definition.required) {
      return ["invalid_input", "required_slot_value_missing"];
    }
    throw new Error("optional slot candidate has no value and no failure");
  }
  if (scenario.suppliedValue.type !== definition.kind) {
    return ["invalid_input", "slot_value_type_mismatch"];
  }
  const length = scalarLength(scenario.suppliedValue.value);
  if (
    length < definition.constraints.minLength ||
    length > definition.constraints.maxLength
  ) {
    return ["invalid_input", "slot_constraint_violation"];
  }
  throw new Error("slot candidate has no intended failure");
};

type AudioScenario = z.infer<typeof audioScenarioSchema>;

const audioResourceFailure = (
  scenario: AudioScenario
): FixtureFailure | undefined => {
  for (const definition of scenario.soundDefinitions) {
    for (const asset of definition.variantAssetIds) {
      if (!ID.test(asset)) {
        return ["invalid_input", "network_resource_forbidden"];
      }
      if (!scenario.assets.includes(asset)) {
        return ["missing_reference", "sound_variant_not_found"];
      }
      if (duplicateValue(definition.variantAssetIds, asset)) {
        return ["ambiguous_reference", "sound_variant_ambiguous"];
      }
    }
  }
  return undefined;
};

const audioEventFailure = (
  scenario: AudioScenario
): FixtureFailure | undefined => {
  for (const event of scenario.events) {
    if (
      duplicateValue(
        scenario.buses.map((bus) => bus.id),
        event.busId
      )
    ) {
      return ["ambiguous_reference", "audio_bus_ambiguous"];
    }
    if (!scenario.buses.some((bus) => bus.id === event.busId)) {
      return ["missing_reference", "audio_bus_not_found"];
    }
    if (
      duplicateValue(
        scenario.soundDefinitions.map((definition) => definition.event),
        event.event
      )
    ) {
      return ["ambiguous_reference", "sound_definition_ambiguous"];
    }
    if (
      !scenario.soundDefinitions.some(
        (definition) => definition.event === event.event
      )
    ) {
      return ["missing_reference", "sound_definition_not_found"];
    }
    if (event.at.type === "marker") {
      const names = scenario.markers
        .filter((marker) => marker.scope === event.scope)
        .map((marker) => marker.name);
      if (duplicateValue(names, event.at.markerName)) {
        return ["ambiguous_reference", "marker_ambiguous"];
      }
      if (!names.includes(event.at.markerName)) {
        return ["missing_reference", "marker_not_found"];
      }
    }
  }
  return undefined;
};

const audioFailure = (
  id: string,
  value: unknown,
  limits: Limits
): FixtureFailure => {
  const scenario = audioScenarioSchema.parse(value);
  assertUniqueIds(scenario.assets, "audio invalid envelope asset");
  assertUniqueIds(
    scenario.events.map((event) => `${event.scope}\u0000${event.id}`),
    "audio invalid envelope event"
  );
  assertUniqueIds(
    scenario.markers.map((marker) => `${marker.scope}\u0000${marker.id}`),
    "audio invalid envelope marker"
  );
  const busIds = scenario.buses.map((bus) => bus.id);
  if (id === "audio_event.ambiguous_bus") {
    assertSingleDeclaredDuplicate(
      busIds,
      new Set(scenario.events.map((event) => event.busId)),
      "audio invalid envelope bus"
    );
  } else {
    assertUniqueIds(busIds, "audio invalid envelope bus");
  }
  const declaredBusIds = new Set(busIds);
  if (
    scenario.soundDefinitions.some(
      (definition) => !declaredBusIds.has(definition.busId)
    )
  ) {
    throw new Error(
      "audio invalid envelope sound definition bus missing reference"
    );
  }
  const markerNames = scenario.markers.map(
    (marker) => `${marker.scope}\u0000${marker.name}`
  );
  if (id === "audio_event.ambiguous_marker") {
    assertSingleDeclaredDuplicate(
      markerNames,
      new Set(
        scenario.events.flatMap((event) =>
          event.at.type === "marker"
            ? [`${event.scope}\u0000${event.at.markerName}`]
            : []
        )
      ),
      "audio invalid envelope marker name"
    );
  } else {
    assertUniqueIds(markerNames, "audio invalid envelope marker name");
  }
  const soundEvents = scenario.soundDefinitions.map(
    (definition) => definition.event
  );
  if (id === "audio_event.ambiguous_sound_definition") {
    assertSingleDeclaredDuplicate(
      soundEvents,
      new Set(scenario.events.map((event) => event.event)),
      "audio invalid envelope sound definition"
    );
  } else {
    assertUniqueIds(soundEvents, "audio invalid envelope sound definition");
  }
  if (id === "audio_event.ambiguous_variant") {
    const duplicates = scenario.soundDefinitions.flatMap((definition) =>
      duplicateKeys(definition.variantAssetIds).map(
        (asset) => `${definition.event}\u0000${asset}`
      )
    );
    const referencedEvents = new Set(
      scenario.events.map((event) => event.event)
    );
    const allowed = new Set(
      scenario.soundDefinitions.flatMap((definition) =>
        referencedEvents.has(definition.event)
          ? definition.variantAssetIds.map(
              (asset) => `${definition.event}\u0000${asset}`
            )
          : []
      )
    );
    assertOnlyDeclaredDuplicateKey(
      duplicates,
      allowed,
      "audio invalid envelope sound variant"
    );
  } else {
    for (const definition of scenario.soundDefinitions) {
      assertUniqueIds(
        definition.variantAssetIds,
        "audio invalid envelope sound variant"
      );
    }
  }
  ensureAtMostPerOwner(
    scenario.events,
    (event) => event.scope,
    limits.maxAudioEventsPerComposition,
    "maxAudioEventsPerComposition"
  );
  ensureAtMostPerOwner(
    scenario.markers,
    (marker) => marker.scope,
    limits.maxMarkersPerComposition,
    "maxMarkersPerComposition"
  );
  for (const definition of scenario.soundDefinitions) {
    for (const asset of definition.variantAssetIds) {
      if (!ID.test(asset) && id !== "audio_event.network_variant") {
        throw new Error("unexpected invalid sound variant ID");
      }
    }
  }
  const failure = audioResourceFailure(scenario) ?? audioEventFailure(scenario);
  if (failure) {
    return failure;
  }
  throw new Error("audio candidate has no intended failure");
};

const invalidMaskSourceSchema = z.discriminatedUnion("type", [
  z.strictObject({
    commands: z.array(pathCommandSchema).min(1),
    type: z.literal("path"),
  }),
  z.strictObject({ layerId: ID_SCHEMA, type: z.literal("layer") }),
  z.strictObject({ svg: z.string().min(1), type: z.literal("inline_svg") }),
  z.strictObject({ path: z.string().min(1), type: z.literal("file") }),
]);

const maskScenarioSchema = z.strictObject({
  availableLayerIds: z.array(ID_SCHEMA).min(1),
  layerId: ID_SCHEMA,
  masks: z
    .array(
      z.strictObject({
        channel: z.enum(["alpha", "luma"]),
        expansionPx: z.number().finite(),
        featherPx: z.number().min(0).finite(),
        id: ID_SCHEMA,
        inverted: z.boolean(),
        operation: z.enum(["add", "subtract", "intersect", "exclude"]),
        source: invalidMaskSourceSchema,
        transformId: ID_SCHEMA,
      })
    )
    .min(1),
  scope: COMPOSITION_SCOPE_SCHEMA,
});

const maskFailure = (value: unknown, limits: Limits): FixtureFailure => {
  const scenario = maskScenarioSchema.parse(value);
  assertUniqueIds(
    scenario.masks.map((mask) => mask.id),
    "mask invalid envelope"
  );
  assertUniqueIds(
    scenario.availableLayerIds,
    "mask invalid envelope available layer"
  );
  ensureAtMost(
    scenario.availableLayerIds.length,
    limits.maxLayersPerComposition,
    "maxLayersPerComposition"
  );
  ensureAtMost(
    scenario.masks.length,
    limits.maxMasksPerLayer,
    "maxMasksPerLayer"
  );
  for (const mask of scenario.masks) {
    if (mask.source.type === "inline_svg") {
      const svg = mask.source.svg.toLowerCase();
      if (svg.includes("<script") || EVENT_HANDLER.test(svg)) {
        return ["invalid_input", "executable_svg"];
      }
    }
    if (mask.source.type === "file") {
      return ["invalid_input", "arbitrary_path_forbidden"];
    }
    if (
      mask.source.type === "layer" &&
      !scenario.availableLayerIds.includes(mask.source.layerId)
    ) {
      return ["missing_reference", "mask_source_not_found"];
    }
  }
  throw new Error("mask candidate has no intended failure");
};

const nonFiniteTransformFailure = (value: unknown): FixtureFailure => {
  const candidate = structuredClone(asRecord(value, "transform.non_finite"));
  if (candidate.scaleX !== "NaN") {
    throw new Error(
      "transform candidate does not contain its non-finite token"
    );
  }
  candidate.scaleX = 1;
  transformSchema.parse(candidate);
  return ["invalid_input", "non_finite_value"];
};

const ambiguousMarkerFailure = (value: unknown): FixtureFailure => {
  const scenario = markerScenarioSchema.parse(value);
  if (scenario.markers.some((marker) => marker.scope !== scenario.scope)) {
    throw new Error("marker scope differs from scenario scope");
  }
  assertSingleDeclaredDuplicate(
    scenario.markers.map((marker) => `${marker.scope}\u0000${marker.name}`),
    new Set([`${scenario.scope}\u0000${scenario.lookupName}`]),
    "marker invalid envelope name"
  );
  if (
    duplicateValue(
      scenario.markers.map((marker) => marker.name),
      scenario.lookupName
    )
  ) {
    return ["ambiguous_reference", "duplicate_marker_name"];
  }
  throw new Error("marker candidate has no intended failure");
};

const invalidSpringFailure = (
  value: unknown,
  limits: Limits
): FixtureFailure => {
  const candidate = structuredClone(asRecord(value, "curve.invalid_spring"));
  const curves = asArray(candidate.curves, "curve.invalid_spring.curves");
  const spring = asRecord(curves[0], "curve.invalid_spring.curves[0]");
  if (spring.type === "spring" && spring.mass === 0) {
    spring.mass = 1;
    const scenario = curveSetSchema.parse(candidate);
    ensureAtMost(
      scenario.keyframes.length,
      limits.maxKeyframesPerChannel,
      "maxKeyframesPerChannel"
    );
    return ["invalid_input", "spring_parameter_out_of_range"];
  }
  throw new Error("spring candidate has no intended failure");
};

const rendererExpressionFailure = (
  value: unknown,
  limits: Limits
): FixtureFailure => {
  const scenario = z
    .strictObject({
      effects: z
        .array(
          z.strictObject({
            expression: z.string().min(1),
            id: ID_SCHEMA,
            type: z.literal("renderer_expression"),
          })
        )
        .min(1),
      layerId: ID_SCHEMA,
      scope: COMPOSITION_SCOPE_SCHEMA,
    })
    .parse(value);
  assertUniqueIds(
    scenario.effects.map((effect) => effect.id),
    "renderer expression invalid envelope"
  );
  ensureAtMost(
    scenario.effects.length,
    limits.maxEffectsPerLayer,
    "maxEffectsPerLayer"
  );
  if (scenario.effects.length > 0) {
    return ["invalid_input", "renderer_expression_forbidden"];
  }
  throw new Error("renderer expression candidate has no intended failure");
};

const effectLimitFailure = (
  value: unknown,
  maxEffects: number
): FixtureFailure => {
  const scenario = effectSchema.parse(value);
  assertUniqueIds(
    scenario.effects.map((effect) => effect.id),
    "effect limit invalid envelope"
  );
  if (scenario.effects.length > maxEffects) {
    return ["invalid_input", "max_effects_per_layer_exceeded"];
  }
  throw new Error("effect limit candidate has no intended failure");
};

const classifyInvalidFixture = (
  id: string,
  value: unknown,
  limits: z.infer<typeof limitsSchema>
): FixtureFailure => {
  if (id === "transform.non_finite") {
    return nonFiniteTransformFailure(value);
  }
  if (id.startsWith("layer.")) {
    return invalidLayerFailure(id, value, limits);
  }
  if (id.startsWith("component.")) {
    return componentFailure(value, limits);
  }
  if (id.startsWith("slot.")) {
    return slotFailure(id, value, limits);
  }
  if (id === "marker.ambiguous_name") {
    const scenario = markerScenarioSchema.parse(value);
    assertUniqueIds(
      scenario.markers.map((marker) => `${marker.scope}\u0000${marker.id}`),
      "marker invalid envelope"
    );
    ensureAtMost(
      scenario.markers.length,
      limits.maxMarkersPerComposition,
      "maxMarkersPerComposition"
    );
    return ambiguousMarkerFailure(scenario);
  }
  if (id === "curve.invalid_spring") {
    return invalidSpringFailure(value, limits);
  }
  if (id.startsWith("mask.")) {
    return maskFailure(value, limits);
  }
  if (id === "effect.renderer_expression") {
    return rendererExpressionFailure(value, limits);
  }
  if (id === "effect.limit_exceeded") {
    return effectLimitFailure(value, limits.maxEffectsPerLayer);
  }
  if (id.startsWith("audio_event.")) {
    return audioFailure(id, value, limits);
  }
  throw new Error(`${id} has no intended deterministic failure`);
};

const conceptSchemas = {
  audio_event: audioSchema,
  component: componentSchema,
  curve: curveSetSchema,
  effect: effectSchema,
  layer: layerSetSchema,
  marker: markerSchema,
  mask: maskSchema,
  slot: slotSchema,
  transform: transformSchema,
} as const;

type Concept = keyof typeof conceptSchemas;

const invalidExpectations: Record<string, readonly [Concept, string, string]> =
  {
    "audio_event.ambiguous_bus": [
      "audio_event",
      "ambiguous_reference",
      "audio_bus_ambiguous",
    ],
    "audio_event.ambiguous_marker": [
      "audio_event",
      "ambiguous_reference",
      "marker_ambiguous",
    ],
    "audio_event.ambiguous_sound_definition": [
      "audio_event",
      "ambiguous_reference",
      "sound_definition_ambiguous",
    ],
    "audio_event.ambiguous_variant": [
      "audio_event",
      "ambiguous_reference",
      "sound_variant_ambiguous",
    ],
    "audio_event.missing_bus": [
      "audio_event",
      "missing_reference",
      "audio_bus_not_found",
    ],
    "audio_event.missing_marker": [
      "audio_event",
      "missing_reference",
      "marker_not_found",
    ],
    "audio_event.missing_sound_definition": [
      "audio_event",
      "missing_reference",
      "sound_definition_not_found",
    ],
    "audio_event.missing_variant": [
      "audio_event",
      "missing_reference",
      "sound_variant_not_found",
    ],
    "audio_event.network_variant": [
      "audio_event",
      "invalid_input",
      "network_resource_forbidden",
    ],
    "component.depth_limit": [
      "component",
      "invalid_input",
      "max_component_depth_exceeded",
    ],
    "component.missing_definition": [
      "component",
      "missing_reference",
      "component_not_found",
    ],
    "component.recursive": ["component", "reference_cycle", "component_cycle"],
    "curve.invalid_spring": [
      "curve",
      "invalid_input",
      "spring_parameter_out_of_range",
    ],
    "effect.limit_exceeded": [
      "effect",
      "invalid_input",
      "max_effects_per_layer_exceeded",
    ],
    "effect.renderer_expression": [
      "effect",
      "invalid_input",
      "renderer_expression_forbidden",
    ],
    "layer.cross_scope_parent": [
      "layer",
      "invalid_input",
      "parent_scope_mismatch",
    ],
    "layer.direct_parent_cycle": [
      "layer",
      "reference_cycle",
      "direct_parent_cycle",
    ],
    "layer.missing_parent": ["layer", "missing_reference", "parent_not_found"],
    "layer.parent_cycle": ["layer", "reference_cycle", "parent_cycle"],
    "marker.ambiguous_name": [
      "marker",
      "ambiguous_reference",
      "duplicate_marker_name",
    ],
    "mask.arbitrary_path": [
      "mask",
      "invalid_input",
      "arbitrary_path_forbidden",
    ],
    "mask.missing_source": [
      "mask",
      "missing_reference",
      "mask_source_not_found",
    ],
    "mask.unsafe_svg": ["mask", "invalid_input", "executable_svg"],
    "slot.arbitrary_path": ["slot", "invalid_input", "unstable_binding_target"],
    "slot.constraint_violation": [
      "slot",
      "invalid_input",
      "slot_constraint_violation",
    ],
    "slot.invalid_default": [
      "slot",
      "invalid_input",
      "slot_default_type_mismatch",
    ],
    "slot.missing_target": [
      "slot",
      "missing_reference",
      "slot_target_not_found",
    ],
    "slot.required_value_missing": [
      "slot",
      "invalid_input",
      "required_slot_value_missing",
    ],
    "slot.type_mismatch": ["slot", "invalid_input", "slot_value_type_mismatch"],
    "transform.non_finite": ["transform", "invalid_input", "non_finite_value"],
  };

const asRecord = (value: unknown, label: string): Record<string, unknown> => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
};

const asArray = (value: unknown, label: string): unknown[] => {
  if (!Array.isArray(value)) {
    throw new TypeError(`${label} must be an array`);
  }
  return value;
};

const asString = (value: unknown, label: string): string => {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${label} must be a non-empty string`);
  }
  return value;
};

const ref = (kind: Reference["kind"], scope: string, id: string): Reference =>
  referenceSchema.parse({ id, kind, scope });

const refKey = (reference: Reference) =>
  `${reference.scope}|${reference.kind}|${reference.id}`;

const normalizedRefs = (references: Reference[]) =>
  [...new Map(references.map((entry) => [refKey(entry), entry])).values()].sort(
    (left, right) => refKey(left).localeCompare(refKey(right))
  );

const metadataRefs = (value: unknown, label: string) => {
  const parsed = asArray(value, label).map((entry) =>
    referenceSchema.parse(entry)
  );
  const keys = parsed.map(refKey);
  if (new Set(keys).size !== keys.length) {
    throw new Error(`${label} contains duplicates`);
  }
  return parsed;
};

const deriveReferences = (
  concept: Concept,
  payload: z.infer<(typeof conceptSchemas)[Concept]>
): { defines: Reference[]; references: Reference[] } => {
  switch (concept) {
    case "transform": {
      const value = transformSchema.parse(payload);
      return {
        defines: [ref("transform", value.scope, value.id)],
        references: [],
      };
    }
    case "layer": {
      const value = layerSetSchema.parse(payload);
      const defines = value.layers.map((layer) =>
        ref("layer", value.scope, layer.id)
      );
      const references = value.layers.flatMap((layer) => [
        ...(layer.parentId ? [ref("layer", value.scope, layer.parentId)] : []),
        ...(layer.transformId
          ? [ref("transform", value.scope, layer.transformId)]
          : []),
        ...layer.masks.map((id) => ref("mask", value.scope, id)),
        ...layer.effects.map((id) => ref("effect", value.scope, id)),
      ]);
      return { defines, references };
    }
    case "component": {
      const value = componentSchema.parse(payload);
      const scope = `component:${value.definition.id}`;
      return {
        defines: [
          ref("component", "project", value.definition.id),
          ...value.definition.layers.map((id) => ref("layer", scope, id)),
        ],
        references: [
          ref("component", "project", value.instance.componentId),
          ...value.definition.slotIds.map((id) => ref("slot", scope, id)),
          ...value.definition.markerIds.map((id) => ref("marker", scope, id)),
        ],
      };
    }
    case "slot": {
      const value = slotSchema.parse(payload);
      return {
        defines: [ref("slot", value.scope, value.id)],
        references: [ref("layer", value.scope, value.binding.targetLayerId)],
      };
    }
    case "marker": {
      const value = markerSchema.parse(payload);
      const references =
        value.relativeTime.type === "marker"
          ? [ref("marker", value.marker.scope, value.relativeTime.markerName)]
          : [];
      return {
        defines: [ref("marker", value.marker.scope, value.marker.id)],
        references,
      };
    }
    case "curve": {
      const value = curveSetSchema.parse(payload);
      const references = value.keyframes.flatMap((keyframe) =>
        keyframe.time.type === "marker"
          ? [ref("marker", value.scope, keyframe.time.markerName)]
          : []
      );
      return {
        defines: [ref("curve", value.scope, value.id)],
        references,
      };
    }
    case "mask": {
      const value = maskSchema.parse(payload);
      return {
        defines: value.masks.map((mask) => ref("mask", value.scope, mask.id)),
        references: [
          ref("layer", value.scope, value.layerId),
          ...value.masks.flatMap((mask) => [
            ref("transform", value.scope, mask.transformId),
            ...(mask.source.type === "layer"
              ? [ref("layer", value.scope, mask.source.layerId)]
              : []),
          ]),
        ],
      };
    }
    case "effect": {
      const value = effectSchema.parse(payload);
      return {
        defines: value.effects.map((effect) =>
          ref("effect", value.scope, effect.id)
        ),
        references: [ref("layer", value.scope, value.layerId)],
      };
    }
    case "audio_event": {
      const value = audioSchema.parse(payload);
      const eventScope = value.event.scope;
      const markerReferences =
        value.event.at.type === "marker"
          ? [ref("marker", eventScope, value.event.at.markerName)]
          : [];
      return {
        defines: [
          ref("audio_bus", "project", value.bus.id),
          ref("sound_definition", "project", value.soundDefinition.event),
          ref("audio_event", eventScope, value.event.id),
        ],
        references: [
          ...markerReferences,
          ...value.soundDefinition.variantAssetIds.map((id) =>
            ref("asset", "project", id)
          ),
          ref("sound_definition", "project", value.event.event),
          ref("audio_bus", "project", value.event.busId),
          ref("audio_bus", "project", value.soundDefinition.busId),
        ],
      };
    }
    default:
      throw new Error(`unknown fixture concept: ${concept}`);
  }
};

const validateLayerDepth = (payload: unknown, limit: number) => {
  const value = layerSetSchema.parse(payload);
  const parents = new Map(
    value.layers.map((layer) => [layer.id, layer.parentId] as const)
  );
  for (const layer of value.layers) {
    const seen = new Set<string>();
    let current: string | null = layer.id;
    let depth = 0;
    while (current) {
      if (seen.has(current)) {
        throw new Error("parent_cycle");
      }
      seen.add(current);
      const parent: string | null | undefined = parents.get(current);
      if (parent !== null && parent !== undefined && !parents.has(parent)) {
        throw new Error("parent_not_found");
      }
      current = parent ?? null;
      depth += 1;
      if (depth > limit) {
        throw new Error("max_parent_depth_exceeded");
      }
    }
  }
};

const assertSameReferences = (
  actualValue: unknown,
  expectedValue: unknown,
  label: string
) => {
  const actual = metadataRefs(actualValue, `${label} actual`)
    .sort((left, right) => refKey(left).localeCompare(refKey(right)))
    .map(refKey);
  const expected = normalizedRefs(
    asArray(expectedValue, `${label} expected`).map((entry) =>
      referenceSchema.parse(entry)
    )
  ).map(refKey);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${label} metadata differs from payload`);
  }
};

const assertUniqueDerivedDefinitions = (
  definitions: Reference[],
  fixtureId: string
) => {
  const keys = definitions.map(refKey);
  if (new Set(keys).size !== keys.length) {
    throw new Error(`${fixtureId} duplicate payload definition`);
  }
};

const validatePayloadLimits = (
  concept: Concept,
  payload: unknown,
  limits: z.infer<typeof limitsSchema>
) => {
  if (concept === "layer") {
    ensureAtMost(
      layerSetSchema.parse(payload).layers.length,
      limits.maxLayersPerComposition,
      "maxLayersPerComposition"
    );
  }
  if (concept === "component") {
    const { definition } = componentSchema.parse(payload);
    ensureAtMost(
      definition.layers.length,
      limits.maxLayersPerComposition,
      "maxLayersPerComposition"
    );
    ensureAtMost(
      definition.slotIds.length,
      limits.maxSlotsPerComponent,
      "maxSlotsPerComponent"
    );
    ensureAtMost(
      definition.markerIds.length,
      limits.maxMarkersPerComposition,
      "maxMarkersPerComposition"
    );
  }
  if (
    concept === "curve" &&
    curveSetSchema.parse(payload).keyframes.length >
      limits.maxKeyframesPerChannel
  ) {
    throw new Error("maxKeyframesPerChannel exceeded");
  }
  if (
    concept === "mask" &&
    maskSchema.parse(payload).masks.length > limits.maxMasksPerLayer
  ) {
    throw new Error("maxMasksPerLayer exceeded");
  }
  if (
    concept === "effect" &&
    effectSchema.parse(payload).effects.length > limits.maxEffectsPerLayer
  ) {
    throw new Error("maxEffectsPerLayer exceeded");
  }
};

interface AggregateCounts {
  audioEvents: Map<string, number>;
  components: number;
  layers: Map<string, number>;
  markers: Map<string, number>;
  slots: Map<string, number>;
}

const incrementScopedCount = (
  counts: Map<string, number>,
  scope: string,
  limit: number,
  label: string
) => {
  const count = (counts.get(scope) ?? 0) + 1;
  ensureAtMost(count, limit, label);
  counts.set(scope, count);
};

const countAggregateDefinition = (
  counts: AggregateCounts,
  definition: Reference,
  limits: z.infer<typeof limitsSchema>
) => {
  switch (definition.kind) {
    case "component":
      counts.components += 1;
      ensureAtMost(
        counts.components,
        limits.maxComponentDefinitions,
        "maxComponentDefinitions"
      );
      break;
    case "layer":
      incrementScopedCount(
        counts.layers,
        definition.scope,
        limits.maxLayersPerComposition,
        "maxLayersPerComposition"
      );
      break;
    case "marker":
      incrementScopedCount(
        counts.markers,
        definition.scope,
        limits.maxMarkersPerComposition,
        "maxMarkersPerComposition"
      );
      break;
    case "slot":
      incrementScopedCount(
        counts.slots,
        definition.scope,
        limits.maxSlotsPerComponent,
        "maxSlotsPerComponent"
      );
      break;
    case "audio_event":
      incrementScopedCount(
        counts.audioEvents,
        definition.scope,
        limits.maxAudioEventsPerComposition,
        "maxAudioEventsPerComposition"
      );
      break;
    default:
      break;
  }
};

const validateUniqueFixtureIds = (catalog: Record<string, unknown>) => {
  const fixtureIds = new Set<string>();
  for (const collection of ["validFixtures", "invalidFixtures"] as const) {
    for (const fixtureValue of asArray(catalog[collection], collection)) {
      const fixture = asRecord(fixtureValue, `${collection} fixture`);
      const id = asString(fixture.id, `${collection} fixture id`);
      if (fixtureIds.has(id)) {
        throw new Error(`duplicate fixture id: ${id}`);
      }
      fixtureIds.add(id);
    }
  }
};

const collectDefinitions = (
  catalog: Record<string, unknown>,
  limits: z.infer<typeof limitsSchema>
) => {
  const definitions = new Map<string, Reference>();
  const aggregateCounts: AggregateCounts = {
    audioEvents: new Map(),
    components: 0,
    layers: new Map(),
    markers: new Map(),
    slots: new Map(),
  };
  for (const resource of asArray(
    catalog.managedResources,
    "managedResources"
  )) {
    const parsed = managedAssetSchema.parse(resource);
    const key = refKey(parsed);
    if (definitions.has(key)) {
      throw new Error(`duplicate managed definition: ${key}`);
    }
    definitions.set(key, parsed);
  }

  for (const fixtureValue of asArray(catalog.validFixtures, "validFixtures")) {
    const fixture = asRecord(fixtureValue, "valid fixture");
    const concept = asString(
      fixture.concept,
      "valid fixture concept"
    ) as Concept;
    const schema = conceptSchemas[concept];
    if (!schema) {
      throw new Error(`unknown fixture concept ${concept}`);
    }
    const payload = schema.parse(fixture.value);
    validatePayloadLimits(concept, payload, limits);
    const derived = deriveReferences(concept, payload);
    assertUniqueDerivedDefinitions(
      derived.defines,
      asString(fixture.id, "fixture id")
    );
    assertSameReferences(
      fixture.defines,
      derived.defines,
      `${fixture.id}.defines`
    );
    assertSameReferences(
      fixture.references,
      derived.references,
      `${fixture.id}.references`
    );
    for (const definition of derived.defines) {
      const key = refKey(definition);
      if (definitions.has(key)) {
        throw new Error(`duplicate logical definition: ${key}`);
      }
      definitions.set(key, definition);
      countAggregateDefinition(aggregateCounts, definition, limits);
    }
    if (concept === "layer") {
      validateLayerDepth(payload, limits.maxParentDepth);
    }
  }
  return definitions;
};

const validateReferenceClosure = (
  catalog: Record<string, unknown>,
  definitions: Map<string, Reference>
) => {
  for (const fixtureValue of asArray(catalog.validFixtures, "validFixtures")) {
    const fixture = asRecord(fixtureValue, "valid fixture");
    for (const reference of asArray(
      fixture.references,
      `${fixture.id}.references`
    )) {
      const parsed = referenceSchema.parse(reference);
      if (!definitions.has(refKey(parsed))) {
        throw new Error(`unresolved logical reference: ${refKey(parsed)}`);
      }
    }
  }
};

const validateInvalidFixtures = (catalog: Record<string, unknown>) => {
  const limits = limitsSchema.parse(catalog.limits);
  const observedInvalid: Record<string, readonly [Concept, string, string]> =
    {};
  for (const fixtureValue of asArray(
    catalog.invalidFixtures,
    "invalidFixtures"
  )) {
    const fixture = asRecord(fixtureValue, "invalid fixture");
    const id = asString(fixture.id, "invalid fixture id");
    const concept = asString(fixture.concept, `${id}.concept`) as Concept;
    const expected = invalidExpectations[id];
    if (!expected) {
      throw new Error(`unexpected invalid fixture ID: ${id}`);
    }
    if (concept !== expected[0]) {
      throw new Error(
        `${id} concept mismatch: expected ${expected[0]}, received ${concept}`
      );
    }
    let observed: FixtureFailure;
    try {
      observed = classifyInvalidFixture(id, fixture.value, limits);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`${id} invalid envelope: ${message}`, { cause: error });
    }
    const declared = [
      asString(fixture.classification, `${id}.classification`),
      asString(fixture.reason, `${id}.reason`),
    ] as const;
    if (JSON.stringify(observed) !== JSON.stringify(declared)) {
      throw new Error(`${id} observed failure differs from declared failure`);
    }
    observedInvalid[id] = [concept, observed[0], observed[1]];
  }
  const observed = Object.entries(observedInvalid).sort(([left], [right]) =>
    left.localeCompare(right)
  );
  const expected = Object.entries(invalidExpectations).sort(([left], [right]) =>
    left.localeCompare(right)
  );
  if (JSON.stringify(observed) !== JSON.stringify(expected)) {
    throw new Error("invalid fixture IDs/classifications/reasons differ");
  }
};

export const validateMotionGraphicsCatalog = (catalogValue: unknown): void => {
  const catalog = asRecord(catalogSchema.parse(catalogValue), "catalog");
  const limits = limitsSchema.parse(catalog.limits);
  validateUniqueFixtureIds(catalog);
  const definitions = collectDefinitions(catalog, limits);
  validateReferenceClosure(catalog, definitions);
  validateInvalidFixtures(catalog);
};

const appendAggregateComponent = (
  catalog: Record<string, unknown>,
  suffix: string
) => {
  const componentId = `aggregate_component_${suffix}`;
  const layerId = `aggregate_layer_${suffix}`;
  asArray(catalog.validFixtures, "validFixtures").push({
    concept: "component",
    defines: [
      { id: componentId, kind: "component", scope: "project" },
      {
        id: layerId,
        kind: "layer",
        scope: `component:${componentId}`,
      },
    ],
    id: `component.aggregate_${suffix}`,
    references: [{ id: componentId, kind: "component", scope: "project" }],
    value: {
      definition: {
        durationMs: 1000,
        height: 1080,
        id: componentId,
        layers: [layerId],
        markerIds: [],
        name: `Aggregate component ${suffix}`,
        slotIds: [],
        trackIds: [`aggregate_track_${suffix}`],
        width: 1920,
      },
      instance: {
        componentId,
        durationMs: 1000,
        id: `aggregate_instance_${suffix}`,
        slotValues: {},
        startMs: 0,
        timeScale: 1,
        trimStartMs: 0,
      },
    },
  });
};

const appendAggregateLayer = (
  catalog: Record<string, unknown>,
  suffix: string
) => {
  const layerId = `aggregate_root_layer_${suffix}`;
  asArray(catalog.validFixtures, "validFixtures").push({
    concept: "layer",
    defines: [{ id: layerId, kind: "layer", scope: "root" }],
    id: `layer.aggregate_${suffix}`,
    references: [],
    value: {
      layers: [
        {
          animationChannels: [],
          blendMode: "normal",
          clip: null,
          effects: [],
          hidden: false,
          id: layerId,
          masks: [],
          parentId: null,
          stableItemIndex: 0,
          transformId: null,
          zIndex: 0,
        },
      ],
      scope: "root",
    },
  });
};

const appendAggregateMarker = (
  catalog: Record<string, unknown>,
  suffix: string
) => {
  const markerId = `aggregate_marker_${suffix}`;
  asArray(catalog.validFixtures, "validFixtures").push({
    concept: "marker",
    defines: [
      {
        id: markerId,
        kind: "marker",
        scope: "component:rule_card",
      },
    ],
    id: `marker.aggregate_${suffix}`,
    references: [],
    value: {
      absoluteTime: { type: "milliseconds", valueMs: 0 },
      marker: {
        id: markerId,
        kind: "cue",
        name: markerId,
        scope: "component:rule_card",
        timeMs: 0,
      },
      relativeTime: { type: "milliseconds", valueMs: 0 },
    },
  });
};

const appendAggregateSlot = (
  catalog: Record<string, unknown>,
  suffix: string
) => {
  const slotId = `aggregate_slot_${suffix}`;
  asArray(catalog.validFixtures, "validFixtures").push({
    concept: "slot",
    defines: [{ id: slotId, kind: "slot", scope: "component:rule_card" }],
    id: `slot.aggregate_${suffix}`,
    references: [
      {
        id: "card_title",
        kind: "layer",
        scope: "component:rule_card",
      },
    ],
    value: {
      binding: { property: "text.document", targetLayerId: "card_title" },
      constraints: { maxLength: 100, minLength: 0 },
      defaultValue: "",
      id: slotId,
      kind: "text",
      name: `Aggregate slot ${suffix}`,
      required: false,
      scope: "component:rule_card",
    },
  });
};

const appendAggregateAudioEvent = (
  catalog: Record<string, unknown>,
  suffix: string
) => {
  const assetId = `aggregate_asset_${suffix}`;
  const busId = `aggregate_bus_${suffix}`;
  const eventId = `aggregate_event_${suffix}`;
  const soundId = `aggregate_sound_${suffix}`;
  asArray(catalog.managedResources, "managedResources").push({
    id: assetId,
    kind: "asset",
    scope: "project",
  });
  asArray(catalog.validFixtures, "validFixtures").push({
    concept: "audio_event",
    defines: [
      { id: busId, kind: "audio_bus", scope: "project" },
      { id: soundId, kind: "sound_definition", scope: "project" },
      {
        id: eventId,
        kind: "audio_event",
        scope: "component:rule_card",
      },
    ],
    id: `audio_event.aggregate_${suffix}`,
    references: [
      { id: assetId, kind: "asset", scope: "project" },
      { id: busId, kind: "audio_bus", scope: "project" },
      { id: soundId, kind: "sound_definition", scope: "project" },
    ],
    value: {
      bus: { id: busId },
      event: {
        at: { type: "milliseconds", valueMs: 0 },
        busId,
        event: soundId,
        gainDb: 0,
        id: eventId,
        scope: "component:rule_card",
        variantSeed: 0,
      },
      soundDefinition: {
        busId,
        defaultGainDb: 0,
        event: soundId,
        variantAssetIds: [assetId],
      },
    },
  });
};

const setLimit = (
  catalog: Record<string, unknown>,
  name: keyof z.infer<typeof limitsSchema>,
  value: number
) => {
  asRecord(catalog.limits, "limits")[name] = value;
};

const expectSchemaFailure = (
  schema: z.ZodType,
  value: unknown,
  label: string
) => {
  if (schema.safeParse(value).success) {
    throw new Error(`${label} unexpectedly passed validation`);
  }
};

const assertCatalogWrapperRegressions = (catalog: Record<string, unknown>) => {
  const duplicateResource = structuredClone(catalog);
  const resources = asArray(
    duplicateResource.managedResources,
    "managedResources"
  );
  resources.push(structuredClone(resources[0]));
  expectCatalogFailure(
    duplicateResource,
    "duplicate managed resource",
    "duplicate managed definition"
  );

  const nonAssetResource = structuredClone(catalog);
  asArray(nonAssetResource.managedResources, "managedResources").push({
    id: "not_an_asset",
    kind: "component",
    scope: "project",
  });
  expectCatalogFailure(
    nonAssetResource,
    "non-asset managed resource",
    "managedResources"
  );

  const nonProjectResource = structuredClone(catalog);
  asArray(nonProjectResource.managedResources, "managedResources").push({
    id: "wrong_scope_asset",
    kind: "asset",
    scope: "root",
  });
  expectCatalogFailure(
    nonProjectResource,
    "non-project managed resource",
    "managedResources"
  );

  const unmanagedResource = structuredClone(catalog);
  asArray(unmanagedResource.managedResources, "managedResources").splice(0, 1);
  expectCatalogFailure(
    unmanagedResource,
    "unmanaged resource reference",
    "unresolved logical reference"
  );

  for (const [collection, sourceCollection] of [
    ["validFixtures", "validFixtures"],
    ["invalidFixtures", "invalidFixtures"],
    ["invalidFixtures", "validFixtures"],
  ] as const) {
    const duplicateId = structuredClone(catalog);
    const target = asArray(duplicateId[collection], collection);
    const source = asArray(duplicateId[sourceCollection], sourceCollection);
    if (collection === sourceCollection) {
      target.push(structuredClone(source[0]));
    } else {
      asRecord(target[0], `${collection}[0]`).id = asString(
        asRecord(source[0], `${sourceCollection}[0]`).id,
        "source fixture id"
      );
    }
    expectCatalogFailure(
      duplicateId,
      `duplicate fixture ID across ${sourceCollection} and ${collection}`,
      "duplicate fixture id"
    );
  }

  const mislabeledInvalid = structuredClone(catalog);
  const mislabeled = asArray(
    mislabeledInvalid.invalidFixtures,
    "invalidFixtures"
  )
    .map((fixture) => asRecord(fixture, "invalid fixture"))
    .find((fixture) => fixture.id === "slot.required_value_missing");
  if (!mislabeled) {
    throw new Error("required slot fixture missing");
  }
  mislabeled.concept = "layer";
  expectCatalogFailure(
    mislabeledInvalid,
    "mislabeled invalid fixture",
    "concept mismatch"
  );
};

const assertAggregateLimitRegressions = (catalog: Record<string, unknown>) => {
  const componentBoundary = structuredClone(catalog);
  setLimit(componentBoundary, "maxComponentDefinitions", 18);
  for (let index = 1; index < 18; index += 1) {
    appendAggregateComponent(componentBoundary, `boundary_${index}`);
  }
  validateMotionGraphicsCatalog(componentBoundary);
  const componentOverflow = structuredClone(componentBoundary);
  appendAggregateComponent(componentOverflow, "overflow");
  expectCatalogFailure(
    componentOverflow,
    "aggregate component limit",
    "maxComponentDefinitions"
  );

  const layerOwnerBoundary = structuredClone(catalog);
  setLimit(layerOwnerBoundary, "maxLayersPerComposition", 3);
  validateMotionGraphicsCatalog(layerOwnerBoundary);
  appendAggregateLayer(layerOwnerBoundary, "one");
  setLimit(layerOwnerBoundary, "maxLayersPerComposition", 4);
  validateMotionGraphicsCatalog(layerOwnerBoundary);
  const layerOverflow = structuredClone(layerOwnerBoundary);
  appendAggregateLayer(layerOverflow, "two");
  expectCatalogFailure(
    layerOverflow,
    "aggregate layer limit",
    "maxLayersPerComposition"
  );

  const markerBoundary = structuredClone(catalog);
  setLimit(markerBoundary, "maxMarkersPerComposition", 2);
  appendAggregateMarker(markerBoundary, "one");
  validateMotionGraphicsCatalog(markerBoundary);
  const markerOverflow = structuredClone(markerBoundary);
  appendAggregateMarker(markerOverflow, "two");
  expectCatalogFailure(
    markerOverflow,
    "aggregate marker limit",
    "maxMarkersPerComposition"
  );

  const slotBoundary = structuredClone(catalog);
  setLimit(slotBoundary, "maxSlotsPerComponent", 2);
  appendAggregateSlot(slotBoundary, "one");
  validateMotionGraphicsCatalog(slotBoundary);
  const slotOverflow = structuredClone(slotBoundary);
  appendAggregateSlot(slotOverflow, "two");
  expectCatalogFailure(
    slotOverflow,
    "aggregate slot limit",
    "maxSlotsPerComponent"
  );

  const audioBoundary = structuredClone(catalog);
  setLimit(audioBoundary, "maxAudioEventsPerComposition", 2);
  appendAggregateAudioEvent(audioBoundary, "one");
  validateMotionGraphicsCatalog(audioBoundary);
  const audioOverflow = structuredClone(audioBoundary);
  appendAggregateAudioEvent(audioOverflow, "two");
  expectCatalogFailure(
    audioOverflow,
    "aggregate audio-event limit",
    "maxAudioEventsPerComposition"
  );
};

const mutateFixtureValue = (
  catalog: Record<string, unknown>,
  collection: "validFixtures" | "invalidFixtures",
  fixtureId: string,
  mutate: (value: Record<string, unknown>) => void
) => {
  const fixture = asArray(catalog[collection], collection)
    .map((entry) => asRecord(entry, `${collection} entry`))
    .find((entry) => entry.id === fixtureId);
  if (!fixture) {
    throw new Error(`fixture ${fixtureId} missing`);
  }
  mutate(asRecord(fixture.value, `${fixtureId} value`));
};

const expectInvalidEnvelopeMutation = (
  catalog: Record<string, unknown>,
  fixtureId: string,
  label: string,
  mutate: (value: Record<string, unknown>) => void
) => {
  const candidate = structuredClone(catalog);
  mutateFixtureValue(candidate, "invalidFixtures", fixtureId, mutate);
  expectCatalogFailure(candidate, label, `${fixtureId} invalid envelope`);
};

const assertDuplicatePayloadDefinitionRegressions = (
  catalog: Record<string, unknown>
) => {
  for (const [fixtureId, collection] of [
    ["layer.ordered_visual", "layers"],
    ["mask.ordered_pair", "masks"],
    ["effect.ordered_stack", "effects"],
  ] as const) {
    const candidate = structuredClone(catalog);
    mutateFixtureValue(candidate, "validFixtures", fixtureId, (value) => {
      const definitions = asArray(value[collection], collection);
      definitions.push(structuredClone(definitions[0]));
    });
    expectCatalogFailure(
      candidate,
      `duplicate ${fixtureId} payload definition`,
      "duplicate payload definition"
    );
  }

  const component = structuredClone(catalog);
  mutateFixtureValue(
    component,
    "validFixtures",
    "component.rule_card",
    (value) => {
      const definition = asRecord(value.definition, "component definition");
      const layers = asArray(definition.layers, "component layers");
      layers.push(structuredClone(layers[0]));
    }
  );
  expectCatalogFailure(
    component,
    "duplicate component-layer payload definition",
    "duplicate payload definition"
  );

  for (const [fixtureId, collection] of [
    ["layer.missing_parent", "layers"],
    ["mask.unsafe_svg", "masks"],
    ["effect.renderer_expression", "effects"],
  ] as const) {
    const candidate = structuredClone(catalog);
    mutateFixtureValue(candidate, "invalidFixtures", fixtureId, (value) => {
      const definitions = asArray(value[collection], collection);
      definitions.push(structuredClone(definitions[0]));
    });
    expectCatalogFailure(
      candidate,
      `duplicate ${fixtureId} invalid definition`,
      "duplicate payload definition"
    );
  }
};

const assertInvalidContextUniquenessRegressions = (
  catalog: Record<string, unknown>
) => {
  const mutations: readonly [
    string,
    string,
    string,
    (value: Record<string, unknown>) => void,
  ][] = [
    [
      "component.recursive",
      "duplicate invalid component ID",
      "component invalid envelope duplicate payload definition",
      (value) => asArray(value.componentIds, "componentIds").push("a"),
    ],
    [
      "component.recursive",
      "duplicate invalid dependency edge",
      "component invalid envelope dependency edge duplicate payload definition",
      (value) =>
        asArray(value.dependencies, "dependencies").push({
          from: "a",
          to: "b",
        }),
    ],
    [
      "slot.required_value_missing",
      "duplicate invalid slot target",
      "slot invalid envelope target layer duplicate payload definition",
      (value) => {
        const targets = asArray(value.targetLayerIds, "targetLayerIds");
        targets.push(targets[0]);
      },
    ],
    [
      "marker.ambiguous_name",
      "duplicate invalid marker ID",
      "marker invalid envelope duplicate payload definition",
      (value) => {
        const markers = asArray(value.markers, "markers").map((marker) =>
          asRecord(marker, "marker")
        );
        const [first, second] = markers;
        if (!(first && second)) {
          throw new Error("ambiguous marker fixture is incomplete");
        }
        second.id = first.id;
      },
    ],
    [
      "mask.missing_source",
      "duplicate invalid mask context layer",
      "mask invalid envelope available layer duplicate payload definition",
      (value) => {
        const layers = asArray(value.availableLayerIds, "availableLayerIds");
        layers.push(layers[0]);
      },
    ],
    [
      "audio_event.missing_bus",
      "duplicate invalid audio asset",
      "audio invalid envelope asset duplicate payload definition",
      (value) => {
        const assets = asArray(value.assets, "assets");
        assets.push(assets[0]);
      },
    ],
    [
      "audio_event.missing_bus",
      "duplicate invalid audio event",
      "audio invalid envelope event duplicate payload definition",
      (value) => {
        const events = asArray(value.events, "events");
        events.push(structuredClone(events[0]));
      },
    ],
    [
      "audio_event.missing_bus",
      "duplicate invalid audio marker ID",
      "audio invalid envelope marker duplicate payload definition",
      (value) => {
        const markers = asArray(value.markers, "markers");
        markers.push(structuredClone(markers[0]));
      },
    ],
    [
      "audio_event.missing_bus",
      "duplicate unrelated audio bus",
      "audio invalid envelope bus duplicate payload definition",
      (value) => {
        const buses = asArray(value.buses, "buses");
        buses.push(structuredClone(buses[0]));
      },
    ],
    [
      "audio_event.missing_bus",
      "duplicate unrelated sound definition",
      "audio invalid envelope sound definition duplicate payload definition",
      (value) => {
        const definitions = asArray(value.soundDefinitions, "soundDefinitions");
        definitions.push(structuredClone(definitions[0]));
      },
    ],
    [
      "audio_event.missing_bus",
      "duplicate unrelated sound variant",
      "audio invalid envelope sound variant duplicate payload definition",
      (value) => {
        const definition = asRecord(
          asArray(value.soundDefinitions, "soundDefinitions")[0],
          "soundDefinition"
        );
        const variants = asArray(definition.variantAssetIds, "variantAssetIds");
        variants.push(variants[0]);
      },
    ],
  ];
  for (const [fixtureId, label, invariant, mutate] of mutations) {
    const candidate = structuredClone(catalog);
    mutateFixtureValue(candidate, "invalidFixtures", fixtureId, mutate);
    expectCatalogFailure(candidate, label, invariant);
  }
};

const assertInvalidEnvelopeLimitRegressions = (
  catalog: Record<string, unknown>
) => {
  const componentBoundary = structuredClone(catalog);
  setLimit(componentBoundary, "maxComponentDefinitions", 18);
  validateMotionGraphicsCatalog(componentBoundary);
  const componentOverflow = structuredClone(componentBoundary);
  mutateFixtureValue(
    componentOverflow,
    "invalidFixtures",
    "component.depth_limit",
    (value) =>
      asArray(value.componentIds, "componentIds").push("overflow_component")
  );
  expectCatalogFailure(
    componentOverflow,
    "invalid component-definition limit",
    "maxComponentDefinitions"
  );

  const layerBoundary = structuredClone(catalog);
  setLimit(layerBoundary, "maxLayersPerComposition", 3);
  mutateFixtureValue(
    layerBoundary,
    "invalidFixtures",
    "layer.missing_parent",
    (value) => {
      const layers = asArray(value.layers, "layers");
      for (const id of ["boundary_layer_one", "boundary_layer_two"]) {
        layers.push({
          ...structuredClone(asRecord(layers[0], "layer")),
          id,
          parentId: null,
        });
      }
    }
  );
  validateMotionGraphicsCatalog(layerBoundary);
  const layerOverflow = structuredClone(layerBoundary);
  mutateFixtureValue(
    layerOverflow,
    "invalidFixtures",
    "layer.missing_parent",
    (value) => {
      asArray(value.layers, "layers").push({
        ...structuredClone(
          asRecord(asArray(value.layers, "layers")[0], "layer")
        ),
        id: "overflow_layer",
        parentId: null,
      });
    }
  );
  expectCatalogFailure(
    layerOverflow,
    "invalid layer limit",
    "maxLayersPerComposition"
  );

  const markerBoundary = structuredClone(catalog);
  setLimit(markerBoundary, "maxMarkersPerComposition", 2);
  validateMotionGraphicsCatalog(markerBoundary);
  const markerOverflow = structuredClone(markerBoundary);
  mutateFixtureValue(
    markerOverflow,
    "invalidFixtures",
    "marker.ambiguous_name",
    (value) => {
      asArray(value.markers, "markers").push({
        id: "overflow_marker",
        kind: "cue",
        name: "unrelated_marker",
        scope: "component:rule_card",
        timeMs: 300,
      });
    }
  );
  expectCatalogFailure(
    markerOverflow,
    "invalid marker limit",
    "maxMarkersPerComposition"
  );

  const keyframeBoundary = structuredClone(catalog);
  setLimit(keyframeBoundary, "maxKeyframesPerChannel", 2);
  mutateFixtureValue(
    keyframeBoundary,
    "invalidFixtures",
    "curve.invalid_spring",
    (value) => {
      const keyframes = asArray(value.keyframes, "keyframes");
      keyframes.push(
        {
          curve: { type: "linear" },
          time: { type: "milliseconds", valueMs: 0 },
          value: 0,
        },
        {
          curve: { type: "hold" },
          time: { type: "milliseconds", valueMs: 1 },
          value: 1,
        }
      );
    }
  );
  validateMotionGraphicsCatalog(keyframeBoundary);
  const keyframeOverflow = structuredClone(keyframeBoundary);
  mutateFixtureValue(
    keyframeOverflow,
    "invalidFixtures",
    "curve.invalid_spring",
    (value) => {
      asArray(value.keyframes, "keyframes").push({
        curve: { type: "linear" },
        time: { type: "milliseconds", valueMs: 2 },
        value: 2,
      });
    }
  );
  expectCatalogFailure(
    keyframeOverflow,
    "invalid keyframe limit",
    "maxKeyframesPerChannel"
  );

  const safeMask = (id: string) => ({
    channel: "alpha",
    expansionPx: 0,
    featherPx: 0,
    id,
    inverted: false,
    operation: "add",
    source: { commands: [{ type: "close" }], type: "path" },
    transformId: "hero",
  });
  const maskBoundary = structuredClone(catalog);
  setLimit(maskBoundary, "maxMasksPerLayer", 16);
  mutateFixtureValue(
    maskBoundary,
    "invalidFixtures",
    "mask.missing_source",
    (value) => {
      const masks = asArray(value.masks, "masks");
      for (let index = 1; index < 16; index += 1) {
        masks.push(safeMask(`boundary_mask_${index}`));
      }
    }
  );
  validateMotionGraphicsCatalog(maskBoundary);
  const maskOverflow = structuredClone(maskBoundary);
  mutateFixtureValue(
    maskOverflow,
    "invalidFixtures",
    "mask.missing_source",
    (value) => {
      asArray(value.masks, "masks").push(safeMask("overflow_mask"));
    }
  );
  expectCatalogFailure(maskOverflow, "invalid mask limit", "maxMasksPerLayer");

  const effectBoundary = structuredClone(catalog);
  setLimit(effectBoundary, "maxEffectsPerLayer", 2);
  mutateFixtureValue(
    effectBoundary,
    "invalidFixtures",
    "effect.renderer_expression",
    (value) => {
      asArray(value.effects, "effects").push({
        expression: "opacity * 0.5",
        id: "boundary_expression",
        type: "renderer_expression",
      });
    }
  );
  validateMotionGraphicsCatalog(effectBoundary);
  const effectOverflow = structuredClone(effectBoundary);
  mutateFixtureValue(
    effectOverflow,
    "invalidFixtures",
    "effect.renderer_expression",
    (value) => {
      asArray(value.effects, "effects").push({
        expression: "opacity * 0.25",
        id: "overflow_expression",
        type: "renderer_expression",
      });
    }
  );
  expectCatalogFailure(
    effectOverflow,
    "invalid effect limit",
    "maxEffectsPerLayer"
  );

  const audioBoundary = structuredClone(catalog);
  setLimit(audioBoundary, "maxAudioEventsPerComposition", 2);
  mutateFixtureValue(
    audioBoundary,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const events = asArray(value.events, "events");
      events.push({
        ...structuredClone(asRecord(events[0], "event")),
        id: "impact_02",
      });
    }
  );
  validateMotionGraphicsCatalog(audioBoundary);
  const audioOverflow = structuredClone(audioBoundary);
  mutateFixtureValue(
    audioOverflow,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const events = asArray(value.events, "events");
      events.push({
        ...structuredClone(asRecord(events[0], "event")),
        id: "impact_03",
      });
    }
  );
  expectCatalogFailure(
    audioOverflow,
    "invalid audio-event limit",
    "maxAudioEventsPerComposition"
  );
};

const assertBranchingComponentGraphRegressions = (
  catalog: Record<string, unknown>
) => {
  const branchingCycle = structuredClone(catalog);
  mutateFixtureValue(
    branchingCycle,
    "invalidFixtures",
    "component.recursive",
    (value) => {
      asArray(value.componentIds, "componentIds").push("c");
      asArray(value.dependencies, "dependencies").push({ from: "a", to: "c" });
    }
  );
  validateMotionGraphicsCatalog(branchingCycle);

  const directCycle = structuredClone(catalog);
  mutateFixtureValue(
    directCycle,
    "invalidFixtures",
    "component.recursive",
    (value) => {
      value.dependencies = [{ from: "a", to: "a" }];
    }
  );
  validateMotionGraphicsCatalog(directCycle);

  const missingEntry = structuredClone(catalog);
  mutateFixtureValue(
    missingEntry,
    "invalidFixtures",
    "component.missing_definition",
    (value) => {
      value.dependencies = [];
      value.entryId = "absent_entry";
    }
  );
  validateMotionGraphicsCatalog(missingEntry);

  const branchingDepth = structuredClone(catalog);
  mutateFixtureValue(
    branchingDepth,
    "invalidFixtures",
    "component.depth_limit",
    (value) => {
      asArray(value.componentIds, "componentIds").push("short_branch");
      asArray(value.dependencies, "dependencies").push({
        from: "c0",
        to: "short_branch",
      });
    }
  );
  validateMotionGraphicsCatalog(branchingDepth);

  const duplicateEdge = structuredClone(catalog);
  mutateFixtureValue(
    duplicateEdge,
    "invalidFixtures",
    "component.recursive",
    (value) => {
      asArray(value.dependencies, "dependencies").push({ from: "a", to: "b" });
    }
  );
  expectCatalogFailure(
    duplicateEdge,
    "duplicate branching dependency edge",
    "component invalid envelope dependency edge duplicate payload definition"
  );
};

const assertAudioScopeAndAmbiguityRegressions = (
  catalog: Record<string, unknown>
) => {
  for (const fixtureId of [
    "audio_event.missing_bus",
    "audio_event.ambiguous_bus",
    "audio_event.missing_marker",
    "audio_event.ambiguous_marker",
    "audio_event.missing_sound_definition",
    "audio_event.ambiguous_sound_definition",
    "audio_event.missing_variant",
    "audio_event.ambiguous_variant",
    "audio_event.network_variant",
  ]) {
    const missingDefinitionBus = structuredClone(catalog);
    mutateFixtureValue(
      missingDefinitionBus,
      "invalidFixtures",
      fixtureId,
      (value) => {
        const definition = asRecord(
          asArray(value.soundDefinitions, "soundDefinitions")[0],
          "soundDefinition"
        );
        definition.busId = "unresolved_bus";
      }
    );
    expectCatalogFailure(
      missingDefinitionBus,
      `${fixtureId} missing sound-definition bus`,
      "audio invalid envelope sound definition bus missing reference"
    );

    const restoredDefinitionBus = structuredClone(missingDefinitionBus);
    mutateFixtureValue(
      restoredDefinitionBus,
      "invalidFixtures",
      fixtureId,
      (value) => {
        asArray(value.buses, "buses").push({ id: "unresolved_bus" });
      }
    );
    validateMotionGraphicsCatalog(restoredDefinitionBus);
  }

  const scopedEvents = structuredClone(catalog);
  mutateFixtureValue(
    scopedEvents,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const events = asArray(value.events, "events");
      events.push({
        ...structuredClone(asRecord(events[0], "event")),
        at: { type: "milliseconds", valueMs: 0 },
        scope: "root",
      });
    }
  );
  validateMotionGraphicsCatalog(scopedEvents);

  const scopedMarkers = structuredClone(catalog);
  mutateFixtureValue(
    scopedMarkers,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const markers = asArray(value.markers, "markers");
      markers.push({
        ...structuredClone(asRecord(markers[0], "marker")),
        name: "root_impact",
        scope: "root",
      });
    }
  );
  validateMotionGraphicsCatalog(scopedMarkers);

  const distributedEvents = structuredClone(catalog);
  setLimit(distributedEvents, "maxAudioEventsPerComposition", 1);
  mutateFixtureValue(
    distributedEvents,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const events = asArray(value.events, "events");
      events.push({
        ...structuredClone(asRecord(events[0], "event")),
        at: { type: "milliseconds", valueMs: 0 },
        id: "root_event",
        scope: "root",
      });
    }
  );
  validateMotionGraphicsCatalog(distributedEvents);

  const eventBoundary = structuredClone(catalog);
  setLimit(eventBoundary, "maxAudioEventsPerComposition", 2);
  mutateFixtureValue(
    eventBoundary,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const events = asArray(value.events, "events");
      events.push({
        ...structuredClone(asRecord(events[0], "event")),
        id: "impact_02",
      });
    }
  );
  validateMotionGraphicsCatalog(eventBoundary);
  const eventOverflow = structuredClone(eventBoundary);
  mutateFixtureValue(
    eventOverflow,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const events = asArray(value.events, "events");
      events.push({
        ...structuredClone(asRecord(events[0], "event")),
        id: "impact_03",
      });
    }
  );
  expectCatalogFailure(
    eventOverflow,
    "same-owner audio-event overflow",
    "maxAudioEventsPerComposition"
  );

  const distributedMarkers = structuredClone(catalog);
  setLimit(distributedMarkers, "maxMarkersPerComposition", 2);
  mutateFixtureValue(
    distributedMarkers,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const marker = asRecord(asArray(value.markers, "markers")[0], "marker");
      asArray(value.markers, "markers").push(
        {
          ...structuredClone(marker),
          id: "root_marker",
          name: "root_marker",
          scope: "root",
        },
        {
          ...structuredClone(marker),
          id: "other_marker",
          name: "other_marker",
          scope: "component:other",
        }
      );
    }
  );
  validateMotionGraphicsCatalog(distributedMarkers);

  const markerBoundary = structuredClone(catalog);
  setLimit(markerBoundary, "maxMarkersPerComposition", 2);
  mutateFixtureValue(
    markerBoundary,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const marker = asRecord(asArray(value.markers, "markers")[0], "marker");
      asArray(value.markers, "markers").push(
        {
          ...structuredClone(marker),
          id: "root_one",
          name: "root_one",
          scope: "root",
        },
        {
          ...structuredClone(marker),
          id: "root_two",
          name: "root_two",
          scope: "root",
        }
      );
    }
  );
  validateMotionGraphicsCatalog(markerBoundary);
  const markerOverflow = structuredClone(markerBoundary);
  mutateFixtureValue(
    markerOverflow,
    "invalidFixtures",
    "audio_event.missing_bus",
    (value) => {
      const marker = asRecord(asArray(value.markers, "markers")[0], "marker");
      asArray(value.markers, "markers").push({
        ...structuredClone(marker),
        id: "root_three",
        name: "root_three",
        scope: "root",
      });
    }
  );
  expectCatalogFailure(
    markerOverflow,
    "same-owner marker overflow",
    "maxMarkersPerComposition"
  );

  const unrelatedDuplicates: readonly [
    string,
    string,
    (value: Record<string, unknown>) => void,
  ][] = [
    [
      "marker.ambiguous_name",
      "marker invalid envelope name",
      (value) => {
        const marker = asRecord(asArray(value.markers, "markers")[0], "marker");
        asArray(value.markers, "markers").push(
          { ...structuredClone(marker), id: "other_one", name: "other" },
          { ...structuredClone(marker), id: "other_two", name: "other" }
        );
      },
    ],
    [
      "audio_event.ambiguous_bus",
      "audio invalid envelope bus",
      (value) =>
        asArray(value.buses, "buses").push({ id: "other" }, { id: "other" }),
    ],
    [
      "audio_event.ambiguous_marker",
      "audio invalid envelope marker name",
      (value) => {
        const marker = asRecord(asArray(value.markers, "markers")[0], "marker");
        asArray(value.markers, "markers").push(
          { ...structuredClone(marker), id: "other_one", name: "other" },
          { ...structuredClone(marker), id: "other_two", name: "other" }
        );
      },
    ],
    [
      "audio_event.ambiguous_sound_definition",
      "audio invalid envelope sound definition",
      (value) =>
        asArray(value.soundDefinitions, "soundDefinitions").push(
          {
            busId: "sfx",
            defaultGainDb: 0,
            event: "other",
            variantAssetIds: ["sfx_impact_a"],
          },
          {
            busId: "sfx",
            defaultGainDb: 0,
            event: "other",
            variantAssetIds: ["sfx_impact_a"],
          }
        ),
    ],
    [
      "audio_event.ambiguous_variant",
      "audio invalid envelope sound variant",
      (value) =>
        asArray(value.soundDefinitions, "soundDefinitions").push({
          busId: "sfx",
          defaultGainDb: 0,
          event: "other",
          variantAssetIds: ["sfx_impact_a", "sfx_impact_a"],
        }),
    ],
  ];
  for (const [fixtureId, invariant, mutate] of unrelatedDuplicates) {
    const candidate = structuredClone(catalog);
    mutateFixtureValue(candidate, "invalidFixtures", fixtureId, mutate);
    expectCatalogFailure(
      candidate,
      `${fixtureId} unrelated ambiguity duplicate`,
      `${invariant} duplicate payload definition outside declared ambiguity`
    );
  }

  for (const [fixtureId, mutate] of [
    [
      "marker.ambiguous_name",
      (value: Record<string, unknown>) => {
        const marker = asRecord(asArray(value.markers, "markers")[0], "marker");
        asArray(value.markers, "markers").push({
          ...structuredClone(marker),
          id: "third_impact",
        });
      },
    ],
    [
      "audio_event.ambiguous_bus",
      (value: Record<string, unknown>) => {
        const event = asRecord(asArray(value.events, "events")[0], "event");
        asArray(value.buses, "buses").push({ id: event.busId });
      },
    ],
    [
      "audio_event.ambiguous_marker",
      (value: Record<string, unknown>) => {
        const marker = asRecord(asArray(value.markers, "markers")[0], "marker");
        asArray(value.markers, "markers").push({
          ...structuredClone(marker),
          id: "third_marker",
        });
      },
    ],
    [
      "audio_event.ambiguous_sound_definition",
      (value: Record<string, unknown>) => {
        const definition = asRecord(
          asArray(value.soundDefinitions, "soundDefinitions")[0],
          "soundDefinition"
        );
        asArray(value.soundDefinitions, "soundDefinitions").push(
          structuredClone(definition)
        );
      },
    ],
    [
      "audio_event.ambiguous_variant",
      (value: Record<string, unknown>) => {
        const definition = asRecord(
          asArray(value.soundDefinitions, "soundDefinitions")[0],
          "soundDefinition"
        );
        const variants = asArray(definition.variantAssetIds, "variantAssetIds");
        variants.push(variants[0]);
      },
    ],
  ] as const) {
    const candidate = structuredClone(catalog);
    mutateFixtureValue(candidate, "invalidFixtures", fixtureId, mutate);
    validateMotionGraphicsCatalog(candidate);
  }
};

const assertMaskSafetyParityRegressions = (
  catalog: Record<string, unknown>
) => {
  for (const svg of [
    '<svg onclick="run()"></svg>',
    '<svg OnErRoR   ="run()"></svg>',
  ]) {
    const candidate = structuredClone(catalog);
    mutateFixtureValue(
      candidate,
      "invalidFixtures",
      "mask.unsafe_svg",
      (value) => {
        const mask = asRecord(asArray(value.masks, "masks")[0], "mask");
        asRecord(mask.source, "mask source").svg = svg;
      }
    );
    validateMotionGraphicsCatalog(candidate);
  }

  const laterUnsafeMask = structuredClone(catalog);
  mutateFixtureValue(
    laterUnsafeMask,
    "invalidFixtures",
    "mask.unsafe_svg",
    (value) => {
      asArray(value.masks, "masks").unshift({
        channel: "alpha",
        expansionPx: 0,
        featherPx: 0,
        id: "safe_mask",
        inverted: false,
        operation: "add",
        source: { commands: [{ type: "close" }], type: "path" },
        transformId: "hero",
      });
    }
  );
  validateMotionGraphicsCatalog(laterUnsafeMask);
};

const assertInvalidEnvelopeParityRegressions = (
  catalog: Record<string, unknown>
) => {
  const mutations: readonly [
    string,
    string,
    (value: Record<string, unknown>) => void,
  ][] = [
    [
      "transform.non_finite",
      "malformed transform invalid envelope",
      (value) => {
        asRecord(value.position, "position").x = "bad";
      },
    ],
    [
      "layer.missing_parent",
      "malformed layer invalid envelope",
      (value) => {
        asRecord(asArray(value.layers, "layers")[0], "layer").stableItemIndex =
          "bad";
      },
    ],
    [
      "component.recursive",
      "malformed component invalid envelope",
      (value) => {
        value.entryId = "bad.id";
      },
    ],
    [
      "layer.missing_parent",
      "empty layer ID invalid envelope",
      (value) => {
        asRecord(asArray(value.layers, "layers")[0], "layer").id = "";
      },
    ],
    [
      "slot.required_value_missing",
      "malformed slot invalid envelope",
      (value) => {
        asRecord(value.definition, "slot definition").name = "";
      },
    ],
    [
      "slot.required_value_missing",
      "illegal slot scope invalid envelope",
      (value) => {
        asRecord(value.definition, "slot definition").scope = "root";
      },
    ],
    [
      "slot.required_value_missing",
      "invalid slot constraints envelope",
      (value) => {
        const constraints = asRecord(
          asRecord(value.definition, "slot definition").constraints,
          "slot constraints"
        );
        constraints.minLength = 121;
        constraints.maxLength = 120;
      },
    ],
    [
      "marker.ambiguous_name",
      "malformed marker invalid envelope",
      (value) => {
        asRecord(asArray(value.markers, "markers")[0], "marker").timeMs =
          Number.MAX_SAFE_INTEGER + 1;
      },
    ],
    [
      "marker.ambiguous_name",
      "missing marker collection invalid envelope",
      (value) => {
        value.markers = [];
      },
    ],
    [
      "curve.invalid_spring",
      "malformed curve invalid envelope",
      (value) => {
        value.scope = "project";
      },
    ],
    [
      "mask.missing_source",
      "malformed mask invalid envelope",
      (value) => {
        asRecord(asArray(value.masks, "masks")[0], "mask").transformId =
          "bad.id";
      },
    ],
    [
      "mask.missing_source",
      "malformed numeric mask envelope",
      (value) => {
        asRecord(asArray(value.masks, "masks")[0], "mask").featherPx = "NaN";
      },
    ],
    [
      "effect.renderer_expression",
      "malformed effect invalid envelope",
      (value) => {
        value.layerId = "bad.id";
      },
    ],
    [
      "audio_event.missing_bus",
      "malformed audio invalid envelope",
      (value) => {
        asRecord(asArray(value.events, "events")[0], "audio event").gainDb = 25;
      },
    ],
  ];
  for (const [fixtureId, label, mutate] of mutations) {
    expectInvalidEnvelopeMutation(catalog, fixtureId, label, mutate);
  }

  const unicodeBoundary = structuredClone(catalog);
  mutateFixtureValue(
    unicodeBoundary,
    "invalidFixtures",
    "slot.required_value_missing",
    (value) => {
      asRecord(value.definition, "slot definition").name = "😀".repeat(200);
    }
  );
  validateMotionGraphicsCatalog(unicodeBoundary);
  expectInvalidEnvelopeMutation(
    catalog,
    "slot.required_value_missing",
    "overlong astral-Unicode invalid envelope",
    (value) => {
      asRecord(value.definition, "slot definition").name = "😀".repeat(201);
    }
  );
};

const assertCorrectedFieldParity = (
  fixtures: Record<string, unknown>[],
  component: Record<string, unknown>,
  slot: Record<string, unknown>
) => {
  const malformedLayer = layerSetSchema.parse(
    fixtures.find((fixture) => fixture.id === "layer.ordered_visual")?.value
  );
  const firstMalformedLayer = malformedLayer.layers.at(0);
  if (!firstMalformedLayer) {
    throw new Error("layer fixture has no layers");
  }
  firstMalformedLayer.animationChannels = [""];
  expectSchemaFailure(
    layerSetSchema,
    malformedLayer,
    "empty animation channel"
  );

  const invalidTrack = componentSchema.parse(structuredClone(component.value));
  invalidTrack.definition.trackIds = ["invalid.track"];
  expectSchemaFailure(
    componentSchema,
    invalidTrack,
    "invalid component track ID"
  );

  const invalidSlotKey = componentSchema.parse(
    structuredClone(component.value)
  );
  invalidSlotKey.instance.slotValues = { "invalid.slot": "value" };
  expectSchemaFailure(
    componentSchema,
    invalidSlotKey,
    "invalid slot-value key"
  );

  const marker = fixtures.find(
    (fixture) => fixture.id === "marker.scoped_impact"
  );
  if (!marker) {
    throw new Error("marker fixture missing");
  }
  const invalidMarkerId = markerSchema.parse(structuredClone(marker.value));
  invalidMarkerId.marker.id = "invalid.marker";
  expectSchemaFailure(markerSchema, invalidMarkerId, "invalid marker ID");
  const invalidMarkerName = markerSchema.parse(structuredClone(marker.value));
  invalidMarkerName.marker.name = "invalid.marker";
  expectSchemaFailure(markerSchema, invalidMarkerName, "invalid marker name");
  const invalidMarkerTime = markerSchema.parse(structuredClone(marker.value));
  invalidMarkerTime.marker.timeMs = Number.MAX_SAFE_INTEGER + 1;
  expectSchemaFailure(
    markerSchema,
    invalidMarkerTime,
    "unsafe marker timestamp"
  );

  const curve = fixtures.find((fixture) => fixture.id === "curve.complete_set");
  if (!curve) {
    throw new Error("curve fixture missing");
  }
  const emptyCurves = curveSetSchema.parse(structuredClone(curve.value));
  emptyCurves.curves = [];
  expectSchemaFailure(curveSetSchema, emptyCurves, "empty curve collection");

  const zeroMaximumSlot = asRecord(
    structuredClone(slot.value),
    "zero max slot"
  );
  zeroMaximumSlot.constraints = { maxLength: 0, minLength: 0 };
  expectSchemaFailure(slotSchema, zeroMaximumSlot, "zero slot maximum length");
};

const assertLimitBoundaryRegressions = (catalog: Record<string, unknown>) => {
  const parsedLimits = limitsSchema.parse(catalog.limits);
  for (const [name, limit] of Object.entries(parsedLimits)) {
    ensureAtMost(limit, limit, name);
    try {
      ensureAtMost(limit + 1, limit, name);
    } catch {
      continue;
    }
    throw new Error(`${name} overflow passed validation`);
  }
  for (const name of Object.keys(parsedLimits)) {
    const boundary = { ...parsedLimits, [name]: Number.MAX_SAFE_INTEGER };
    limitsSchema.parse(boundary);
    const overflow = {
      ...parsedLimits,
      [name]: Number.MAX_SAFE_INTEGER + 1,
    };
    const result = limitsSchema.safeParse(overflow);
    if (result.success) {
      throw new Error(`${name} unsafe catalog limit passed validation`);
    }
    if (!result.error.issues.some((issue) => issue.path[0] === name)) {
      throw new Error(`${name} overflow failed for an unrelated invariant`);
    }
  }
};

export const assertMalformedPayloadRegressions = (
  catalogValue: unknown
): void => {
  const catalog = asRecord(structuredClone(catalogValue), "catalog");
  const fixtures = asArray(catalog.validFixtures, "validFixtures").map(
    (entry) => asRecord(entry, "valid fixture")
  );
  const transform = fixtures.find(
    (fixture) => fixture.id === "transform.complete"
  );
  if (!transform) {
    throw new Error("transform fixture missing");
  }
  const wrongType = {
    ...asRecord(transform.value, "transform"),
    opacity: "opaque",
  };
  if (transformSchema.safeParse(wrongType).success) {
    throw new Error("string opacity passed strict validation");
  }
  for (const unsafeResource of [
    "/tmp/mask.svg",
    "C:/outside/mask.svg",
    "C:\\outside\\mask.svg",
    "\\\\server\\share\\mask.svg",
    "../mask.svg",
    "https://example.invalid/mask.svg",
    "filter_complex=overlay",
  ]) {
    const unknownField = {
      ...asRecord(transform.value, "transform"),
      resource: unsafeResource,
    };
    if (transformSchema.safeParse(unknownField).success) {
      throw new Error(`unsafe resource field passed: ${unsafeResource}`);
    }
  }
  const missingRequired = {
    ...asRecord(transform.value, "transform"),
    position: undefined,
  };
  if (transformSchema.safeParse(missingRequired).success) {
    throw new Error("missing required field passed strict validation");
  }

  const slot = fixtures.find((fixture) => fixture.id === "slot.typed_title");
  if (!slot) {
    throw new Error("slot fixture missing");
  }
  const textWithUrl = {
    ...asRecord(slot.value, "slot"),
    defaultValue: "Read https://example.com safely",
  };
  if (!slotSchema.safeParse(textWithUrl).success) {
    throw new Error("ordinary URL-like text was rejected");
  }

  const component = fixtures.find(
    (fixture) => fixture.id === "component.rule_card"
  );
  if (!component) {
    throw new Error("component fixture missing");
  }
  const drifted = structuredClone(component);
  const references = asArray(drifted.references, "component references");
  references[1] = { id: "title", kind: "slot", scope: "root" };
  const payload = componentSchema.parse(drifted.value);
  const derived = deriveReferences("component", payload);
  let rejected = false;
  try {
    assertSameReferences(
      drifted.references,
      derived.references,
      "scope mismatch"
    );
  } catch {
    rejected = true;
  }
  if (!rejected) {
    throw new Error("cross-scope metadata drift passed validation");
  }

  assertCatalogWrapperRegressions(catalog);
  assertAggregateLimitRegressions(catalog);
  assertDuplicatePayloadDefinitionRegressions(catalog);
  assertInvalidContextUniquenessRegressions(catalog);
  assertInvalidEnvelopeLimitRegressions(catalog);
  assertBranchingComponentGraphRegressions(catalog);
  assertAudioScopeAndAmbiguityRegressions(catalog);
  assertMaskSafetyParityRegressions(catalog);
  assertInvalidEnvelopeParityRegressions(catalog);

  const swapped = structuredClone(catalog);
  const invalidFixtures = asArray(
    swapped.invalidFixtures,
    "invalidFixtures"
  ).map((fixture) => asRecord(fixture, "invalid fixture"));
  const missingBus = invalidFixtures.find(
    (fixture) => fixture.id === "audio_event.missing_bus"
  );
  const nonFinite = invalidFixtures.find(
    (fixture) => fixture.id === "transform.non_finite"
  );
  if (!(missingBus && nonFinite)) {
    throw new Error("swap sentinel fixtures missing");
  }
  [missingBus.value, nonFinite.value] = [nonFinite.value, missingBus.value];
  expectCatalogFailure(swapped, "swapped invalid payloads", "invalid envelope");

  SAFE_INTEGER.parse(Number.MAX_SAFE_INTEGER);
  if (SAFE_INTEGER.safeParse(Number.MAX_SAFE_INTEGER + 1).success) {
    throw new Error("unsafe integer passed validation");
  }

  const unicodeSlot = asRecord(structuredClone(slot.value), "Unicode slot");
  unicodeSlot.defaultValue = "é";
  unicodeSlot.constraints = { maxLength: 1, minLength: 1 };
  slotSchema.parse(unicodeSlot);

  assertCorrectedFieldParity(fixtures, component, slot);
  assertLimitBoundaryRegressions(catalog);
};

const expectCatalogFailure = (
  catalog: unknown,
  label: string,
  expectedInvariant: string
) => {
  try {
    validateMotionGraphicsCatalog(catalog);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (!message.includes(expectedInvariant)) {
      throw new Error(
        `${label} failed for an unrelated invariant: expected ${expectedInvariant}, received ${message}`,
        { cause: error }
      );
    }
    return;
  }
  throw new Error(`${label} unexpectedly passed validation`);
};
