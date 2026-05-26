---
title: What hoppy could deliver for Terraform on bunny.net
type: research
date: 2026-05-15
status: draft
tags:
  - research
  - terraform
  - iac
  - provider
---

# Terraform on bunny.net — what hoppy can deliver

There is no officially supported Terraform provider for bunny.net. A
community provider exists (`BunnyWay/terraform-provider-bunnynet`,
maintained outside Anthropic/Bunny) and covers a meaningful subset, but
hoppy is in a strong position to either complement it or compete on
freshness and surface coverage.

Below is a menu of deliverables, from "easy wins" to "build a provider",
plus the build-vs-buy tradeoffs.

## Why hoppy is well-positioned

- The work of iter-32 (`bunny-net-api` consolidated crate) is exactly
  the kind of *typed, redaction-aware, recording-friendly* API client
  that a Terraform provider needs. A provider is, mechanically, a thin
  glue layer between Terraform's CRUD lifecycle and that crate.
- iter-33/36/37 produced shape-first tests and a live-API recording
  loop — that infrastructure also catches drift in a provider's
  resource schemas.
- The `--format json` and `--format text` outputs of hoppy already
  produce the kind of structured data that Terraform data sources need.

## Deliverable menu

### 1. `bunny-net-api` crate as the engine for a Terraform plugin (Go)

The community provider is in Go. If we want to keep hoppy as the only
Rust artifact, we can publish stable JSON over the existing crate as a
**provider runtime**:

- Add `hoppy provider serve` (or a separate `hoppy-tfp` binary) that
  speaks the Terraform Plugin Framework v6 protocol *over gRPC*, using
  `tonic`.
- Schema generation: we already have serde structs with field-level
  redaction. Walk them with a derive macro to emit Terraform schema
  blocks (`tfschema!` macro).
- Lifecycle: every `*_api::create/update/delete` already exists. The
  provider's CRUD just dispatches to those.
- Drift detection comes for free — the `--format json` round-trip is
  what `terraform plan` does internally.

**Effort**: 3–4 iterations. A Rust-native provider is unusual but
viable (Pulumi has done it). The bigger risk is HashiCorp's plugin
protocol evolving faster than we maintain `tonic` plumbing.

### 2. `terraform-provider-bunnynet` SDK in Go that wraps hoppy

A pragmatic compromise:

- Publish `bunny-net-api` as a thin Go SDK by generating Go bindings
  from the OpenAPI spec hoppy already maintains
  (`bunny-net-api-spec=2026-05-05` per `--version`).
- The Go provider uses the Go SDK; hoppy uses the Rust crate; both
  share the spec.
- Net effect: there's one source-of-truth API spec, two clients, one
  provider.

**Effort**: 2 iterations to scaffold + sync. The spec sync is the
ongoing maintenance cost.

### 3. `hoppy export` — emit Terraform HCL for an existing account

A killer adoption feature for the community provider (or our own):

```sh
hoppy export --format terraform --resources pull-zone,storage-zone,dns-zone > main.tf
```

- Walks the account, calls `list` on each surface, and emits one
  `resource "bunnynet_pull_zone" "name" { ... }` block per resource.
- Uses `--format json` underneath to fetch full state, maps to HCL.
- Optional `--state` mode that writes a `terraform.tfstate` so the
  resources can be `terraform import`-free.

**Effort**: 1–2 iterations. Mostly mechanical translation. Big win
because it lets users adopt Terraform incrementally on existing
accounts — today they'd have to write every block by hand or use
`terraform import` per resource.

### 4. `hoppy plan` — terraform-style diff for ad-hoc config files

Even without Terraform: define a YAML/JSON spec for desired pull-zone
state, run `hoppy plan -f mypz.yaml`, see the diff against the live
account. Could share the diff renderer with a future Terraform
provider.

**Effort**: 1 iteration if scoped to one surface. Useful as a
standalone feature before any provider lands.

### 5. Documentation: "hoppy as a substrate for IaC"

The shortest path to value:

- A how-to in the docs: "wrap hoppy calls in `local-exec` provisioners"
  with idempotency tips (hoppy already returns 404 on missing resources
  with a clean error message).
- A second how-to: "use hoppy in CI to assert your account state
  matches a checked-in expected.json".

**Effort**: half a day. Doesn't build a provider but lowers the floor
for users who already have `local-exec`-based workflows.

## Recommendation

**Short term (iter-38 or 39)**: deliver #3 (`hoppy export --format
terraform`). It plays to hoppy's strengths (typed listing across every
surface), produces an artifact users want immediately, and de-risks
items #1 / #2 by exercising the HCL schema mapping before we commit to
implementing CRUD.

**Medium term**: pick between #1 and #2 based on whether we want to be
a Rust-shop or align with the wider Terraform ecosystem (Go). #2 is
lower risk; #1 is more interesting and reusable as a Pulumi runtime.

**Don't**: build a competing provider from scratch in Go without
reusing the bunny-net-api crate's redaction/typing work. We'd
re-implement what iter-32/33 already gave us.

## Open questions

- Is there commercial demand?  Should we float #3 as a blog post / GH
  issue on the community provider to gauge interest before building?
- The community provider's resource coverage on 2026-05-15 vs hoppy's
  — does hoppy already cover more surfaces (containers, shield event
  logs, log forwarding) than the community provider?  If yes, that's
  the headline.
- Recording framework: could the same `HOPPY_RECORD_DIR` fixture
  format be reused for Terraform acceptance tests?  Probably yes — and
  it'd halve the maintenance cost of provider tests.
