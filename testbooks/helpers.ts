/**
 * Shared helpers for hoppy E2E lifecycle test books.
 *
 * Provides a typed wrapper around spawning the hoppy CLI binary,
 * capturing JSON/text output, and managing cleanup of test resources.
 *
 * Set CAPTURE_FIXTURES=1 to capture raw API responses from --debug
 * output and write them to the shared fixtures/ directory.
 */

import { resolve } from "path";
import { mkdirSync, writeFileSync } from "fs";

/** Result of running a hoppy command. */
export interface HoppyResult {
  /** Parsed JSON output (only when --format json was used). */
  json: Record<string, unknown> | null;
  /** Raw stdout as a string. */
  stdout: string;
  /** Raw stderr as a string. */
  stderr: string;
  /** Process exit code. */
  exitCode: number;
}

/** Path to the hoppy binary — override with HOPPY_BIN env var if needed. */
const hoppyBin =
  process.env.HOPPY_BIN ??
  resolve(
    import.meta.dir,
    process.platform === "win32"
      ? "../target/debug/hoppy.exe"
      : "../target/debug/hoppy",
  );

/** When true, pass --debug and capture raw API responses as fixture files. */
const captureFixtures = !!process.env.CAPTURE_FIXTURES;

/** Root of the shared fixtures directory. */
const fixturesDir = resolve(import.meta.dir, "../fixtures");

/**
 * Run a hoppy command with `--format json` and return parsed output.
 *
 * @param args - CLI arguments (e.g. ["pull-zone", "create", "--name", "foo"])
 * @param timeoutMs - per-command timeout (default 30s for live API)
 */
export function hoppy(args: string[], timeoutMs = 30_000): HoppyResult {
  const cliArgs = captureFixtures
    ? [hoppyBin, "--debug", "--format", "json", ...args]
    : [hoppyBin, "--format", "json", ...args];

  const proc = Bun.spawnSync(cliArgs, {
    env: { ...process.env },
    timeout: timeoutMs,
  });

  const stdout = proc.stdout.toString();
  const stderr = proc.stderr.toString();
  const exitCode = proc.exitCode;

  if (captureFixtures && exitCode === 0) {
    extractFixtures(stderr);
  }

  let json: Record<string, unknown> | null = null;
  if (stdout.trim()) {
    try {
      json = JSON.parse(stdout);
    } catch {
      // Not valid JSON — leave as null
    }
  }

  return { json, stdout, stderr, exitCode };
}

/**
 * Run a hoppy command without `--format json` (for table/text output or
 * commands that don't produce JSON, like delete).
 */
export function hoppyRaw(args: string[], timeoutMs = 30_000): HoppyResult {
  const cliArgs = captureFixtures
    ? [hoppyBin, "--debug", ...args]
    : [hoppyBin, ...args];

  const proc = Bun.spawnSync(cliArgs, {
    env: { ...process.env },
    timeout: timeoutMs,
  });

  const stderr = proc.stderr.toString();

  if (captureFixtures && proc.exitCode === 0) {
    extractFixtures(stderr);
  }

  return {
    json: null,
    stdout: proc.stdout.toString(),
    stderr,
    exitCode: proc.exitCode,
  };
}

/** Monotonic counter to guarantee uniqueness even within the same millisecond. */
let nameCounter = 0;

/** Generate a unique resource name with a timestamp and counter to avoid collisions. */
export function testName(prefix = "hoppy-e2e"): string {
  return `${prefix}-${Date.now()}-${nameCounter++}`;
}

/**
 * Cleanup registry — collects cleanup functions and runs them in reverse
 * order. Use with `afterAll(() => runCleanups())`.
 *
 * Note: this is a module-level singleton. Bun runs describe blocks
 * sequentially within a file, so each afterAll drains before the next
 * describe registers new cleanups. Do not run test files in parallel.
 */
const cleanupFns: Array<() => void> = [];

/** Register a cleanup function to run after all tests. */
export function onCleanup(fn: () => void): void {
  cleanupFns.push(fn);
}

/** Register a hoppy delete command as cleanup. */
export function onCleanupDelete(args: string[]): void {
  onCleanup(() => {
    try {
      hoppyRaw(["--yes", ...args], 15_000);
    } catch {
      // Best-effort cleanup
    }
  });
}

