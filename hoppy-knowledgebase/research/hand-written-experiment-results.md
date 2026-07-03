---
date: 2026-03-18
status: completed
tags:
- codegen
- hand-written
- serde
- api-client
- experiment
- decision
title: Iteration 0.5 — Hand-Written Experiment Results
type: research
---

# Hand-Written Experiment Results

## Summary

Hand-wrote idiomatic Rust API clients for all 5 bunny.net APIs using reqwest + serde, covering the same endpoints as the Progenitor codegen experiment. **4,035 lines** of hand-written code vs **51,537 lines** of generated code — a **12.8x reduction** — while delivering better ergonomics.

## Results per API

| API | Methods | Types | Lines | Notes |
|-----|---------|-------|-------|-------|
| Core Platform | 6 | 9 | 616 | Pull Zone CRUD only (iter-1 scope) |
| Stream | 9 | 9 | 743 | Includes binary video upload |
| Shield | 24 | 53 | 1,725 | WAF, rate limits, access lists, bot detection |
| Edge Scripting | 22 | 25 | 713 | Scripts, variables, secrets, releases |
| Storage | 4 | 2 | 238 | All 4 endpoints |
| **Total** | **65** | **98** | **4,035** | |

## Head-to-Head Comparison

| Dimension | Progenitor (codegen) | Hand-written |
|-----------|---------------------|--------------|
| **Lines of code** | 51,537 | 4,035 (12.8x less) |
| **Types** | 438 | 98 |
| **Methods** | 192 | 65 |
| **Spec patching required** | Yes — all 5 specs needed fixes | No — types derived from spec by reading |
| **Tooling required** | cargo-progenitor + nightly rustfmt + custom wrapper tool | None |
| **Enum ergonomics** | Integer newtypes: `PullZoneType(0)` | Named variants: `PullZoneType::Premium` |
| **Field nullability** | All fields `Option<T>` | Required fields are non-optional |
| **Method names** | `pull_zone_public_index()`, `dns_zone_public_index2()` | `list_pull_zones()`, `get_pull_zone()` |
| **Binary upload (Stream)** | Not supported — stripped from spec | `upload_video(body: impl Into<Body>)` |
| **Error handling** | `Error<()>` — no structured errors | `ApiError` / `ProblemDetails` parsed from response |
| **Auth injection** | Via `ClientHooks` trait | Built into client constructor |
| **Build time impact** | 5 large crates (~52K lines to compile) | 5 small crates (~4K lines) |
| **API coverage** | Full — every endpoint in every spec | Selective — key endpoints per API |
| **Maintenance on spec change** | Re-run codegen + re-patch spec | Manual update of affected types |
| **Time to implement** | ~minutes (codegen) + hours (patching) | ~30 min per API |

## Key Advantages of Hand-Written

1. **Named enums**: `VideoStatus::Finished` instead of `VideoStatus(3)`. Massive readability win. Progenitor can't produce these from bunny.net's integer-only enum specs.

2. **Non-optional required fields**: `pull_zone.name: String` not `pull_zone.name: Option<String>`. The spec marks everything nullable, but we know from API behavior which fields are always present.

3. **Clean method names**: Self-explanatory names like `list_pull_zones()` instead of `pull_zone_public_index()` with numbered duplicates.

4. **Binary upload support**: The Stream API's video upload works out of the box. Progenitor can't express `application/octet-stream` bodies.

5. **Structured errors**: Error responses are parsed into typed `ApiError` / `ProblemDetails` structs. Progenitor had to strip error schemas to avoid assertion failures.

6. **12.8x less code**: Easier to review, debug, and understand. Faster compile times.

7. **No tooling dependencies**: No nightly rustfmt, no spec patching, no custom wrapper scripts.

## Key Advantages of Progenitor (codegen)

1. **Full API coverage**: Every endpoint in every spec is covered. Hand-written only covers key endpoints.

2. **Mechanical updates**: When the spec changes, re-run codegen. Hand-written requires manual updates.

3. **Exhaustive types**: All 438 types from all specs are generated. Hand-written only includes the ~98 types we actually need.

4. **No interpretation needed**: Codegen is deterministic — no risk of misreading the spec.

## Conclusion

For a CLI tool where we control which endpoints to expose, the hand-written approach is clearly better:
- The CLI will never expose all 192 endpoints — we pick the useful ones
- Ergonomic types make the CLI code much cleaner
- Named enums are essential for a good CLI experience
- The 12.8x code reduction matters for build times and reviewability

The only scenario where codegen wins is if we needed full coverage of all endpoints immediately — but the iterative roadmap means we add APIs one at a time.

**Recommendation: Use hand-written clients.** Start with Core Platform Pull Zones (already done), expand incrementally per iteration. Use the OpenAPI specs as reference documentation, not as codegen input.

## Related

- [[research/openapi-codegen-rust]] — codegen evaluation that preceded this experiment
- [[decision-log]] — decision to use hand-written clients
- [[api/bunny-api-client-patterns]] — patterns that emerged from hand-written approach
