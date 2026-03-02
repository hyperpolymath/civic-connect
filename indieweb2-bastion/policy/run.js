// Deno — verify signature, publish to IPFS, write provenance (pure JS)
// Usage: deno run --allow-read --allow-run --allow-write --unstable policy/run.js .tmp/policy.json .tmp/policy.sig.json

const [policyPath, sigBundlePath] = Deno.args;
if (!policyPath || !sigBundlePath) {
  console.error("Usage: run.js <policy.json> <policy.sig.json>");
  Deno.exit(2);
}

const policyBytes = await Deno.readFile(policyPath);
const sigBundle = JSON.parse(await Deno.readTextFile(sigBundlePath));

if (sigBundle.alg !== "Ed25519") {
  console.error("Unsupported alg:", sigBundle.alg);
  Deno.exit(2);
}

const publicKey = await crypto.subtle.importKey(
  "jwk",
  sigBundle.jwk,
  { name: "Ed25519" },
  true,
  ["verify"],
);

// base64 → Uint8Array
const signature = Uint8Array.from(
  atob(sigBundle.signature),
                                  c => c.charCodeAt(0),
);

const ok = await crypto.subtle.verify("Ed25519", publicKey, signature, policyBytes);
if (!ok) {
  console.error("Signature verification failed");
  Deno.exit(1);
}

// Publish to IPFS (local daemon)
const addCmd = new Deno.Command("ipfs", { args: ["add", "-Q", policyPath] });
const { stdout, code } = await addCmd.output();
if (code !== 0) {
  console.error("ipfs add failed");
  Deno.exit(code);
}
const cid = new TextDecoder().decode(stdout).trim();

// Git commit id
const gitCmd = new Deno.Command("git", { args: ["rev-parse", "HEAD"] });
const gitOut = await gitCmd.output();
const commitId = new TextDecoder().decode(gitOut.stdout).trim();

const prov = {
  commitId,
  createdAt: new Date().toISOString(),
  ipfsCid: cid,
  ipnsName: "",
  sbomHash: "",
  sourceId: "policy/curps/policy.json",
  targetId: `ipfs://${cid}`,
  consentHash: "",
};

await Deno.writeTextFile(".tmp/provenance.json", JSON.stringify(prov, null, 2));
console.log("Verified signature and published to IPFS. CID:", cid);
