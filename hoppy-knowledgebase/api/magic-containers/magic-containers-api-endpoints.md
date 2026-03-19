---
title: Magic Containers API - Endpoints
type: api-reference
category: magic-containers
subcategory: endpoints
base_url: https://api.bunny.net/mc
auth_header: AccessKey
date: 2026-03-18
---

# Magic Containers API - Endpoints

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header (personal API key)

---

## 1. Add Application Endpoint

**POST** `/apps/{appId}/containers/{containerId}/endpoints`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| containerId | string | Yes |

### Request Body (`application/json`)

**EndpointRequest:**

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| displayName | string | Yes | Min 1, Max 50 chars |
| cdn | CdnEndpointRequest | No | |
| anycast | AnycastEndpointRequest | No | |

**CdnEndpointRequest (nested):**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| isSslEnabled | boolean | No | |
| stickySessions | StickySessionSettings | No | |
| pullZoneId | integer (int32) | No | |
| portMappings | ContainerPortMappingRequest[] | Yes | Min 1 item |

**StickySessionSettings (nested):**

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| enabled | boolean | No | |
| sessionHeaders | string[] | Yes | 1-3 items |
| cookieName | string | No | |

**ContainerPortMappingRequest (nested):**

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| containerPort | integer (int32) | Yes | 1-65535 |
| exposedPort | integer (int32) | No | 1-65535 |
| protocols | Protocol[] | No | Enum: `Tcp`, `Udp`, `Sctp` |

**AnycastEndpointRequest (nested):**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| type | AnycastIpProtocolVersion | Yes | Enum: `IPv4` |
| portMappings | ContainerPortMappingRequest[] | Yes | Min 1 item |

### Response

| Status | Description | Body |
|--------|-------------|------|
| 201 | Created | `SaveEndpointResponse`: `{ id: string }` |
| 400 | Invalid input / duplicate name | ErrorDetails |
| 401 | Unauthorized | ProblemDetails |
| 403 | Invalid payment / suspended | ErrorDetails |
| 404 | App/container not found | ErrorDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## 2. Delete Application Endpoint

**DELETE** `/apps/{appId}/endpoints/{endpointId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| endpointId | string | Yes |

### Request Body
None

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Endpoint removed | (empty) |
| 401 | Unauthorized | ProblemDetails |
| 403 | Invalid card / suspended | ErrorDetails |
| 404 | App/endpoint not found | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## 3. List Application Endpoints

**GET** `/apps/{appId}/endpoints`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Request Body
None

### Response (200)

**ListEndpointsResponse:**

| Field | Type |
|-------|------|
| items | EndpointListItem[] |
| meta | ListMeta |
| cursor | string (nullable) |

**EndpointListItem:**

| Field | Type |
|-------|------|
| id | string |
| displayName | string |
| publicHost | string |
| type | EndpointType enum: `CDN`, `Anycast`, `PublicIp` |
| isSslEnabled | boolean |
| pullZoneId | string |
| portMappings | EndpointPortMapping[] |
| containerName | string |
| containerId | string |
| stickySessions | EndpointStickySession (optional) |
| internalIpAddresses | EndpointInternalIp[] (nullable) |
| publicIpAddresses | EndpointInternalIp[] (nullable) |

**EndpointPortMapping:**

| Field | Type |
|-------|------|
| containerPort | integer (int32) |
| exposedPort | integer (int32) |
| protocols | Protocol[] enum: `Tcp`, `Udp`, `Sctp` |

**EndpointStickySession:**

| Field | Type |
|-------|------|
| enabled | boolean |
| sessionHeaders | string[] |
| cookieName | string |

**EndpointInternalIp:**

| Field | Type |
|-------|------|
| address | string |
| region | string |

**ListMeta:**

| Field | Type |
|-------|------|
| totalItems | integer (int64) |

### Error Responses

| Status | Description | Body |
|--------|-------------|------|
| 401 | Unauthorized | ProblemDetails |
| 404 | App not found | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## 4. Update Application Endpoint

**PUT** `/apps/{appId}/endpoints/{endpointId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| endpointId | string | Yes |

### Request Body (`application/json`)

Same schema as **Add Application Endpoint** (`EndpointRequest`):
- `displayName` (string, required, 1-50 chars)
- `cdn` (CdnEndpointRequest, optional)
- `anycast` (AnycastEndpointRequest, optional)

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Updated | (empty) |
| 400 | Invalid input | ErrorDetails |
| 401 | Unauthorized | ProblemDetails |
| 403 | Invalid card / suspended | ErrorDetails |
| 404 | App/container/endpoint not found | ErrorDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## Enums Reference

**Protocol:** `Tcp`, `Udp`, `Sctp`

**AnycastIpProtocolVersion:** `IPv4`

**EndpointType:** `CDN`, `Anycast`, `PublicIp`

---

## Common Error Schemas

**ErrorDetails:**

| Field | Type | Notes |
|-------|------|-------|
| title | string | read-only |
| status | integer (int32) | read-only |
| detail | string | nullable, read-only |
| instance | string | nullable, read-only |
| errors | ValidationError[] | nullable, read-only |

**ValidationError:**

| Field | Type |
|-------|------|
| field | string (nullable) |
| message | string |

**ProblemDetails:**

| Field | Type |
|-------|------|
| type | string (nullable) |
| title | string (nullable) |
| status | integer (int32, nullable) |
| detail | string (nullable) |
| instance | string (nullable) |
