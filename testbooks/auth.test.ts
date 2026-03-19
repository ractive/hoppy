/**
 * Auth check test book.
 *
 * Verifies that the API key is valid and auth check returns account info.
 *
 * Requires BUNNY_API_KEY in the environment.
 */

import { describe, it, expect } from "bun:test";
import { hoppy } from "./helpers.ts";

describe("Auth", () => {
  it("check", () => {
    const { json, exitCode } = hoppy(["auth", "check"]);

    expect(exitCode).toBe(0);
    expect(json).not.toBeNull();
    // Billing response should have balance and billing type info
    expect(json!["Balance"]).toEqual(expect.any(Number));
  });
});
