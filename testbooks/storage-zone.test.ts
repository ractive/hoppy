/**
 * Storage Zone lifecycle test book.
 *
 * Exercises:
 *   1. Storage Zone Lifecycle: create → get → list → update → verify → delete
 *   2. Storage File Operations: create zone → upload → ls → download → verify → rm → delete zone
 *
 * Requires BUNNY_API_KEY in the environment.
 */

import { describe, it, expect, afterAll } from "bun:test";
import { tmpdir } from "os";
import { join } from "path";
import { rmSync } from "fs";
import { hoppy, hoppyRaw, testName, onCleanupDelete, onCleanup, runCleanups } from "./helpers.ts";

describe("Storage Zone Lifecycle", () => {
  let id: string;
  // Storage zone names must be lowercase alphanumeric — strip hyphens.
  const name = testName("hoppye2e").replace(/-/g, "");

  afterAll(() => runCleanups());

  it("create", () => {
    const { json, exitCode } = hoppy([
      "storage-zone", "create",
      "--name", name,
      "--region", "DE",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Name"]).toBe(name);

    id = String(json!["Id"]);
    onCleanupDelete(["storage-zone", "delete", "--id", id]);

    expect(json).toMatchSnapshot({
      Id: expect.any(Number),
      Name: expect.any(String),
      Password: expect.any(String),
      ReadOnlyPassword: expect.any(String),
      UserId: expect.any(String),
      DateModified: expect.any(String),
    });
  });

  it("get", () => {
    const { json, exitCode } = hoppy(["storage-zone", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Name"]).toBe(name);
    expect(json!["Region"]).toBe("DE");
  });

  it("list", () => {
    const { json, exitCode } = hoppy(["storage-zone", "list"]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json!["Items"] as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((z) => String(z["Id"]) === id)).toBe(true);
  });

  it("update", () => {
    const { exitCode } = hoppyRaw([
      "storage-zone", "update",
      "--id", id,
      "--rewrite-404-to-200", "true",
      "--custom-404-file-path", "/custom-404.html",
    ]);

    expect(exitCode).toBe(0);
  });

  it("verify update", () => {
    const { json, exitCode } = hoppy(["storage-zone", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json!["Rewrite404To200"]).toBe(true);
    expect(json!["Custom404FilePath"]).toBe("/custom-404.html");
  });

  it("delete", () => {
    const { exitCode } = hoppyRaw([
      "storage-zone", "delete", "--id", id, "--yes",
    ]);

    expect(exitCode).toBe(0);
    id = ""; // prevent afterAll double-delete
  });
});

describe("Storage File Operations", () => {
  let zoneName: string;
  let zoneId: string;

  afterAll(() => runCleanups());

  it("create zone for file ops", () => {
    zoneName = testName("hoppye2e").replace(/-/g, "");

    const { json, exitCode } = hoppy([
      "storage-zone", "create",
      "--name", zoneName,
      "--region", "DE",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    zoneId = String(json!["Id"]);
    onCleanupDelete(["storage-zone", "delete", "--id", zoneId]);
  });

  it("upload file", async () => {
    const uploadPath = join(tmpdir(), `hoppy-e2e-upload-${Date.now()}.txt`);
    const content = "hello from hoppy e2e test";

    await Bun.write(uploadPath, content);
    onCleanup(() => {
      try { rmSync(uploadPath); } catch { /* best-effort */ }
    });

    const { exitCode } = hoppyRaw([
      "storage", "upload",
      "--zone", zoneName,
      "--remote-path", "/test.txt",
      "--file", uploadPath,
    ]);

    expect(exitCode).toBe(0);
  });

  it("ls", () => {
    const { json, exitCode } = hoppy([
      "storage", "ls",
      "--zone", zoneName,
      "--path", "/",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    // ls returns a JSON array of file entries directly
    const entries = json as unknown as Array<Record<string, unknown>>;
    expect(entries).toBeArray();
    expect(entries.some((e) => e["ObjectName"] === "test.txt")).toBe(true);
  });

  it("download and verify", async () => {
    const downloadPath = join(tmpdir(), `hoppy-e2e-download-${Date.now()}.txt`);
    onCleanup(() => {
      try { rmSync(downloadPath); } catch { /* best-effort */ }
    });

    const { exitCode } = hoppyRaw([
      "storage", "download",
      "--zone", zoneName,
      "--remote-path", "/test.txt",
      "--output", downloadPath,
    ]);

    expect(exitCode).toBe(0);

    const downloaded = await Bun.file(downloadPath).text();
    expect(downloaded).toBe("hello from hoppy e2e test");
  });

  it("rm", () => {
    const { exitCode } = hoppyRaw([
      "storage", "rm",
      "--zone", zoneName,
      "--remote-path", "/test.txt",
      "--yes",
    ]);

    expect(exitCode).toBe(0);
  });

  it("delete zone", () => {
    const { exitCode } = hoppyRaw([
      "storage-zone", "delete", "--id", zoneId, "--yes",
    ]);

    expect(exitCode).toBe(0);
    zoneId = ""; // prevent afterAll double-delete
  });
});
