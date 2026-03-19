/**
 * Shield lifecycle test book.
 *
 * Exercises: pull zone create → shield zone create → get → get-by-pullzone →
 *            update (WAF enable) → waf profiles → waf add-rule → waf update-rule →
 *            waf get-rule → waf list-rules → waf delete-rule →
 *            rate-limit create → rate-limit update → rate-limit get →
 *            rate-limit list → rate-limit delete →
 *            access-list create → get → list → update → delete →
 *            bot-detection get → update →
 *            pull zone delete (cascades to shield zone)
 *
 * Requires BUNNY_API_KEY in the environment.
 */

import { describe, it, expect, afterAll } from "bun:test";
import { hoppy, hoppyRaw, testName, onCleanupDelete, runCleanups } from "./helpers.ts";

describe("Shield Lifecycle", () => {
  let pullZoneId: string;
  let shieldZoneId: string;
  let wafRuleId: string;
  let rateLimitId: string;
  let accessListId: string;

  const pullZoneName = testName("hoppy-e2e-shield");

  afterAll(() => runCleanups());

  it("create pull zone", () => {
    const { json, exitCode } = hoppy([
      "pull-zone", "create",
      "--name", pullZoneName,
      "--origin-url", "https://example.com",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["Id"]).toEqual(expect.any(Number));
    expect(json!["Name"]).toBe(pullZoneName);

    pullZoneId = String(json!["Id"]);
    onCleanupDelete(["pull-zone", "delete", "--id", pullZoneId]);
  });

  it("create shield zone", () => {
    const { json, exitCode } = hoppy([
      "shield", "zone", "create",
      "--pull-zone-id", pullZoneId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["shieldZoneId"]).toEqual(expect.any(Number));
    expect(json!["pullZoneId"]).toEqual(Number(pullZoneId));

    shieldZoneId = String(json!["shieldZoneId"]);

    expect(json).toMatchSnapshot({
      shieldZoneId: expect.any(Number),
      pullZoneId: expect.any(Number),
    });
  });

  it("get shield zone", () => {
    const { json, exitCode } = hoppy([
      "shield", "zone", "get",
      "--shield-zone-id", shieldZoneId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["shieldZoneId"]).toEqual(Number(shieldZoneId));
    expect(json!["pullZoneId"]).toEqual(Number(pullZoneId));
  });

  it("get shield zone by pull zone", () => {
    const { json, exitCode } = hoppy([
      "shield", "zone", "get-by-pullzone",
      "--pull-zone-id", pullZoneId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["shieldZoneId"]).toEqual(Number(shieldZoneId));
    expect(json!["pullZoneId"]).toEqual(Number(pullZoneId));
  });

  it("list shield zones", () => {
    const { json, exitCode } = hoppy(["shield", "zone", "list"]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();

    const items = json as unknown as Array<Record<string, unknown>>;
    expect(items).toBeArray();
    expect(items.some((z) => String(z["shieldZoneId"]) === shieldZoneId)).toBe(true);
  });

  it("update shield zone (enable WAF)", () => {
    const { exitCode } = hoppyRaw([
      "shield", "zone", "update",
      "--shield-zone-id", shieldZoneId,
      "--waf-enabled", "true",
    ]);

    expect(exitCode).toBe(0);
  });

  it("get WAF profiles", () => {
    const { json, exitCode } = hoppy(["shield", "waf", "profiles"]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("add WAF rule", () => {
    const { json, exitCode } = hoppy([
      "shield", "waf", "add-rule",
      "--shield-zone-id", shieldZoneId,
      "--name", "hoppy-e2e-waf-rule",
      "--action-type", "1",    // Block
      "--operator-type", "0",  // Contains
      "--severity-type", "1",  // Medium
      "--value", "test",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["id"]).toEqual(expect.any(Number));

    wafRuleId = String(json!["id"]);

    expect(json).toMatchSnapshot({
      id: expect.any(Number),
      shieldZoneId: expect.any(Number),
    });
  });

  it("update WAF rule", () => {
    const { json, exitCode } = hoppy([
      "shield", "waf", "update-rule",
      "--id", wafRuleId,
      "--name", "hoppy-e2e-waf-rule-upd",
    ]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("get WAF rule", () => {
    const { json, exitCode } = hoppy([
      "shield", "waf", "get-rule",
      "--id", wafRuleId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["id"]).toEqual(Number(wafRuleId));
    expect(json!["shieldZoneId"]).toEqual(Number(shieldZoneId));
  });

  it("list WAF rules", () => {
    const { exitCode } = hoppyRaw([
      "shield", "waf", "list-rules",
      "--shield-zone-id", shieldZoneId,
    ]);

    expect(exitCode).toBe(0);
  });

  it("delete WAF rule", () => {
    const { exitCode } = hoppyRaw([
      "shield", "waf", "delete-rule",
      "--id", wafRuleId,
      "--yes",
    ]);

    expect(exitCode).toBe(0);
    wafRuleId = ""; // prevent double-delete
  });

  it("create rate limit", () => {
    const { json, exitCode } = hoppy([
      "shield", "rate-limit", "create",
      "--shield-zone-id", shieldZoneId,
      "--name", "hoppy-e2e-rate-limit",
      "--action-type", "1",       // Block
      "--operator-type", "0",     // Contains
      "--severity-type", "1",     // Medium
      "--value", "/api",
      "--request-count", "100",
      "--counter-key-type", "0",  // IP
      "--timeframe", "60",
      "--block-time", "300",
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["id"]).toEqual(expect.any(Number));

    rateLimitId = String(json!["id"]);

    expect(json).toMatchSnapshot({
      id: expect.any(Number),
      shieldZoneId: expect.any(Number),
    });
  });

  it("update rate limit", () => {
    const { json, exitCode } = hoppy([
      "shield", "rate-limit", "update",
      "--id", rateLimitId,
      "--name", "hoppy-e2e-rate-limit-upd",
    ]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("get rate limit", () => {
    const { json, exitCode } = hoppy([
      "shield", "rate-limit", "get",
      "--id", rateLimitId,
    ]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    expect(json!["id"]).toEqual(Number(rateLimitId));
    expect(json!["shieldZoneId"]).toEqual(Number(shieldZoneId));
  });

  it("list rate limits", () => {
    const { exitCode } = hoppyRaw([
      "shield", "rate-limit", "list",
      "--shield-zone-id", shieldZoneId,
    ]);

    expect(exitCode).toBe(0);
  });

  it("delete rate limit", () => {
    const { exitCode } = hoppyRaw([
      "shield", "rate-limit", "delete",
      "--id", rateLimitId,
      "--yes",
    ]);

    expect(exitCode).toBe(0);
    rateLimitId = ""; // prevent double-delete
  });

  it("create access list", () => {
    const { json, exitCode } = hoppy([
      "shield", "access-list", "create",
      "--shield-zone-id", shieldZoneId,
      "--name", "hoppy-e2e-access-list",
      "--type", "0",
      "--content", "192.168.1.0/24",
    ]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    // Store ID for subsequent steps — look for "id" (camelCase)
    accessListId = String(json!["id"]);
  });

  it("get access list", () => {
    const { json, exitCode } = hoppy([
      "shield", "access-list", "get",
      "--shield-zone-id", shieldZoneId,
      "--id", accessListId,
    ]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("list access lists", () => {
    const { exitCode } = hoppyRaw([
      "shield", "access-list", "list",
      "--shield-zone-id", shieldZoneId,
    ]);
    expect(exitCode).toBe(0);
  });

  it("update access list", () => {
    const { json, exitCode } = hoppy([
      "shield", "access-list", "update",
      "--shield-zone-id", shieldZoneId,
      "--id", accessListId,
      "--name", "hoppy-e2e-access-list-upd",
    ]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("delete access list", () => {
    const { exitCode } = hoppyRaw([
      "shield", "access-list", "delete",
      "--shield-zone-id", shieldZoneId,
      "--id", accessListId,
      "--yes",
    ]);
    expect(exitCode).toBe(0);
  });

  it("get bot detection", () => {
    const { json, exitCode } = hoppy([
      "shield", "bot-detection", "get",
      "--shield-zone-id", shieldZoneId,
    ]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("update bot detection", () => {
    const { json, exitCode } = hoppy([
      "shield", "bot-detection", "update",
      "--shield-zone-id", shieldZoneId,
      "--execution-mode", "1",
    ]);
    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
  });

  it("delete pull zone", () => {
    const { exitCode } = hoppyRaw([
      "pull-zone", "delete", "--id", pullZoneId, "--yes",
    ]);

    expect(exitCode).toBe(0);
    pullZoneId = ""; // prevent afterAll double-delete
  });
});
