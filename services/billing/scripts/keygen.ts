// Generate the entitlement issuer keypair.
//   node scripts/keygen.ts [key-id]
// Prints:
//   - ENTITLEMENT_SIGNING_KEY (PKCS#8 DER, base64) → Netlify env var. SECRET.
//   - ENZIME_ENTITLEMENT_PUBKEY (raw 32-byte hex)  → app build env. Public.
// Keep an offline copy of the private key; losing it means re-issuing every
// artifact with a new key id (old artifacts keep verifying while the old
// public key stays in the app's trust list).

import { generateKeyPairSync } from "node:crypto";

const keyId = process.argv[2] ?? `enzime-entitlement-${new Date().getFullYear()}`;
const { privateKey, publicKey } = generateKeyPairSync("ed25519");
const pkcs8 = privateKey.export({ format: "der", type: "pkcs8" }).toString("base64");
// SPKI DER for Ed25519 is a fixed 12-byte prefix + 32-byte raw key.
const raw = publicKey.export({ format: "der", type: "spki" }).subarray(12).toString("hex");

console.log(`ENTITLEMENT_KEY_ID=${keyId}`);
console.log(`ENTITLEMENT_SIGNING_KEY=${pkcs8}`);
console.log("");
console.log("# App build environment (public):");
console.log(`ENZIME_ENTITLEMENT_KEY_ID=${keyId}`);
console.log(`ENZIME_ENTITLEMENT_PUBKEY=${raw}`);
