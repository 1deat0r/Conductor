export type JsonInputErrorKind =
  | 'duplicate_property'
  | 'input_too_large'
  | 'invalid_json'
  | 'invalid_limits'
  | 'invalid_number'
  | 'negative_zero'
  | 'nesting_limit'
  | 'unsafe_number';

export class JsonInputError extends Error {
  constructor(readonly kind: JsonInputErrorKind) {
    super(kind);
    this.name = 'JsonInputError';
  }
}

export interface JsonInputLimits {
  /** Maximum UTF-8 byte length accepted from the trusted transport or caller. */
  maxBytes: number;
  /** Maximum nested arrays/objects. The shared parser ceiling is 128. */
  maxDepth: number;
}

export type JsonInputValue = null | boolean | number | string | JsonInputValue[] | { [key: string]: JsonInputValue };

const MAX_JSON_DEPTH = 128;
const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;
const NUMBER_TOKEN = /-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/y;

class JsonInputScanner {
  private position = 0;

  constructor(private readonly input: string, private readonly maxDepth: number) {}

  scan(): void {
    this.skipWhitespace();
    this.scanValue(0);
    this.skipWhitespace();
    if (this.position !== this.input.length) this.fail('invalid_json');
  }

  private scanValue(depth: number): void {
    const next = this.input[this.position];
    if (next === '"') {
      this.scanString();
      return;
    }
    if (next === '{') {
      const nestedDepth = depth + 1;
      if (nestedDepth > this.maxDepth) this.fail('nesting_limit');
      this.scanObject(nestedDepth);
      return;
    }
    if (next === '[') {
      const nestedDepth = depth + 1;
      if (nestedDepth > this.maxDepth) this.fail('nesting_limit');
      this.scanArray(nestedDepth);
      return;
    }
    if (next === 't') {
      this.scanLiteral('true');
      return;
    }
    if (next === 'f') {
      this.scanLiteral('false');
      return;
    }
    if (next === 'n') {
      this.scanLiteral('null');
      return;
    }
    if (next === '-' || (next !== undefined && next >= '0' && next <= '9')) {
      this.scanNumber();
      return;
    }
    this.fail('invalid_json');
  }

  private scanObject(depth: number): void {
    this.position += 1;
    this.skipWhitespace();
    if (this.consume('}')) return;

    const propertyNames = new Set<string>();
    while (true) {
      if (this.input[this.position] !== '"') this.fail('invalid_json');
      const propertyName = this.scanString();
      if (propertyNames.has(propertyName)) this.fail('duplicate_property');
      propertyNames.add(propertyName);

      this.skipWhitespace();
      if (!this.consume(':')) this.fail('invalid_json');
      this.skipWhitespace();
      this.scanValue(depth);
      this.skipWhitespace();

      if (this.consume('}')) return;
      if (!this.consume(',')) this.fail('invalid_json');
      this.skipWhitespace();
    }
  }

  private scanArray(depth: number): void {
    this.position += 1;
    this.skipWhitespace();
    if (this.consume(']')) return;

    while (true) {
      this.scanValue(depth);
      this.skipWhitespace();
      if (this.consume(']')) return;
      if (!this.consume(',')) this.fail('invalid_json');
      this.skipWhitespace();
    }
  }

  private scanString(): string {
    const start = this.position;
    this.position += 1;
    let closed = false;

    while (this.position < this.input.length) {
      const code = this.input.charCodeAt(this.position);
      const character = this.input[this.position];
      if (character === '"') {
        this.position += 1;
        closed = true;
        break;
      }
      if (character === '\\') {
        this.position += 1;
        const escape = this.input[this.position];
        if (escape === 'u') {
          const hex = this.input.slice(this.position + 1, this.position + 5);
          if (!/^[0-9a-fA-F]{4}$/.test(hex)) this.fail('invalid_json');
          this.position += 5;
          continue;
        }
        if (escape === undefined || !'"\\/bfnrt'.includes(escape)) this.fail('invalid_json');
        this.position += 1;
        continue;
      }
      if (code < 0x20) this.fail('invalid_json');
      this.position += 1;
    }

    if (!closed) this.fail('invalid_json');
    let decoded: unknown;
    try {
      decoded = JSON.parse(this.input.slice(start, this.position)) as unknown;
    } catch {
      this.fail('invalid_json');
    }
    if (typeof decoded !== 'string' || !hasUnicodeScalarValues(decoded)) this.fail('invalid_json');
    return decoded;
  }

  private scanNumber(): void {
    NUMBER_TOKEN.lastIndex = this.position;
    const match = NUMBER_TOKEN.exec(this.input);
    if (!match) this.fail('invalid_json');
    const token = match[0];
    this.position += token.length;

    const value = Number(token);
    if (!Number.isFinite(value)) this.fail('invalid_number');
    if (Object.is(value, -0)) this.fail('negative_zero');
    if (!Number.isSafeInteger(value) || Math.abs(value) > MAX_SAFE_INTEGER) this.fail('unsafe_number');
  }

  private scanLiteral(literal: 'true' | 'false' | 'null'): void {
    if (!this.input.startsWith(literal, this.position)) this.fail('invalid_json');
    this.position += literal.length;
  }

  private skipWhitespace(): void {
    while (this.position < this.input.length) {
      const character = this.input[this.position];
      if (character !== ' ' && character !== '\t' && character !== '\n' && character !== '\r') return;
      this.position += 1;
    }
  }

  private consume(character: string): boolean {
    if (this.input[this.position] !== character) return false;
    this.position += 1;
    return true;
  }

  private fail(kind: JsonInputErrorKind): never {
    throw new JsonInputError(kind);
  }
}

function hasUnicodeScalarValues(value: string): boolean {
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!Number.isFinite(next) || next < 0xdc00 || next > 0xdfff) return false;
      index += 1;
    } else if (code >= 0xdc00 && code <= 0xdfff) {
      return false;
    }
  }
  return true;
}

/**
 * Parses JSON for command-contract input. Callers must pass a byte bound from
 * their trusted transport or operation policy; no unbounded default is used.
 * Duplicate decoded property names, non-safe integers, invalid Unicode,
 * negative zero, and nesting beyond the caller's limit are rejected before
 * callers can validate a schema or calculate a durable identity.
 */
export function parseJsonInput(input: string, limits: JsonInputLimits): JsonInputValue {
  if (
    !Number.isSafeInteger(limits.maxBytes) ||
    limits.maxBytes < 0 ||
    !Number.isSafeInteger(limits.maxDepth) ||
    limits.maxDepth < 0 ||
    limits.maxDepth > MAX_JSON_DEPTH
  ) {
    throw new JsonInputError('invalid_limits');
  }
  if (input.length > limits.maxBytes || new TextEncoder().encode(input).byteLength > limits.maxBytes) {
    throw new JsonInputError('input_too_large');
  }

  new JsonInputScanner(input, limits.maxDepth).scan();
  try {
    return JSON.parse(input) as JsonInputValue;
  } catch {
    throw new JsonInputError('invalid_json');
  }
}
