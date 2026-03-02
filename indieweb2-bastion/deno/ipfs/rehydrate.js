// Deno stub: rehydrate SurrealDB snapshot from IPFS CID
const [cid] = Deno.args;
if (!cid) {
  console.error("Usage: rehydrate.js <ipfs://CID>");
  Deno.exit(2);
}
console.log("Would fetch snapshot from", cid);
// TODO: ipfs cat + surreal import
