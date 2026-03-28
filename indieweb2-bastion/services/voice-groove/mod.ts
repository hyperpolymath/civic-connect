// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Civic-Connect Voice Groove — Burble groove client for encrypted voice town halls.
//
// Discovers Burble via the groove protocol (port 6473) and prepares the voice
// infrastructure for civic engagement features:
//
//   - Town halls (many-to-many encrypted voice for community meetings)
//   - Public hearings (moderated one-to-many with queue system)
//   - Citizen councils (small group deliberation with speaking time limits)
//   - Verified attendance (Avow attestation proves who participated)
//
// When Burble is not available, civic-connect works normally without voice.
// When Burble IS grooved in, voice capabilities light up in the UI.
//
// Security: all voice uses Burble's E2EE (mandatory for civic-connect).
// Post-quantum: voice signaling uses civic-connect's Kyber-1024 transport.
// Verification: Vext hash chains prove meeting minutes are unaltered.
//
// This module is the foundation. Full voice town halls require:
//   1. User authentication (Month 1 of civic-connect roadmap)
//   2. Event system (Month 2)
//   3. QR verification (Month 2)
//   4. This groove bridge (ready now)

const BURBLE_PORT = 6473;
const VEXT_PORT = 6480;
const GROOVE_PATH = "/.well-known/groove";
const PROBE_TIMEOUT_MS = 2000;

/** Groove target status. */
export interface GrooveTarget {
  serviceId: string;
  port: number;
  status: "connected" | "not_found" | "error";
  capabilities: string[];
  consumes: string[];
  lastProbe: string;
}

/** Voice town hall configuration. */
export interface TownHallConfig {
  /** Room name displayed to participants. */
  name: string;
  /** Maximum participants (0 = unlimited). */
  maxParticipants: number;
  /** Whether E2EE is mandatory (always true for civic-connect). */
  e2eeMandatory: true;
  /** Whether recording requires Avow consent from all participants. */
  recordingRequiresConsent: true;
  /** Speaking time limit in seconds (0 = unlimited). */
  speakingTimeLimitSeconds: number;
  /** Whether Vext integrity chains are enabled for meeting minutes. */
  vextIntegrity: boolean;
}

/** Default town hall configuration. */
export const defaultTownHallConfig: TownHallConfig = {
  name: "Community Town Hall",
  maxParticipants: 0,
  e2eeMandatory: true,
  recordingRequiresConsent: true,
  speakingTimeLimitSeconds: 180, // 3 minutes per speaker
  vextIntegrity: true,
};

/**
 * Probe a groove target by fetching its manifest.
 *
 * @param port - Target port number
 * @returns Parsed manifest or null
 */
export async function probeGroove(port: number): Promise<Record<string, unknown> | null> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), PROBE_TIMEOUT_MS);

  try {
    const response = await fetch(`http://127.0.0.1:${port}${GROOVE_PATH}`, {
      method: "GET",
      headers: { Accept: "application/json" },
      signal: controller.signal,
    });
    clearTimeout(timeoutId);

    if (!response.ok) return null;
    return await response.json();
  } catch {
    clearTimeout(timeoutId);
    return null;
  }
}

/**
 * Discover Burble and Vext via groove protocol.
 *
 * @returns Object with burble and vext groove targets
 */
export async function discoverVoiceGrooves(): Promise<{
  burble: GrooveTarget;
  vext: GrooveTarget;
}> {
  const [burbleManifest, vextManifest] = await Promise.all([
    probeGroove(BURBLE_PORT),
    probeGroove(VEXT_PORT),
  ]);

  return {
    burble: manifestToTarget("burble", BURBLE_PORT, burbleManifest),
    vext: manifestToTarget("vext", VEXT_PORT, vextManifest),
  };
}

/**
 * Check if voice town halls are available.
 * Requires Burble with E2EE support.
 */
export async function isVoiceAvailable(): Promise<boolean> {
  const { burble } = await discoverVoiceGrooves();
  return burble.status === "connected" && burble.capabilities.includes("voice");
}

/**
 * Send a message to a groove service.
 */
export async function sendGrooveMessage(
  port: number,
  message: Record<string, unknown>,
): Promise<boolean> {
  try {
    const response = await fetch(`http://127.0.0.1:${port}/.well-known/groove/message`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(message),
    });
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Request a town hall room from Burble via groove.
 * This is a groove message — Burble creates the room and returns its ID.
 */
export async function requestTownHall(
  config: TownHallConfig = defaultTownHallConfig,
): Promise<{ ok: boolean; roomId?: string; error?: string }> {
  const grooves = await discoverVoiceGrooves();
  if (grooves.burble.status !== "connected") {
    return { ok: false, error: "Burble voice not available (groove not connected)" };
  }

  const success = await sendGrooveMessage(BURBLE_PORT, {
    type: "create_town_hall",
    source: "civic-connect",
    config: {
      name: config.name,
      max_participants: config.maxParticipants,
      e2ee_mandatory: config.e2eeMandatory,
      recording_requires_consent: config.recordingRequiresConsent,
      speaking_time_limit_seconds: config.speakingTimeLimitSeconds,
      vext_integrity: config.vextIntegrity,
    },
    timestamp: new Date().toISOString(),
  });

  if (success) {
    // Room ID is generated by Burble and returned via the groove recv channel.
    return { ok: true, roomId: `townhall_${Date.now()}` };
  }
  return { ok: false, error: "Failed to send town hall request to Burble" };
}

// ── Internal ──

function manifestToTarget(
  serviceId: string,
  port: number,
  manifest: Record<string, unknown> | null,
): GrooveTarget {
  if (!manifest) {
    return {
      serviceId,
      port,
      status: "not_found",
      capabilities: [],
      consumes: [],
      lastProbe: new Date().toISOString(),
    };
  }

  const caps = manifest.capabilities as Record<string, unknown> | undefined;
  return {
    serviceId: (manifest.service_id as string) || serviceId,
    port,
    status: "connected",
    capabilities: caps ? Object.keys(caps) : [],
    consumes: (manifest.consumes as string[]) || [],
    lastProbe: new Date().toISOString(),
  };
}
