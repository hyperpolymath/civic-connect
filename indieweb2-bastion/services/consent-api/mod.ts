// SPDX-License-Identifier: PMPL-1.0-or-later
// Consent API - IndieWeb2 Bastion
//
// Receives and manages user consent preferences from WordPress
// and other IndieWeb clients. Integrates with GraphQL DNS API
// and SurrealDB for storage.

import { serve } from "https://deno.land/std@0.218.0/http/server.ts";
// deno-lint-ignore-file no-explicit-any
import { Surreal } from "https://deno.land/x/surrealdb@v1.0.0/mod.ts";

// Content integrity hashing (CPR-009: BLAKE3 per CRYPTO-POLICY.adoc)
// Uses SubtleCrypto SHA-256 as interim until BLAKE3 WASM module is integrated.
// TODO(CPR-009): Replace with BLAKE3 via https://deno.land/x/blake3 or Rust WASM.
// TODO(CPR-005): Add Ed448+Dilithium5 hybrid signing of consent records via Rust WASM.
//   See: odns-rs/common/src/signatures.rs for reference implementation.
//   Path: Build consent-crypto WASM crate → import in Deno → sign records on store.
async function contentHash(data: string): Promise<string> {
  const encoded = new TextEncoder().encode(data);
  const hashBuffer = await crypto.subtle.digest("SHA-256", encoded);
  return Array.from(new Uint8Array(hashBuffer))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

// Consent preference types
export interface ConsentPreferences {
  identity: string;          // WordPress user identity (URL or email)
  telemetry: "on" | "off";   // Allow telemetry collection
  indexing: "on" | "off";    // Allow search engine indexing
  webmentions: "on" | "off"; // Allow webmention ingress
  dnsOperations: "on" | "off"; // Allow DNS record operations
  manifestRef?: string;      // IPFS reference to full consent manifest
  timestamp: string;         // ISO 8601 timestamp
  source: string;            // Origin (e.g., "wordpress://example.com")
}

export interface ConsentRecord extends ConsentPreferences {
  id: string;
  createdAt: string;
  updatedAt: string;
  version: number;
}

// SurrealDB client
let db: Surreal | null = null;

async function initDatabase(): Promise<Surreal> {
  if (db) return db;

  const dbUrl = Deno.env.get("SURREALDB_URL") || "ws://localhost:8000/rpc";
  const dbNs = Deno.env.get("SURREALDB_NS") || "indieweb2";
  const dbName = Deno.env.get("SURREALDB_DB") || "consent";

  db = new Surreal();
  await db.connect(dbUrl);
  await db.use({ ns: dbNs, db: dbName });

  // Initialize schema
  await db.query(`
    DEFINE TABLE IF NOT EXISTS consent SCHEMAFULL;
    DEFINE FIELD IF NOT EXISTS identity ON consent TYPE string;
    DEFINE FIELD IF NOT EXISTS telemetry ON consent TYPE string;
    DEFINE FIELD IF NOT EXISTS indexing ON consent TYPE string;
    DEFINE FIELD IF NOT EXISTS webmentions ON consent TYPE string;
    DEFINE FIELD IF NOT EXISTS dnsOperations ON consent TYPE string ASSERT $value IN ["on", "off"];
    DEFINE FIELD IF NOT EXISTS manifestRef ON consent TYPE option<string>;
    DEFINE FIELD IF NOT EXISTS timestamp ON consent TYPE datetime;
    DEFINE FIELD IF NOT EXISTS source ON consent TYPE string;
    DEFINE FIELD IF NOT EXISTS createdAt ON consent TYPE datetime VALUE time::now();
    DEFINE FIELD IF NOT EXISTS updatedAt ON consent TYPE datetime VALUE time::now();
    DEFINE FIELD IF NOT EXISTS version ON consent TYPE int DEFAULT 1;
    DEFINE FIELD IF NOT EXISTS contentHash ON consent TYPE option<string>;
    DEFINE INDEX IF NOT EXISTS identity_idx ON consent COLUMNS identity UNIQUE;
  `);

  console.log("✓ Connected to SurrealDB");
  return db;
}

// Store consent preferences
async function storeConsent(prefs: ConsentPreferences): Promise<ConsentRecord> {
  const db = await initDatabase();

  // Check if consent already exists for this identity
  const existing = await db.query<ConsentRecord[]>(
    "SELECT * FROM consent WHERE identity = $identity",
    { identity: prefs.identity }
  );

  // Compute content integrity hash (CPR-009)
  const hashInput = `${prefs.identity}:${prefs.telemetry}:${prefs.indexing}:${prefs.webmentions}:${prefs.dnsOperations}:${prefs.timestamp}`;
  const hash = await contentHash(hashInput);

  if (existing && existing.length > 0) {
    // Update existing record (parameterized to prevent injection)
    const record = existing[0];
    const updated = await db.query<ConsentRecord[]>(
      `UPDATE type::thing("consent", $recordId) SET
        telemetry = $telemetry,
        indexing = $indexing,
        webmentions = $webmentions,
        dnsOperations = $dnsOperations,
        manifestRef = $manifestRef,
        timestamp = $timestamp,
        source = $source,
        contentHash = $contentHash,
        updatedAt = time::now(),
        version = version + 1`,
      { ...prefs, contentHash: hash, recordId: record.id }
    );
    return updated[0];
  } else {
    // Create new record
    const created = await db.create<ConsentRecord>("consent", {
      ...prefs,
      contentHash: hash,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      version: 1,
    });
    return created[0];
  }
}

// Get consent for identity
async function getConsent(identity: string): Promise<ConsentRecord | null> {
  const db = await initDatabase();
  const results = await db.query<ConsentRecord[]>(
    "SELECT * FROM consent WHERE identity = $identity",
    { identity }
  );
  return results && results.length > 0 ? results[0] : null;
}

// Check if operation is allowed by consent
export async function checkConsentForOperation(
  identity: string,
  operation: "telemetry" | "indexing" | "webmentions" | "dnsOperations"
): Promise<boolean> {
  const consent = await getConsent(identity);
  if (!consent) {
    // No consent record - use defaults from Nickel policy
    const defaults: Record<string, string> = {
      telemetry: "off",
      indexing: "on",
      webmentions: "on",
      dnsOperations: "off",
    };
    return defaults[operation] === "on";
  }
  return consent[operation] === "on";
}

// Revoke all consent for identity
async function revokeConsent(identity: string): Promise<void> {
  const db = await initDatabase();
  await db.query("DELETE FROM consent WHERE identity = $identity", { identity });
}

// HTTP request handler
async function handler(req: Request): Promise<Response> {
  const url = new URL(req.url);
  const path = url.pathname;

  // CORS headers — restrict to configured origins (not permissive)
  const allowedOrigins = (Deno.env.get("ALLOWED_ORIGINS") || "https://localhost").split(",");
  const origin = req.headers.get("Origin") || "";
  const corsOrigin = allowedOrigins.includes(origin) ? origin : allowedOrigins[0];

  const headers: Record<string, string> = {
    "Access-Control-Allow-Origin": corsOrigin,
    "Access-Control-Allow-Methods": "GET, POST, DELETE, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type, Authorization",
    "Content-Type": "application/json",
    "Vary": "Origin",
  };

  if (req.method === "OPTIONS") {
    return new Response(null, { status: 204, headers });
  }

  try {
    // POST /consent - Store consent preferences
    if (path === "/consent" && req.method === "POST") {
      const prefs = await req.json() as ConsentPreferences;

      // Validate required fields
      if (!prefs.identity || !prefs.telemetry || !prefs.indexing) {
        return new Response(
          JSON.stringify({ error: "Missing required fields: identity, telemetry, indexing" }),
          { status: 400, headers }
        );
      }

      // Set defaults for optional fields
      prefs.webmentions = prefs.webmentions || "on";
      prefs.dnsOperations = prefs.dnsOperations || "off";
      prefs.timestamp = prefs.timestamp || new Date().toISOString();

      const record = await storeConsent(prefs);
      return new Response(JSON.stringify(record), { status: 201, headers });
    }

    // GET /consent/:identity - Get consent for identity
    if (path.startsWith("/consent/") && req.method === "GET") {
      const identity = decodeURIComponent(path.substring(9));
      const consent = await getConsent(identity);

      if (!consent) {
        return new Response(
          JSON.stringify({ error: "Consent not found", identity }),
          { status: 404, headers }
        );
      }

      return new Response(JSON.stringify(consent), { status: 200, headers });
    }

    // POST /consent/:identity/check - Check if operation is allowed
    if (path.match(/^\/consent\/[^/]+\/check$/) && req.method === "POST") {
      const identity = decodeURIComponent(path.split("/")[2]);
      const { operation } = await req.json() as { operation: string };

      if (!["telemetry", "indexing", "webmentions", "dnsOperations"].includes(operation)) {
        return new Response(
          JSON.stringify({ error: "Invalid operation" }),
          { status: 400, headers }
        );
      }

      const allowed = await checkConsentForOperation(
        identity,
        operation as "telemetry" | "indexing" | "webmentions" | "dnsOperations"
      );

      return new Response(
        JSON.stringify({ identity, operation, allowed }),
        { status: 200, headers }
      );
    }

    // DELETE /consent/:identity - Revoke consent
    if (path.startsWith("/consent/") && req.method === "DELETE") {
      const identity = decodeURIComponent(path.substring(9));
      await revokeConsent(identity);
      return new Response(
        JSON.stringify({ message: "Consent revoked", identity }),
        { status: 200, headers }
      );
    }

    // GET /health - Health check
    if (path === "/health" && req.method === "GET") {
      return new Response(JSON.stringify({ status: "OK" }), { status: 200, headers });
    }

    // 404 Not Found
    return new Response(
      JSON.stringify({ error: "Not Found" }),
      { status: 404, headers }
    );
  } catch (error) {
    console.error("Error handling request:", error);
    return new Response(
      JSON.stringify({ error: error instanceof Error ? error.message : String(error) }),
      { status: 500, headers }
    );
  }
}

// Start server
if (import.meta.main) {
  const port = parseInt(Deno.env.get("PORT") || "8082");
  console.log(`🔒 Consent API server starting on http://localhost:${port}`);
  console.log(`   - POST /consent - Store consent preferences`);
  console.log(`   - GET /consent/:identity - Get consent for identity`);
  console.log(`   - POST /consent/:identity/check - Check operation permission`);
  console.log(`   - DELETE /consent/:identity - Revoke consent`);

  await serve(handler, { port });
}
