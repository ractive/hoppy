---
title: "Bunny.net API Overview"
date: 2026-03-17
tags:
  - bunny-net
  - api
  - cdn
  - cloud
status: research-complete
---

# Bunny.net API Overview

## Services

Bunny.net is a cloud/edge platform offering the following services:

### CDN (Pull Zones)
Content delivery with smart caching, edge rules, and real-time analytics. Custom hostnames, SSL certificates, cache optimization, and security features (hotlink protection, token authentication).

### Edge Storage (Storage Zones)
Object storage with geo-replication. Access via HTTP, FTP, and SDKs. Fastest edge-replicated object storage.

### Video Streaming (Bunny Stream)
Complete video platform: upload, manage, and deliver videos. Adaptive bitrate streaming, transcoding, DRM protection (MediaCage, FairPlay, Widevine), analytics.

### DNS
Ultra-fast DNS with scriptable records, DNSSEC, logging, and dynamic routing via JavaScript.

### Magic Containers
Containerized app deployment across distributed bare-metal servers. Autoscaling, multi-region, persistent volumes, health checks.

### Edge Scripting
Execute JavaScript/TypeScript at network edges. APIs, middleware, CDN extensions. Webhooks and log forwarding.

### Bunny Shield
Security stack: WAF, DDoS mitigation, rate limiting, bot detection, upload scanning, ACLs.

### Bunny Database
Globally distributed SQLite-compatible database with replication. SQL API access.

### Bunny Optimizer
Automatic image compression, dynamic image transformation (resize, crop, color), HTML prerender, watermarking.

### AI Image Generation
AI-powered image generation with customizable engines and resolutions.

## Authentication

The API uses an **AccessKey** (API key) passed in the `AccessKey` HTTP header:

```
AccessKey: your-api-key-here
```

For the CLI tool, we read this from `BUNNY_API_KEY` environment variable.

Note: Some services (Storage, Stream) have their own separate API keys in addition to the main account API key.

## API Structure

### Separate APIs with Different Base URLs

| API | Base URL | Description |
|-----|----------|-------------|
| Core Platform | `https://api.bunny.net` | Pull zones, DNS, storage zones, account, billing |
| Storage | `https://{region}.storage.bunnycdn.com` | File operations (upload, download, delete, list) |
| Stream | `https://video.bunnycdn.com` | Video management, collections, encoding |
| Shield | `https://api.bunny.net/shield` | WAF, DDoS, rate limiting, bot detection |
| Edge Scripting | Part of core API | Script management and deployment |

### OpenAPI Spec Locations

| API | Spec URL |
|-----|----------|
| Core Platform | https://core-api-public-docs.b-cdn.net/docs/v3/public.json |
| Origin Errors | Relative: /api-reference/origin-errors/openapi.json |
| Storage | Relative: /api-reference/storage/openapi.json |
| Stream | https://video.bunnycdn.com/openapi/bunnynet-video-api.public.json |
| Shield | https://api.bunny.net/shield/docs/v1/swagger.json |
| Edge Scripting | https://core-api-public-docs.b-cdn.net/docs/v3/compute.json |

### Key Endpoint Groups (Core API)

- **Pull Zone**: CRUD, purge cache, add/remove hostnames, set edge rules, SSL management
- **Storage Zone**: CRUD, list files, manage connections
- **DNS Zone**: CRUD, manage records, DNSSEC, import/export
- **Statistics**: Traffic, bandwidth, cache hit rate, per pull zone
- **Billing**: Summary, invoices, charges, payment methods
- **Account**: API keys, user management, team settings
- **Region**: List available regions and their capabilities

### Key Endpoint Groups (Stream API)

- **Video Library**: CRUD, settings, DRM, watermark
- **Video**: Upload, manage, encode, captions, chapters
- **Collection**: Organize videos into collections
- **Statistics**: Views, watch time, per video/library

### Key Endpoint Groups (Shield API)

- **WAF**: Custom rules, managed rulesets
- **Rate Limiting**: Configure rate limits per zone
- **DDoS**: Protection settings, metrics
- **Bot Detection**: Bot detection rules and settings
- **Access Control**: IP ACLs, geo-blocking

## CLI Command Mapping

Proposed service-to-command mapping for `hoppy`:

```
hoppy pull-zone list|get|create|update|delete|purge
hoppy storage-zone list|get|create|update|delete
hoppy storage upload|download|delete|ls
hoppy dns list|get|create|update|delete
hoppy dns record list|add|update|delete
hoppy stream library list|get|create|update|delete
hoppy stream video list|get|upload|delete
hoppy shield waf ...
hoppy shield rate-limit ...
hoppy container list|get|create|update|delete|logs
hoppy script list|get|create|update|delete|deploy
hoppy stats get --pull-zone-id=123 --date-from=... --date-to=...
```

## Documentation

- LLM-friendly docs: https://docs.bunny.net/llms.txt
- API reference: https://docs.bunny.net/api-reference
- OpenAPI specs: https://docs.bunny.net/openapi.md
