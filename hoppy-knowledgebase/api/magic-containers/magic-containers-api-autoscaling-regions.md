---
title: Magic Containers API - Autoscaling & Region Settings
type: api-reference
category: magic-containers
subcategory: autoscaling-regions
base_url: https://api.bunny.net/mc
auth_header: AccessKey
date: 2026-03-18
---

# Magic Containers API - Autoscaling & Region Settings

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header (personal API key)

---

## 1. Get Application Autoscaling

**GET** `/apps/{appId}/autoscaling`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Response (200)

**AutoscalingSettings:**

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| min | integer (int32) | Yes | Range 1-1000 |
| max | integer (int32) | Yes | Range 1-1000 |

### Error Responses

| Status | Body |
|--------|------|
| 401 | ProblemDetails |
| 404 | ErrorDetails |
| 500 | ErrorDetails |

---

## 2. Update Application Autoscaling

**PUT** `/apps/{appId}/autoscaling`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Request Body (`application/json`)

**AutoscalingSettings:**

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| min | integer (int32) | Yes | Range 1-1000 |
| max | integer (int32) | Yes | Range 1-1000 |

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Updated | (empty) |
| 400 | Invalid input | ErrorDetails |
| 401 | Unauthorized | ProblemDetails |
| 403 | Invalid card / suspended | ErrorDetails |
| 404 | App not found | ErrorDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## 3. Get Application Region Settings

**GET** `/apps/{appId}/region-settings`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Response (200)

**RegionSettings:**

| Field | Type | Required | Nullable | Notes |
|-------|------|----------|----------|-------|
| allowedRegionIds | string[] (unique) | Yes | No | e.g. `["DE", "UK", "US"]` |
| requiredRegionIds | string[] (unique) | Yes | No | e.g. `["DE"]` |
| maxAllowedRegions | integer (int32) | No | Yes | e.g. `5` |
| provisioningType | RegionProvisioningType | No | No | Enum: `Static`, `Dynamic` |

### Error Responses

| Status | Body |
|--------|------|
| 401 | ProblemDetails |
| 404 | ErrorDetails |
| 500 | ErrorDetails |

---

## 4. Update Application Region Settings

**PUT** `/apps/{appId}/region-settings`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Request Body (`application/json`)

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| allowedRegionIds | string[] | No | |
| requiredRegionIds | string[] | No | |
| maxAllowedRegions | integer (int32) | No | |
| nodeSelectors | object (key-value pairs) | No | |

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Updated | (empty) |
| 400 | Invalid input | ErrorDetails |
| 401 | Unauthorized | ProblemDetails |
| 403 | Invalid card / suspended | ErrorDetails |
| 404 | App not found | ErrorDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## Enums Reference

**RegionProvisioningType:** `Static`, `Dynamic`
