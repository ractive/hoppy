/**
 * Stream Library and Collection lifecycle test book.
 *
 * Exercises:
 *   Library:    create → get → list → update → verify update → delete
 *   Collection: create library → create collection → get → list → update → verify → delete collection → delete library
 *
 * Requires BUNNY_API_KEY in the environment.
 * Note: video upload/fetch is deferred — not tested here.
 */

import { describe, it, expect, afterAll } from "bun:test";
import { hoppy, hoppyRaw, testName, onCleanupDelete, runCleanups } from "./helpers.ts";

describe("Stream Library Lifecycle", () => {
  let id: string;
  const name = testName("hoppy-e2e-lib");

  afterAll(() => runCleanups());

  it("create", () => {
    const { json, exitCode } = hoppy([
      "stream", "library", "create",
      "--name", name,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Name"]).toBe(name);

    id = String(json!["Id"]);
    onCleanupDelete(["stream", "library", "delete", "--id", id]);

    expect(json).toMatchSnapshot({
      Id: expect.any(Number),
      Name: expect.any(String),
      VideoCount: expect.any(Number),
      TrafficUsage: expect.any(Number),
      StorageUsage: expect.any(Number),
      DateCreated: expect.any(String),
      PullZoneId: expect.any(Number),
      StorageZoneId: expect.any(Number),
    });
  });

  it("get", () => {
    const { json, exitCode } = hoppy(["stream", "library", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toBe(Number(id));
    expect(json!["Name"]).toBe(name);
  });

  it("list", () => {
    const { json, exitCode } = hoppy(["stream", "library", "list"]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json!["Items"] as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((l) => String(l["Id"]) === id)).toBe(true);
  });

  it("update", () => {
    const { json, exitCode } = hoppy([
      "stream", "library", "update",
      "--id", id,
      "--name", `${name}-upd`,
      "--allow-direct-play", "true",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("verify update", () => {
    const { json, exitCode } = hoppy(["stream", "library", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Name"]).toBe(`${name}-upd`);
    expect(json!["AllowDirectPlay"]).toBe(true);
  });

  it("delete", () => {
    const { exitCode } = hoppyRaw([
      "stream", "library", "delete", "--id", id, "--yes",
    ]);

    expect(exitCode).toBe(0);
    id = ""; // prevent afterAll double-delete
  });
});

describe("Stream Collection Lifecycle", () => {
  let libraryId: string;
  let collectionId: string;
  const libName = testName("hoppy-e2e-clib");
  const collName = testName("hoppy-e2e-coll");

  afterAll(() => runCleanups());

  it("create library", () => {
    const { json, exitCode } = hoppy([
      "stream", "library", "create",
      "--name", libName,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    libraryId = String(json!["Id"]);
    onCleanupDelete(["stream", "library", "delete", "--id", libraryId]);
  });

  it("create collection", () => {
    const { json, exitCode } = hoppy([
      "stream", "collection", "create",
      "--library-id", libraryId,
      "--name", collName,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Guid"]).toEqual(expect.any(String));
    expect(json!["Name"]).toBe(collName);

    collectionId = String(json!["Guid"]);
    onCleanupDelete([
      "stream", "collection", "delete",
      "--library-id", libraryId,
      "--collection-id", collectionId,
    ]);

    expect(json).toMatchSnapshot({
      VideoLibraryId: expect.any(Number),
      Guid: expect.any(String),
      Name: expect.any(String),
      VideoCount: expect.any(Number),
      TotalSize: expect.any(Number),
    });
  });

  it("get collection", () => {
    const { json, exitCode } = hoppy([
      "stream", "collection", "get",
      "--library-id", libraryId,
      "--collection-id", collectionId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Guid"]).toBe(collectionId);
    expect(json!["Name"]).toBe(collName);
  });

  it("list collections", () => {
    const { json, exitCode } = hoppy([
      "stream", "collection", "list",
      "--library-id", libraryId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json!["Items"] as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((c) => c["Guid"] === collectionId)).toBe(true);
  });

  it("update collection", () => {
    const { json, exitCode } = hoppy([
      "stream", "collection", "update",
      "--library-id", libraryId,
      "--collection-id", collectionId,
      "--name", `${collName}-upd`,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("verify collection update", () => {
    const { json, exitCode } = hoppy([
      "stream", "collection", "get",
      "--library-id", libraryId,
      "--collection-id", collectionId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Name"]).toBe(`${collName}-upd`);
  });

  it("delete collection", () => {
    const { exitCode } = hoppyRaw([
      "stream", "collection", "delete",
      "--library-id", libraryId,
      "--collection-id", collectionId,
      "--yes",
    ]);

    expect(exitCode).toBe(0);
    collectionId = ""; // prevent afterAll double-delete
  });

  it("delete library", () => {
    const { exitCode } = hoppyRaw([
      "stream", "library", "delete", "--id", libraryId, "--yes",
    ]);

    expect(exitCode).toBe(0);
    libraryId = ""; // prevent afterAll double-delete
  });
});
