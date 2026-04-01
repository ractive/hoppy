---
title: Magic Containers API - Limits, Nodes, Regions, Pods
type: api-reference
category: magic-containers
subcategory: misc
base_url: https://api.bunny.net/mc
auth_header: AccessKey
date: 2026-03-18
status: active
---

# Magic Containers API - Limits, Nodes, Regions, Pods

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header (personal API key)

---

## 1. Get User Limits

**GET** `/limits`

### Request Body
None

### Response (200)

**UserLimits:**

| Field | Type | Required | Nullable |
|-------|------|----------|----------|
| maxNumberOfApplications | integer (int32) | Yes | No |
| existingNumberOfApplications | integer (int32) | Yes | No |
| maxNumberOfRegionsPerApplication | integer (int32) | No | Yes |
| maxNumberOfInstancesPerRegion | integer (int32) | Yes | No |
| maxNumberOfInstancesPerApplication | integer (int32) | No | Yes |
| maxNumberOfVolumesPerApplication | integer (int32) | Yes | No |
| maxVolumeSize | integer (int32) | No | Yes |

### Error Responses

| Status | Body |
|--------|------|
| 401 | ProblemDetails |
| 404 | ErrorDetails |
| 500 | ErrorDetails |

---

## 2. List Nodes

**GET** `/nodes`

### Query Parameters

| Parameter | Type | Required | Default | Constraints |
|-----------|------|----------|---------|-------------|
| nextCursor | string | No | - | Pagination cursor |
| limit | integer | No | 20 | Min: 1, Max: 1000 |

### Response (200)

**ListNodesResponse:**

| Field | Type |
|-------|------|
| items | string[] (IP addresses) |
| meta | ListMeta: `{ totalItems: integer (int64) }` |
| cursor | string (nullable) |

### Error Responses

| Status | Body |
|--------|------|
| 500 | ErrorDetails |

---

## 3. List Regions

**GET** `/regions`

### Query Parameters

| Parameter | Type | Required | Default | Constraints |
|-----------|------|----------|---------|-------------|
| nextCursor | string | No | - | Pagination cursor |
| limit | integer | No | 20 | Min: 1, Max: 1000 |

### Response (200)

**ListRegionsResponse:**

| Field | Type |
|-------|------|
| items | Region[] |
| meta | ListMeta: `{ totalItems: integer (int64) }` |
| cursor | string (nullable) |

**Region:**

| Field | Type |
|-------|------|
| id | string |
| name | string |
| group | string |
| hasAnycastSupport | boolean |
| hasCapacity | boolean |

### Error Responses

| Status | Body |
|--------|------|
| 400 | ErrorDetails |
| 401 | ProblemDetails |
| 500 | ErrorDetails |

---

## 4. Get Optimal Base Region

**GET** `/regions/optimal`

### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| cdnServerToken | string | No | CDN server token for location-based determination |

### Response (200)

**OptimalBaseRegionResponse:**

| Field | Type |
|-------|------|
| region | Region (see above) |

### Error Responses

| Status | Body |
|--------|------|
| 401 | ProblemDetails |
| 500 | ErrorDetails |

---

## 5. Recreate Pod

**POST** `/apps/{appId}/pods/{podId}/recreate`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| podId | string | Yes |

### Request Body
None

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Pod reset triggered | (empty) |
| 401 | Unauthorized | ProblemDetails |
| 403 | Invalid card / suspended | ProblemDetails |
| 404 | Pod not found | ErrorDetails |
| 409 | Conflict | ErrorDetails |
| 500 | Server error | ErrorDetails |
