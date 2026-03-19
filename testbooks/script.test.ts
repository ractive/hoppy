/**
 * Edge Script lifecycle test book.
 *
 * Exercises:
 *   1. Script Lifecycle:   create → get → list → update → code update → code get → publish → release list → release get-active → delete
 *   2. Variable Lifecycle: create script → add variable → list variables → update variable → upsert variable → delete variable → delete script
 *   3. Secret Lifecycle:   create script → add secret → list secrets → update secret → upsert secret → delete secret → delete script
 *
 * Requires BUNNY_API_KEY in the environment.
 */

import { describe, it, expect, afterAll } from "bun:test";
import { hoppy, hoppyRaw, testName, onCleanupDelete, runCleanups } from "./helpers.ts";

describe("Edge Script Lifecycle", () => {
  let id: string;
  const name = testName("hoppy-e2e");

  afterAll(() => runCleanups());

  it("create", () => {
    const { json, exitCode } = hoppy([
      "script", "create",
      "--name", name,
      "--script-type", "1",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Name"]).toBe(name);

    id = String(json!["Id"]);
    onCleanupDelete(["script", "delete", "--id", id]);

    expect(json).toMatchSnapshot({
      Id: expect.any(Number),
      Name: expect.any(String),
    });
  });

  it("get", () => {
    const { json, exitCode } = hoppy(["script", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Name"]).toBe(name);
  });

  it("list", () => {
    const { json, exitCode } = hoppy(["script", "list"]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json!["Items"] as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((s) => String(s["Id"]) === id)).toBe(true);
  });

  it("update", () => {
    const { json, exitCode } = hoppy(["script", "update", "--id", id, "--name", `${name}-upd`]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("code update", () => {
    const { exitCode } = hoppyRaw([
      "script", "code", "update",
      "--id", id,
      "--code", "export default { async fetch(request) { return new Response('hello'); } }",
    ]);

    expect(exitCode).toBe(0);
  });

  it("code get", () => {
    const { json, exitCode } = hoppy(["script", "code", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(typeof json!["Code"]).toBe("string");
  });

  it("publish", () => {
    const { exitCode } = hoppyRaw(["script", "publish", "--id", id, "--note", "e2e test release"]);

    expect(exitCode).toBe(0);
  });

  it("release list", () => {
    const { json, exitCode } = hoppy(["script", "release", "list", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json!["Items"] as Array<Record<string, unknown>>;
    expect(items).toBeArray();
  });

  it("release get-active", () => {
    const { json, exitCode } = hoppy(["script", "release", "get-active", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("delete", () => {
    const { exitCode } = hoppyRaw(["script", "delete", "--id", id, "--yes"]);

    expect(exitCode).toBe(0);
    id = ""; // prevent afterAll double-delete
  });
});

describe("Script Variable Lifecycle", () => {
  let scriptId: string;
  let variableId: string;
  const name = testName("hoppy-e2e");

  afterAll(() => runCleanups());

  it("create script", () => {
    const { json, exitCode } = hoppy([
      "script", "create",
      "--name", name,
      "--script-type", "1",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    scriptId = String(json!["Id"]);
    onCleanupDelete(["script", "delete", "--id", scriptId]);
  });

  it("add variable", () => {
    const { json, exitCode } = hoppy([
      "script", "variable", "add",
      "--id", scriptId,
      "--name", "MY_VAR",
      "--default-value", "hello",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    variableId = String(json!["Id"]);
  });

  it("list variables", () => {
    const { json, exitCode } = hoppy([
      "script", "variable", "list",
      "--id", scriptId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json as unknown as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((v) => String(v["Id"]) === variableId)).toBe(true);
  });

  it("update variable", () => {
    const { json, exitCode } = hoppy([
      "script", "variable", "update",
      "--id", scriptId,
      "--variable-id", variableId,
      "--default-value", "updated",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("upsert variable", () => {
    const { json, exitCode } = hoppy([
      "script", "variable", "upsert",
      "--id", scriptId,
      "--name", "MY_VAR_2",
      "--default-value", "upserted",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("delete variable", () => {
    const { exitCode } = hoppyRaw([
      "script", "variable", "delete",
      "--id", scriptId,
      "--variable-id", variableId,
      "--yes",
    ]);

    expect(exitCode).toBe(0);
  });

  it("delete script", () => {
    const { exitCode } = hoppyRaw(["script", "delete", "--id", scriptId, "--yes"]);

    expect(exitCode).toBe(0);
    scriptId = ""; // prevent afterAll double-delete
  });
});

describe("Script Secret Lifecycle", () => {
  let scriptId: string;
  let secretId: string;
  const name = testName("hoppy-e2e");

  afterAll(() => runCleanups());

  it("create script", () => {
    const { json, exitCode } = hoppy([
      "script", "create",
      "--name", name,
      "--script-type", "1",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    scriptId = String(json!["Id"]);
    onCleanupDelete(["script", "delete", "--id", scriptId]);
  });

  it("add secret", () => {
    const { json, exitCode } = hoppy([
      "script", "secret", "add",
      "--id", scriptId,
      "--name", "MY_SECRET",
      "--value", "secret123",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    secretId = String(json!["Id"]);
  });

  it("list secrets", () => {
    const { json, exitCode } = hoppy([
      "script", "secret", "list",
      "--id", scriptId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json as unknown as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((s) => String(s["Id"]) === secretId)).toBe(true);
  });

  it("update secret", () => {
    const { json, exitCode } = hoppy([
      "script", "secret", "update",
      "--id", scriptId,
      "--secret-id", secretId,
      "--value", "updated-secret",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("upsert secret", () => {
    const { json, exitCode } = hoppy([
      "script", "secret", "upsert",
      "--id", scriptId,
      "--name", "MY_SECRET_2",
      "--value", "upserted-secret",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("delete secret", () => {
    const { exitCode } = hoppyRaw([
      "script", "secret", "delete",
      "--id", scriptId,
      "--secret-id", secretId,
      "--yes",
    ]);

    expect(exitCode).toBe(0);
  });

  it("delete script", () => {
    const { exitCode } = hoppyRaw(["script", "delete", "--id", scriptId, "--yes"]);

    expect(exitCode).toBe(0);
    scriptId = ""; // prevent afterAll double-delete
  });
});
