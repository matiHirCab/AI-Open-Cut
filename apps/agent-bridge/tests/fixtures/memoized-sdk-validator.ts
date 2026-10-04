import type {
  JsonSchemaType,
  JsonSchemaValidator,
  jsonSchemaValidator,
} from "@modelcontextprotocol/client";
import { AjvJsonSchemaValidator } from "@modelcontextprotocol/client/validators/ajv";

export const memoizedSdkValidator = (
  provider: jsonSchemaValidator = new AjvJsonSchemaValidator()
) => {
  const validators = new Map<string, JsonSchemaValidator<unknown>>();
  return {
    clear() {
      validators.clear();
    },
    getValidator<T>(schema: JsonSchemaType): JsonSchemaValidator<T> {
      let key: string;
      try {
        key = JSON.stringify(schema);
      } catch {
        return provider.getValidator<T>(schema);
      }
      const existing = validators.get(key);
      if (existing) {
        return existing as JsonSchemaValidator<T>;
      }
      const validator = provider.getValidator<T>(schema);
      if (validators.size >= 78) {
        const oldest = validators.keys().next().value;
        if (oldest !== undefined) {
          validators.delete(oldest);
        }
      }
      validators.set(key, validator);
      return validator;
    },
  };
};