/** Run all registered cleanups in reverse order, then clear the list. */
export function runCleanups(): void {
  for (const fn of [...cleanupFns].reverse()) {
    try {
      fn();
    } catch {
      // Best-effort — don't let one cleanup failure block others
    }
  }
  cleanupFns.length = 0;
}

// ---------------------------------------------------------------------------
// Fixture capture (active only when CAPTURE_FIXTURES=1)
// ---------------------------------------------------------------------------

/** Map a debug >> line and <<< body to a fixture path under fixtures/. */
type FixtureRoute = {
  /** Host substring to match (e.g. "api.bunny.net"). */
  host: string;
  /** Returns relative fixture path or null to skip. */
  map: (method: string, pathname: string, hasQuery: boolean) => string | null;
};

const FIXTURE_ROUTES: FixtureRoute[] = [
  {
    // Storage API: {region}.storage.bunnycdn.com
    host: "storage.bunnycdn.com",
    map: (method, pathname) => {
      if (method === "GET" && pathname.split("/").length > 2) return "storage/storage_list_files.json";
      if (method === "PUT") return "storage/storage_upload_success.json";
      if (method === "DELETE") return "storage/storage_delete_success.json";
      return null;
    },
  },
  {
    // Stream API: video.bunnycdn.com
    host: "video.bunnycdn.com",
    map: (method, pathname) => {
      // /library/{libId}/collections/{colId}
      if (pathname.includes("/collections")) {
        if (method === "POST") return "stream/collection_create.json";
        if (method === "GET" && pathname.match(/\/collections\/[^/]+$/)) return "stream/collection_get.json";
        if (method === "GET") return "stream/collection_list_paginated.json";
        return null;
      }
      // /library/{libId}/videos/{videoId}
      if (pathname.includes("/videos")) {
        if (method === "POST") return "stream/video_create.json";
        if (method === "GET" && pathname.match(/\/videos\/[^/]+$/)) return "stream/video_get.json";
        if (method === "GET") return "stream/video_list_paginated.json";
        return null;
      }
      return null;
    },
  },
  {
    // Core + Shield + Compute + Containers API: api.bunny.net
    host: "api.bunny.net",
    map: (method, pathname, hasQuery) => {
      // Shield API: /shield/*
      if (pathname.startsWith("/shield/")) {
        if (pathname.includes("/waf/rules") && method === "POST") return "shield/waf_rule_create.json";
        if (pathname.match(/\/waf\/rules\/\d+$/) && method === "GET") return "shield/waf_rule_get.json";
        if (pathname.includes("/waf/rules") && method === "GET") return "shield/waf_rules_list.json";
        if (pathname.includes("/waf/profiles") && method === "GET") return "shield/waf_profiles_list.json";
        if (pathname.includes("/waf") && method === "POST") return "shield/waf_enable.json";
        if (pathname.includes("/rate-limiting/rules") && method === "POST") return "shield/rate_limit_rule_create.json";
        if (pathname.match(/\/rate-limiting\/rules\/\d+$/) && method === "GET") return "shield/rate_limit_rule_get.json";
        if (pathname.includes("/rate-limiting/rules") && method === "GET") return "shield/rate_limit_rules_list.json";
        if (pathname.includes("/access-lists") && method === "POST") return "shield/access_list_create.json";
        if (pathname.match(/\/access-lists\/\d+$/) && method === "GET") return "shield/access_list_get.json";
        if (pathname.includes("/access-lists") && method === "GET") return "shield/access_lists_get.json";
        if (pathname.includes("/bot-detection") && method === "GET") return "shield/bot_detection_get.json";
        if (pathname.includes("/bot-detection") && method === "POST") return "shield/bot_detection_update.json";
        if (pathname.match(/\/shield-zone\/\d+$/) && method === "GET") return "shield/shield_zone_get.json";
        if (pathname.includes("/shield-zone") && method === "GET") return "shield/shield_zones_list.json";
        return null;
      }
      // Compute API: /compute/*
      if (pathname.startsWith("/compute/")) {
        if (pathname.includes("/code") && method === "GET") return "compute/script_code_get.json";
        if (pathname.includes("/code") && method === "POST") return null; // code update returns no useful body
        if (pathname.includes("/variables") && method === "POST") return "compute/variable_add.json";
        if (pathname.match(/\/variables\/\d+$/) && method === "GET") return "compute/variable_get.json";
        if (pathname.includes("/secrets") && method === "POST") return "compute/secret_add.json";
        if (pathname.includes("/secrets") && method === "GET") return "compute/secrets_list.json";
        if (pathname.includes("/releases") && method === "GET") return "compute/releases_list.json";
        if (pathname.match(/\/script\/[^/]+$/) && method === "GET") return "compute/script_get.json";
        if (pathname.match(/\/script\/[^/]+$/) && method === "POST") return "compute/script_update.json";
        if (pathname.includes("/script") && method === "POST") return "compute/script_create.json";
        if (pathname.includes("/script") && method === "GET") return "compute/scripts_list.json";
        return null;
      }
      // Containers API: /mc/*
      if (pathname.startsWith("/mc/")) {
        // Skip — containers fixtures are complex, add mappings as needed
        return null;
      }
      // Core API
      if (pathname.startsWith("/pullzone")) {
        if (method === "POST") return "core/pullzone_create.json";
        if (method === "GET" && hasQuery) return "core/pullzone_list_paginated.json";
        if (method === "GET") return "core/pullzone_get.json";
        return null;
      }
      if (pathname.startsWith("/storagezone")) {
        if (method === "POST") return "core/storagezone_create.json";
        if (method === "GET" && hasQuery) return "core/storagezone_list_paginated.json";
        if (method === "GET") return "core/storagezone_get.json";
        return null;
      }
      if (pathname.startsWith("/dnszone")) {
        if (pathname.includes("/records") && method === "PUT") return "core/dnsrecord_add.json";
        if (method === "POST") return "core/dnszone_create.json";
        if (method === "GET" && hasQuery) return "core/dnszone_list_paginated.json";
        if (method === "GET") return "core/dnszone_get.json";
        return null;
      }
      if (pathname.startsWith("/videolibrary")) {
        if (method === "POST") return "core/videolibrary_create.json";
        if (method === "GET" && hasQuery) return "core/videolibrary_list_paginated.json";
        if (method === "GET") return "core/videolibrary_get.json";
        return null;
      }
      if (pathname.startsWith("/billing")) {
        if (method === "GET") return "core/billing_get.json";
        return null;
      }
      return null;
    },
  },
];

