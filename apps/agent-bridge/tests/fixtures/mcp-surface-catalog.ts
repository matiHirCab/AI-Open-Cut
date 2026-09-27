type JsonRecord = Record<string, unknown>;

export interface McpSurfaceCatalog {
  capabilityIdentifiers: string[];
  prompts: string[];
  resources: { name: string; uriTemplate: string }[];
  toolDefinitions: Record<
    string,
    {
      annotations: JsonRecord;
      inputSchema: JsonRecord;
      outputSchema: JsonRecord;
    }
  >;
  tools: string[];
  version: number;
}

const DEFINITION_NAME = /^[A-Za-z][A-Za-z0-9]*$/;
const REFERENCE = /^#\/\$defs\/([A-Za-z][A-Za-z0-9]*)$/;

const asRecord = (value: unknown, label: string): JsonRecord => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as JsonRecord;
};

const assertKeys = (value: JsonRecord, keys: string[], label: string) => {
  if (
    Object.keys(value).length !== keys.length ||
    keys.some((key) => !Object.hasOwn(value, key))
  ) {
    throw new Error(`${label} has unexpected or missing fields`);
  }
};

export const expandMcpSurfaceCatalog = (source: unknown): McpSurfaceCatalog => {
  const catalog = asRecord(source, "MCP catalog");
  assertKeys(
    catalog,
    [
      "capabilityIdentifiers",
      "prompts",
      "resources",
      "toolDefinitions",
      "tools",
      "version",
      "$defs",
    ],
    "MCP catalog"
  );
  const definitions = asRecord(catalog.$defs, "MCP catalog $defs");
  const sourceTools = asRecord(
    catalog.toolDefinitions,
    "MCP catalog toolDefinitions"
  );
  const memo = new Map<string, JsonRecord>();
  const used = new Set<string>();

  for (const [name, value] of Object.entries(definitions)) {
    if (!DEFINITION_NAME.test(name)) {
      throw new Error(`Invalid MCP definition name: ${name}`);
    }
    asRecord(value, `MCP definition ${name}`);
  }

  function resolveReference(object: JsonRecord, stack: string[]): JsonRecord {
    if (Object.keys(object).length !== 1) {
      throw new Error("MCP reference must have no sibling fields");
    }
    const match =
      typeof object.$ref === "string" ? REFERENCE.exec(object.$ref) : null;
    if (!match) {
      throw new Error(
        `Malformed or non-local MCP reference: ${String(object.$ref)}`
      );
    }
    const [, name] = match;
    if (!name) {
      throw new Error("Malformed MCP definition name");
    }
    if (!Object.hasOwn(definitions, name)) {
      throw new Error(`Missing MCP definition: ${name}`);
    }
    if (stack.includes(name)) {
      throw new Error(
        `Cyclic MCP definition: ${[...stack, name].join(" -> ")}`
      );
    }
    used.add(name);
    const cached = memo.get(name);
    if (cached) {
      return structuredClone(cached);
    }
    const resolved = asRecord(
      expand(definitions[name], [...stack, name]),
      `Expanded MCP definition ${name}`
    );
    memo.set(name, resolved);
    return structuredClone(resolved);
  }

  function expand(value: unknown, stack: string[]): unknown {
    if (Array.isArray(value)) {
      return value.map((child) => expand(child, stack));
    }
    if (value === null || typeof value !== "object") {
      return value;
    }
    const object = asRecord(value, "MCP schema node");
    if (Object.hasOwn(object, "$ref")) {
      return resolveReference(object, stack);
    }
    return Object.fromEntries(
      Object.entries(object).map(([key, child]) => [key, expand(child, stack)])
    );
  }

  const expandedTools = Object.fromEntries(
    Object.entries(sourceTools).map(([name, rawDefinition]) => {
      const definition = asRecord(rawDefinition, `MCP tool ${name}`);
      assertKeys(
        definition,
        ["annotations", "inputSchema", "outputSchema"],
        `MCP tool ${name}`
      );
      return [
        name,
        {
          annotations: asRecord(
            definition.annotations,
            `MCP tool ${name} annotations`
          ),
          inputSchema: asRecord(
            expand(definition.inputSchema, []),
            `MCP tool ${name} inputSchema`
          ),
          outputSchema: asRecord(
            expand(definition.outputSchema, []),
            `MCP tool ${name} outputSchema`
          ),
        },
      ];
    })
  );

  const unused = Object.keys(definitions).filter((name) => !used.has(name));
  if (unused.length > 0) {
    throw new Error(`Unused MCP definitions: ${unused.join(", ")}`);
  }

  return Object.fromEntries(
    Object.entries(catalog)
      .filter(([key]) => key !== "$defs")
      .map(([key, value]) => [
        key,
        key === "toolDefinitions" ? expandedTools : value,
      ])
  ) as unknown as McpSurfaceCatalog;
};
