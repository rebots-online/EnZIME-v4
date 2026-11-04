// Emit the cross-language golden vector consumed by the Rust test
// crates/entitlement/tests/ts_golden.rs. Deterministic: fixed seed, nonce,
// clock and ids (Ed25519 signatures are deterministic).
//   node scripts/golden.ts  → rewrites crates/entitlement/tests/fixtures/ts_golden.txt

import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { goldenVector } from "../test/golden-vector.ts";

const out = fileURLToPath(
  new URL("../../../crates/entitlement/tests/fixtures/ts_golden.txt", import.meta.url),
);
const v = goldenVector();
writeFileSync(out, [v.keyId, v.publicKeyHex, v.deviceId, v.artifactHex, ""].join("\n"));
console.log(`wrote ${out}`);
