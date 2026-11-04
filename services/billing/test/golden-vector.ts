import { createPrivateKey, createPublicKey } from "node:crypto";
import { parseCatalog } from "../src/catalog.ts";
import { issueArtifact } from "../src/issue.ts";
import catalogJson from "../catalog.json" with { type: "json" };

const PKCS8_ED25519_PREFIX = Buffer.from("302e020100300506032b657004220420", "hex");

export function goldenVector() {
  const seed = Buffer.alloc(32, 7);
  const key = createPrivateKey({
    key: Buffer.concat([PKCS8_ED25519_PREFIX, seed]),
    format: "der",
    type: "pkcs8",
  });
  const publicKeyHex = createPublicKey(key)
    .export({ format: "der", type: "spki" })
    .subarray(12)
    .toString("hex");
  const keyId = "golden-test-key";
  const deviceId = "01JGOLDENDEVICE000000000000";
  const issued = issueArtifact(
    {
      deviceId,
      providerId: "chargebee",
      catalog: parseCatalog(JSON.stringify(catalogJson)),
      nowMs: 1_800_000_000_000,
      nonce: new Uint8Array(32).fill(9),
      artifactId: "ent_golden",
      purchases: [
        {
          providerProductId: "enzime-pro-USD-Yearly",
          kind: { type: "subscription", paidThroughMs: 1_831_536_000_000 },
          reference: "sub:AzZ1",
        },
        { providerProductId: "enzime-lifetime-USD", kind: { type: "one_time" }, reference: "inv:42" },
      ],
    },
    { keyId, key },
  );
  if (!issued) throw new Error("golden: no artifact");
  return {
    keyId,
    publicKeyHex,
    deviceId,
    artifactHex: Buffer.from(issued.artifactCbor).toString("hex"),
    issued,
  };
}
