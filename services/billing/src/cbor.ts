// Minimal deterministic CBOR encoder (RFC 8949) covering exactly the value
// shapes the Rust `entitlement` crate serializes with ciborium:
//   text strings, byte strings, unsigned/negative integers, null,
//   arrays, and maps with text keys in *insertion order*.
// Insertion order matters: ciborium writes struct fields in declaration
// order, and the signature covers those bytes. `test/golden.test.ts` plus the
// Rust test `crates/entitlement/tests/ts_golden.rs` pin the two encoders to
// identical output.

export type CborValue =
  | string
  | Uint8Array
  | number
  | bigint
  | null
  | CborValue[]
  | CborMap;

/** Ordered map. Use an array of pairs so key order is explicit. */
export class CborMap {
  readonly entries: ReadonlyArray<readonly [string, CborValue]>;
  constructor(entries: ReadonlyArray<readonly [string, CborValue]>) {
    this.entries = entries;
  }
}

export const map = (...entries: Array<readonly [string, CborValue]>): CborMap =>
  new CborMap(entries);

const utf8 = new TextEncoder();

function header(major: number, value: bigint, out: number[]): void {
  const m = major << 5;
  if (value < 24n) {
    out.push(m | Number(value));
  } else if (value < 0x100n) {
    out.push(m | 24, Number(value));
  } else if (value < 0x10000n) {
    out.push(m | 25, Number(value >> 8n), Number(value & 0xffn));
  } else if (value < 0x100000000n) {
    out.push(m | 26);
    for (let s = 24n; s >= 0n; s -= 8n) out.push(Number((value >> s) & 0xffn));
  } else {
    out.push(m | 27);
    for (let s = 56n; s >= 0n; s -= 8n) out.push(Number((value >> s) & 0xffn));
  }
}

function encodeInto(v: CborValue, out: number[]): void {
  if (v === null) {
    out.push(0xf6);
  } else if (typeof v === "string") {
    const b = utf8.encode(v);
    header(3, BigInt(b.length), out);
    for (const x of b) out.push(x);
  } else if (v instanceof Uint8Array) {
    header(2, BigInt(v.length), out);
    for (const x of v) out.push(x);
  } else if (typeof v === "number" || typeof v === "bigint") {
    if (typeof v === "number" && !Number.isSafeInteger(v)) {
      throw new RangeError(`cbor: non-integer or unsafe number ${v}`);
    }
    const n = BigInt(v);
    if (n >= 0n) header(0, n, out);
    else header(1, -1n - n, out);
  } else if (Array.isArray(v)) {
    header(4, BigInt(v.length), out);
    for (const item of v) encodeInto(item, out);
  } else if (v instanceof CborMap) {
    header(5, BigInt(v.entries.length), out);
    for (const [k, val] of v.entries) {
      encodeInto(k, out);
      encodeInto(val, out);
    }
  } else {
    throw new TypeError("cbor: unsupported value");
  }
}

export function encode(v: CborValue): Uint8Array {
  const out: number[] = [];
  encodeInto(v, out);
  return Uint8Array.from(out);
}
