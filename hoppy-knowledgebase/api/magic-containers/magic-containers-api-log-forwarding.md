---
title: Magic Containers API - Log Forwarding
type: api-reference
category: magic-containers
subcategory: log-forwarding
base_url: https://api.bunny.net/mc
auth_header: AccessKey
date: 2026-03-18
status: active
---

# Magic Containers API - Log Forwarding

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header (personal API key)

---

## 1. Create Log Forwarding Configuration

**POST** `/log/forwarding`

### Request Body (`application/json`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| app | string | Yes | App unique identifier |
| type | LogForwardingType enum | Yes | `SyslogUdp`, `SyslogTcp` |
| endpoint | string | Yes | Log destination endpoint |
| port | integer | Yes | Log destination port |
| token | string | No | Token passed as syslog field |
| format | SyslogFormat enum | Yes | `SyslogRfc3164`, `SyslogRfc5424` |
| enabled | boolean | Yes | Whether forwarding is enabled |

### Response (201)

**LogForwardingConfiguration:**

| Field | Type | Description |
|-------|------|-------------|
| id | string | Unique identifier |
| app | string | App unique identifier |
| productId | string | Product unique identifier |
| type | LogForwardingType enum | `SyslogUdp`, `SyslogTcp` |
| endpoint | string | Log destination endpoint |
| port | integer | Log destination port |
| createdAt | string (date-time) | ISO 8601 timestamp |
| token | string | Token passed as syslog field |
| format | SyslogFormat enum | `SyslogRfc3164`, `SyslogRfc5424` |
| enabled | boolean | Forwarding enabled status |

### Error Responses

| Status | Body |
|--------|------|
| 400 | ErrorDetails |
| 401 | ProblemDetails |
| 500 | ErrorDetails |

---

## 2. Get Log Forwarding Configuration

**GET** `/log/forwarding/{appId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Response (200)

Returns **LogForwardingConfiguration** (same schema as create response above).

### Error Responses

| Status | Body |
|--------|------|
| 401 | ProblemDetails |
| 404 | ErrorDetails |
| 500 | ErrorDetails |

---

## 3. List Log Forwarding Configurations

**GET** `/log/forwarding`

### Request Body
None

### Query Parameters
None

### Response (200)

| Field | Type |
|-------|------|
| items | LogForwardingConfiguration[] |

Each item has the same schema as the create response above.

### Error Responses

| Status | Body |
|--------|------|
| 401 | ProblemDetails |
| 500 | ErrorDetails |

---

## 4. Update Log Forwarding Configuration

**PUT** `/log/forwarding/{appId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Request Body (`application/json`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| app | string | Yes | App unique identifier |
| type | LogForwardingType enum | Yes | `SyslogUdp`, `SyslogTcp` |
| endpoint | string | Yes | Log destination endpoint |
| port | integer | Yes | Log destination port |
| token | string | No | Token passed as syslog field |
| format | SyslogFormat enum | Yes | `SyslogRfc3164`, `SyslogRfc5424` |
| enabled | boolean | Yes | Whether forwarding is enabled |

### Response (200)

Returns **LogForwardingConfiguration** (same schema as create response).

### Error Responses

| Status | Body |
|--------|------|
| 400 | ErrorDetails |
| 403 | Unauthorized |
| 404 | ErrorDetails |
| 500 | ErrorDetails |

---

## 5. Delete Log Forwarding Configuration

**DELETE** `/log/forwarding/{appId}`

### Path Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| appId | string | Yes |

### Request Body
None

### Response

| Status | Description | Body |
|--------|-------------|------|
| 204 | Deleted | (empty) |
| 401 | Unauthorized | - |
| 404 | Not found | - |
| 500 | Server error | - |

---

## Enums Reference

**LogForwardingType:** `SyslogUdp`, `SyslogTcp`

**SyslogFormat:** `SyslogRfc3164`, `SyslogRfc5424`
