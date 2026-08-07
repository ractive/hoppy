---
type: adr
title: Hand-written API clients instead of OpenAPI codegen
status: accepted
date: 2026-03-18
deciders:
  - hoppy-maintainers
---

# Hand-written API clients instead of OpenAPI codegen

## Context and Problem Statement

hoppy needs typed Rust clients for the bunny.net REST APIs. Should we generate
them from the published OpenAPI specification, or write them by hand? The
bunny.net API uses PascalCase field names and has several spec inaccuracies,
which affects how well codegen performs.

## Considered Options

- Generate clients with `progenitor` from the OpenAPI spec
- Hand-write the clients from the spec and live-API observation

## Decision Outcome

Chosen option: hand-written clients. Progenitor codegen produced ~51K lines
versus ~4K hand-written, and the PascalCase API surface made the generated code
awkward to consume. See [[research/hand-written-experiment-results]] for the
comparison that drove this call.
