/**
 * Pull Zone lifecycle test book.
 *
 * Exercises: create → get → list → update → verify update → purge → delete
 *
 * Requires BUNNY_API_KEY in the environment.
 */

import { describe, it, expect, afterAll } from "bun:test";
import { hoppy, hoppyRaw, testName, onCleanupDelete, runCleanups } from "./helpers.ts";

describe("Pull Zone Lifecycle", () => {
  let id: string;
  const name = testName("hoppy-e2e");

  afterAll(() => runCleanups());

  it("create", () => {
    const { json, exitCode } = hoppy([
      "pull-zone", "create",
      "--name", name,
      "--origin-url", "https://example.com",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Name"]).toBe(name);

    id = String(json!["Id"]);
    onCleanupDelete(["pull-zone", "delete", "--id", id]);

    // Snapshot captures the full response structure.
    // Property matchers handle fields that change per run.
    expect(json).toMatchSnapshot({
      Id: expect.any(Number),
      Name: expect.any(String),
      CacheVersion: expect.any(Number),
      Hostnames: expect.any(Array),
    });
  });

  it("get", () => {
    const { json, exitCode } = hoppy(["pull-zone", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Name"]).toBe(name);
    expect(json!["OriginUrl"]).toBe("https://example.com");
  });

  it("list", () => {
    const { json, exitCode } = hoppy(["pull-zone", "list"]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json!["Items"] as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    // Find our zone in the list (don't rely on --search which has indexing lag)
    expect(items.some((z) => String(z["Id"]) === id)).toBe(true);
  });

  it("update", () => {
    const { exitCode } = hoppyRaw([
      "pull-zone", "update",
      "--id", id,
      "--origin-url", "https://example.org",
      "--cache-expiration-time", "3600",
      "--zone-security-enabled", "true",
    ]);

    expect(exitCode).toBe(0);
  });

  it("verify update", () => {
    const { json, exitCode } = hoppy(["pull-zone", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json!["OriginUrl"]).toBe("https://example.org");
    expect(json!["CacheExpirationTime"]).toBe(3600);
    expect(json!["ZoneSecurityEnabled"]).toBe(true);
  });

  it("purge cache", () => {
    const { exitCode } = hoppyRaw(["pull-zone", "purge", "--id", id, "--cache-tag", "my-tag"]);

    expect(exitCode).toBe(0);
  });

  it("delete", () => {
    const { exitCode } = hoppyRaw([
      "pull-zone", "delete", "--id", id, "--yes",
    ]);

    expect(exitCode).toBe(0);
    id = ""; // prevent afterAll double-delete
  });
});

describe("Pull Zone Errors", () => {
  it("get non-existent id", () => {
    const { exitCode, stderr } = hoppyRaw(["pull-zone", "get", "--id", "999999999"]);

    expect(exitCode).not.toBe(0);
    expect(stderr).toContain("Error:");
  });

  it("update non-existent id", () => {
    const { exitCode, stderr } = hoppyRaw([
      "pull-zone", "update",
      "--id", "999999999",
      "--origin-url", "https://example.com",
    ]);

    expect(exitCode).not.toBe(0);
    expect(stderr).toContain("Error:");
  });
});
