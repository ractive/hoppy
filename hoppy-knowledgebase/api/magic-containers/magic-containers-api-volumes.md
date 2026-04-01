---
title: Magic Containers API - Volumes
type: api-reference
category: magic-containers
subcategory: volumes
base_url: https://api.bunny.net/mc
auth_header: AccessKey
date: 2026-03-18
status: active
---

# Magic Containers API - Volumes

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header (personal API key)

---

## 1. List Volumes

**GET** `/apps/{appId}/volumes`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Response (200)

**ListVolumesResponse:**

| Field | Type |
|-------|------|
| items | VolumeInList[] |
| meta | ListMeta |
| cursor | string (nullable) |
| summary | ListVolumesSummary |

**VolumeInList:**

| Field | Type |
|-------|------|
| name | string |
| id | string |
| size | number (double) |
| totalUsage | number (double) |
| totalInstancesCount | integer |
| attachedInstancesCount | integer |
| containersCount | integer |
| volumeInstances | VolumeInstance[] |

**VolumeInstance:**

| Field | Type |
|-------|------|
| id | string |
| attachedPods | string[] |
| attachedContainers | string[] |
| region | string |
| status | VolumeInstanceStatus enum |
| size | number (double) |
| usage | number (double) |

**VolumeInstanceStatus enum:** `Unknown`, `Attached`, `Detached`, `Extending`, `Deleting`, `Creating`, `NotScheduled`, `Scheduled`, `Failed`

**ListMeta:**

| Field | Type |
|-------|------|
| totalItems | integer (int64) |

**ListVolumesSummary:**

| Field | Type |
|-------|------|
| totalPods | integer |
| totalContainers | integer |
| totalStorage | number (double) |

### Error Responses

| Status | Body |
|--------|------|
| 401 | ProblemDetails |
| 500 | ErrorDetails |

---

## 2. Update Volume

**PATCH** `/apps/{appId}/volumes/{volumeId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| volumeId | string | Yes |

### Request Body (`application/json`)

**PatchVolumeRequest:**

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| name | string | No | Min 1, Max 50 chars, nullable |
| size | integer (int32) | No | Min 1, Max 100, nullable |

Only provided fields will be updated.

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Updated | `UpdateVolumeResponse`: `{ name: string, size: number (double) }` |
| 400 | Invalid input | ErrorDetails |
| 401 | Unauthorized | ProblemDetails |
| 404 | Volume not found | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## 3. Detach Volume

**POST** `/apps/{appId}/volumes/{volumeId}/detach`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| volumeId | string | Yes |

### Request Body
None

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Detached | `DetachVolumeResponse`: `{ name: string }` |
| 401 | Unauthorized | ProblemDetails |
| 404 | Not found | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## 4. Delete All Volume Instances

**DELETE** `/apps/{appId}/volumes/{volumeId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| volumeId | string | Yes |

### Request Body
None

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Deleted | `DeleteAllVolumeInstancesResponse`: `{ ids: string[] }` |
| 400 | Some instances not detached | ErrorDetails |
| 401 | Unauthorized | ProblemDetails |
| 404 | Volume not found | ErrorDetails |
| 500 | Server error | ErrorDetails |

Note: All volume instances must be detached before deletion.

---

## 5. Delete Volume Instance

**DELETE** `/apps/{appId}/volumes/{volumeId}/instances/{instanceId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |
| volumeId | string | Yes |
| instanceId | string | Yes |

### Request Body
None

### Response

| Status | Description | Body |
|--------|-------------|------|
| 200 | Deleted | `DeleteVolumeInstanceResponse`: `{ id: string }` |
| 400 | Invalid state | ErrorDetails |
| 401 | Unauthorized | ProblemDetails |
| 404 | Volume/instance not found | ErrorDetails |
| 500 | Server error | ErrorDetails |

Note: Volume must be detached before deletion.
