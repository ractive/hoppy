/**
 * Shared helpers for hoppy E2E lifecycle test books.
 *
 * Provides a typed wrapper around spawning the hoppy CLI binary,
 * capturing JSON/text output, and managing cleanup of test resources.
 */

import { resolve } from "path";

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

/** Path to the hoppy binary (built by cargo). */
const hoppyBin = resolve(import.meta.dir, "../target/debug/hoppy");

/**
 * Run a hoppy command with `--format json` and return parsed output.
 *
 * @param args - CLI arguments (e.g. ["pull-zone", "create", "--name", "foo"])
 * @param timeoutMs - per-command timeout (default 30s for live API)
 */
export function hoppy(args: string[], timeoutMs = 30_000): HoppyResult {
  const proc = Bun.spawnSync([hoppyBin, "--format", "json", ...args], {
    env: { ...process.env },
    timeout: timeoutMs,
  });

  const stdout = proc.stdout.toString();
  const stderr = proc.stderr.toString();
  const exitCode = proc.exitCode;

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
  const proc = Bun.spawnSync([hoppyBin, ...args], {
    env: { ...process.env },
    timeout: timeoutMs,
  });

  return {
    json: null,
    stdout: proc.stdout.toString(),
    stderr: proc.stderr.toString(),
    exitCode: proc.exitCode,
  };
}

/** Generate a unique resource name with a timestamp to avoid collisions. */
export function testName(prefix = "hoppy-e2e"): string {
  return `${prefix}-${Date.now()}`;
}

/**
 * Cleanup registry — collects cleanup functions and runs them in reverse
 * order. Use with `afterAll(() => runCleanups())`.
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
  for (const fn of cleanupFns.reverse()) {
    try {
      fn();
    } catch {
      // Best-effort — don't let one cleanup failure block others
    }
  }
  cleanupFns.length = 0;
}