/**
 * Parse --debug stderr output and write captured API responses as fixture files.
 *
 * Looks for pairs of lines:
 *   >> METHOD URL
 *   <<< {"json":"body"}
 */
function extractFixtures(stderr: string): void {
  const lines = stderr.split("\n");
  let pendingMethod: string | null = null;
  let pendingUrl: URL | null = null;

  for (const line of lines) {
    if (line.startsWith(">> ")) {
      const parts = line.slice(3).split(" ", 2);
      pendingMethod = parts[0] ?? null;
      try {
        pendingUrl = parts[1] ? new URL(parts[1]) : null;
      } catch {
        pendingUrl = null;
      }
      continue;
    }

    if (line.startsWith("<<< ") && pendingMethod && pendingUrl) {
      const body = line.slice(4);
      // Only capture JSON responses
      if (!body.startsWith("{") && !body.startsWith("[")) {
        pendingMethod = null;
        pendingUrl = null;
        continue;
      }

      const fixturePath = resolveFixturePath(pendingMethod, pendingUrl);
      if (fixturePath) {
        try {
          const parsed = JSON.parse(body);
          const pretty = JSON.stringify(parsed, null, 2) + "\n";
          const fullPath = resolve(fixturesDir, fixturePath);
          mkdirSync(resolve(fullPath, ".."), { recursive: true });
          writeFileSync(fullPath, pretty);
        } catch {
          // Skip unparseable responses
        }
      }

      pendingMethod = null;
      pendingUrl = null;
    }
  }
}

/** Map a request method + URL to a fixture file path, or null to skip. */
function resolveFixturePath(method: string, url: URL): string | null {
  for (const route of FIXTURE_ROUTES) {
    if (url.host.includes(route.host)) {
      const hasQuery = url.searchParams.has("page") || url.searchParams.has("perPage");
      return route.map(method, url.pathname, hasQuery);
    }
  }
  return null;
}
