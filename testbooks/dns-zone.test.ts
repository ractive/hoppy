/**
 * DNS Zone and Record lifecycle test book.
 *
 * Exercises:
 *   Zone:   create → get → list → update → verify update → delete
 *   Record: create zone → add A record → list records → update record → verify → delete record → delete zone
 *
 * Requires BUNNY_API_KEY in the environment.
 */

import { describe, it, expect, afterAll } from "bun:test";
import { hoppy, hoppyRaw, testName, onCleanupDelete, runCleanups } from "./helpers.ts";

describe("DNS Zone Lifecycle", () => {
  let id: string;
  const domain = `${testName("hoppy-e2e")}.test`;

  afterAll(() => runCleanups());

  it("create", () => {
    const { json, exitCode } = hoppy([
      "dns", "zone", "create",
      "--domain", domain,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Domain"]).toBe(domain);

    id = String(json!["Id"]);
    onCleanupDelete(["dns", "zone", "delete", "--id", id]);

    expect(json).toMatchSnapshot({
      Id: expect.any(Number),
      Domain: expect.any(String),
      DateCreated: expect.any(String),
      DateModified: expect.any(String),
    });
  });

  it("get", () => {
    const { json, exitCode } = hoppy(["dns", "zone", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Domain"]).toBe(domain);
    expect(json!["Records"]).toEqual(expect.any(Array));
  });

  it("list", () => {
    const { json, exitCode } = hoppy(["dns", "zone", "list"]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json!["Items"] as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((z) => String(z["Id"]) === id)).toBe(true);
  });

  it("update", () => {
    const { exitCode } = hoppyRaw([
      "dns", "zone", "update",
      "--id", id,
      "--logging-enabled", "true",
      "--soa-email", "admin@example.com",
    ]);

    expect(exitCode).toBe(0);
  });

  it("verify update", () => {
    const { json, exitCode } = hoppy(["dns", "zone", "get", "--id", id]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["LoggingEnabled"]).toBe(true);
    expect(json!["SoaEmail"]).toBe("admin@example.com");
  });

  it("delete", () => {
    const { exitCode } = hoppyRaw([
      "dns", "zone", "delete", "--id", id, "--yes",
    ]);

    expect(exitCode).toBe(0);
    id = ""; // prevent afterAll double-delete
  });
});

describe("DNS Record Lifecycle", () => {
  let zoneId: string;
  let recordId: string;
  const domain = `${testName("hoppy-e2e-rec")}.test`;

  afterAll(() => runCleanups());

  it("create zone", () => {
    const { json, exitCode } = hoppy([
      "dns", "zone", "create",
      "--domain", domain,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    zoneId = String(json!["Id"]);
    onCleanupDelete(["dns", "zone", "delete", "--id", zoneId]);
  });

  it("add A record", () => {
    const { json, exitCode } = hoppy([
      "dns", "record", "add",
      "--zone-id", zoneId,
      "--type", "A",
      "--name", "test",
      "--value", "1.2.3.4",
      "--ttl", "300",
      "--comment", "e2e test record",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Type"]).toEqual(expect.any(Number));
    expect(json!["Name"]).toBe("test");
    expect(json!["Value"]).toBe("1.2.3.4");
    expect(json!["Ttl"]).toBe(300);

    recordId = String(json!["Id"]);
    onCleanupDelete(["dns", "record", "delete", "--zone-id", zoneId, "--record-id", recordId]);

    expect(json).toMatchSnapshot({
      Id: expect.any(Number),
      Type: expect.any(Number),
      Name: expect.any(String),
      Value: expect.any(String),
      Ttl: expect.any(Number),
      Comment: expect.any(String),
    });
  });

  it("list records", () => {
    const { json, exitCode } = hoppy([
      "dns", "record", "list",
      "--zone-id", zoneId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const records = json as unknown as Array<Record<string, unknown>>;
    expect(records).toBeArray();
    expect(records.some((r) => String(r["Id"]) === recordId)).toBe(true);
  });

  it("update record", () => {
    const { exitCode } = hoppyRaw([
      "dns", "record", "update",
      "--zone-id", zoneId,
      "--record-id", recordId,
      "--type", "A",
      "--value", "5.6.7.8",
    ]);

    expect(exitCode).toBe(0);
  });

  it("verify record update", () => {
    const { json, exitCode } = hoppy([
      "dns", "record", "list",
      "--zone-id", zoneId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const records = json as unknown as Array<Record<string, unknown>>;
    const updated = records.find((r) => String(r["Id"]) === recordId);
    expect(updated).toBeDefined();
    expect(updated!["Value"]).toBe("5.6.7.8");
  });

  it("delete record", () => {
    const { exitCode } = hoppyRaw([
      "dns", "record", "delete",
      "--zone-id", zoneId,
      "--record-id", recordId,
      "--yes",
    ]);

    expect(exitCode).toBe(0);
    recordId = ""; // prevent afterAll double-delete
  });

  it("delete zone", () => {
    const { exitCode } = hoppyRaw([
      "dns", "zone", "delete", "--id", zoneId, "--yes",
    ]);

    expect(exitCode).toBe(0);
    zoneId = ""; // prevent afterAll double-delete
  });
});

describe("DNS Record Types", () => {
  let zoneId: string;
  const domain = `${testName("hoppy-e2e-mx")}.test`;

  afterAll(() => runCleanups());

  it("create zone", () => {
    const { json, exitCode } = hoppy([
      "dns", "zone", "create",
      "--domain", domain,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));

    zoneId = String(json!["Id"]);
    onCleanupDelete(["dns", "zone", "delete", "--id", zoneId]);
  });

  it("add MX record with priority", () => {
    const { json, exitCode } = hoppy([
      "dns", "record", "add",
      "--zone-id", zoneId,
      "--type", "MX",
      "--name", "mail",
      "--value", "mail.example.com",
      "--ttl", "300",
      "--priority", "10",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Priority"]).toEqual(expect.any(Number));
  });

  it("delete zone", () => {
    const { exitCode } = hoppyRaw([
      "dns", "zone", "delete", "--id", zoneId, "--yes",
    ]);

    expect(exitCode).toBe(0);
    zoneId = ""; // prevent afterAll double-delete
  });
});
