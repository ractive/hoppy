---
title: Magic Containers - Containers (Templates) API
type: api-reference
source: https://docs.bunny.net/api-reference/magic-containers/containers
base_url: https://api.bunny.net/mc
auth_header: AccessKey
fetched: 2026-03-18
status: active
---

# Containers (Templates) API

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header with personal API key.

---

## 1. Add Container Template

**POST** `/apps/{appId}/containers`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| appId | string | yes |

### Query Parameters
None.

### Request Body - AddContainerRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| name | string | yes | 1-50 characters |
| imageName | string | yes | minLength: 1; pattern: `[a-zA-Z0-9]+(?:[./_-]{1,2}[a-zA-Z0-9]+)*` |
| imageNamespace | string | yes | minLength: 1; same pattern as imageName |
| imageTag | string | yes | minLength: 1; pattern: `[a-zA-Z0-9]+(?:[._-]{1,2}[a-zA-Z0-9]+)*` |
| imageRegistryId | string | yes | minLength: 1 |
| image | string | no | nullable |
| imageDigest | string | no | pattern: `sha256:[a-z0-9]{64}` |
| imagePullPolicy | ImagePullPolicy | no | enum |
| entryPoint | ContainerEntryPoint | no | |
| probes | ContainerProbes | no | |
| environmentVariables | EnvironmentVariable[] | no | |
| endpoints | EndpointRequest[] | no | |
| volumeMounts | VolumeMountRequest[] | no | |

### Response 201 - ContainerTemplate
See ContainerTemplate schema below.

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 201 | Container template was created | ContainerTemplate |
| 400 | Bad request | ErrorDetails |
| 401 | User is unauthorized | ProblemDetails |
| 403 | User has invalid card or suspended | ErrorDetails |
| 404 | Application not found | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 2. Get Container Template

**GET** `/apps/{appId}/containers/{containerId}`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| appId | string | yes |
| containerId | string | yes |

### Query Parameters
None.

### Request Body
None.

### Response 200 - ContainerTemplate
See ContainerTemplate schema below.

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Successful retrieval | ContainerTemplate |
| 401 | User is unauthorized | ProblemDetails |
| 404 | Application or container template not found | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 3. Patch Container Template

**PATCH** `/apps/{appId}/containers/{containerId}`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| appId | string | yes |
| containerId | string | yes |

### Query Parameters
None.

### Request Body - PatchContainerRequest
All fields are optional (nullable):

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| name | string | no | nullable |
| image | string | no | nullable |
| imageName | string | no | minLength: 1; pattern: `[a-zA-Z0-9]+(?:[./_-]{1,2}[a-zA-Z0-9]+)*` |
| imageNamespace | string | no | minLength: 1; same pattern as imageName |
| imageTag | string | no | minLength: 1; pattern: `[a-zA-Z0-9]+(?:[._-]{1,2}[a-zA-Z0-9]+)*` |
| imageDigest | string | no | pattern: `sha256:[a-z0-9]{64}` |
| imageRegistryId | string | no | minLength: 1 |
| imagePullPolicy | ImagePullPolicy | no | nullable, enum |
| entryPoint | ContainerEntryPoint | no | nullable |
| probes | ContainerProbes | no | nullable |
| environmentVariables | EnvironmentVariable[] | no | nullable |
| endpoints | EndpointRequest[] | no | nullable |
| volumeMounts | VolumeMountRequest[] | no | nullable |

### Response 200 - ContainerTemplate
See ContainerTemplate schema below.

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container template was updated | ContainerTemplate |
| 400 | Bad request | ErrorDetails |
| 401 | User is unauthorized | ProblemDetails |
| 403 | User has invalid card or suspended | ErrorDetails |
| 404 | Application or container template not found | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 4. Delete Container Template

**DELETE** `/apps/{appId}/containers/{containerId}`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| appId | string | yes |
| containerId | string | yes |

### Query Parameters
None.

### Request Body
None.

### Response 200
Empty/no content.

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container template was successfully deleted | (empty) |
| 400 | Cannot delete the last container template | ErrorDetails |
| 401 | User is unauthorized | ProblemDetails |
| 403 | User has invalid card or suspended | ErrorDetails |
| 404 | Application or container template not found | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 5. Set Container Environment Variables

**PUT** `/apps/{appId}/containers/{containerId}/env`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| appId | string | yes |
| containerId | string | yes |

### Query Parameters
None.

### Request Body
A JSON object with string key-value pairs (additionalProperties of type string). Each key is a variable name, each value is a string.

Example:
```json
{
  "DATABASE_URL": "postgres://...",
  "LOG_LEVEL": "info"
}
```

### Response 200 - ContainerTemplate
See ContainerTemplate schema below.

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Success | ContainerTemplate |
| 400 | Invalid input | ErrorDetails |
| 401 | Unauthorized user | ProblemDetails |
| 403 | Invalid card/suspended | ErrorDetails |
| 404 | App/container not found | ErrorDetails |
| 500 | Server error | ErrorDetails |

---

## Shared Types

### ContainerTemplate (Response Object)
All fields required.

| Field | Type |
|-------|------|
| id | string (minLength: 1) |
| name | string (minLength: 1) |
| packageId | string (minLength: 1) |
| image | string (minLength: 1) |
| imageName | string (minLength: 1) |
| imageNamespace | string (minLength: 1) |
| imageTag | string (minLength: 1) |
| imageRegistryId | string (minLength: 1) |
| imageDigest | string (minLength: 1) |
| imagePullPolicy | ImagePullPolicy |
| entryPoint | ContainerEntryPoint |
| probes | ContainerProbes |
| environmentVariables | EnvironmentVariable[] |
| endpoints | ContainerEndpoint[] |
| volumeMounts | ContainerVolumeMount[] |

### ImagePullPolicy Enum
- `Always`
- `IfNotPresent`

### ContainerEntryPoint
| Field | Type | Notes |
|-------|------|-------|
| command | string | nullable |
| commandArray | string[] | nullable |
| arguments | string | nullable |
| argumentsArray | string[] | nullable |
| workingDirectory | string | nullable |

### ContainerProbes
| Field | Type | Notes |
|-------|------|-------|
| startup | ContainerProbe | optional |
| readiness | ContainerProbe | optional |
| liveness | ContainerProbe | optional |

### ContainerProbe
| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| initialDelaySeconds | integer | 1-3600, nullable | 10 |
| periodSeconds | integer | 1-3600, nullable | 10 |
| timeoutSeconds | integer | 1-3600, nullable | 7 |
| failureThreshold | integer | 1-1000, nullable | 3 |
| successThreshold | integer | 1-1000, nullable | 1 |
| httpGet | HttpGetProbe | optional | |
| tcpSocket | TcpSocketProbe | optional | |
| grpc | GrpcProbe | optional | |

### HttpGetProbe
| Field | Type |
|-------|------|
| request | HttpGetProbeRequestDetails (optional) |
| response | HttpGetProbeResponseDetails (optional) |

### HttpGetProbeRequestDetails
| Field | Type | Constraints |
|-------|------|-------------|
| path | string | optional |
| portNumber | integer | 1-65535, optional |

### HttpGetProbeResponseDetails
| Field | Type |
|-------|------|
| expectedStatusCode | HttpStatusCode (enum, optional) |

### TcpSocketProbe
| Field | Type |
|-------|------|
| request | TcpSocketProbeRequestDetails (optional) |

### TcpSocketProbeRequestDetails
| Field | Type | Constraints |
|-------|------|-------------|
| portNumber | integer | 1-65535 |

### GrpcProbe
| Field | Type |
|-------|------|
| request | GrpcProbeRequestDetails (optional) |

### GrpcProbeRequestDetails
| Field | Type | Constraints |
|-------|------|-------------|
| portNumber | integer | 1-65535 |
| serviceName | string | nullable; probes overall health if unspecified |

### EnvironmentVariable
| Field | Type | Required |
|-------|------|----------|
| name | string | yes (minLength: 1) |
| value | string | no |

### EndpointRequest (used in add/patch requests)
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| displayName | string | yes | 1-50 characters |
| cdn | CdnEndpointRequest | no | |
| anycast | AnycastEndpointRequest | no | |

### CdnEndpointRequest
| Field | Type | Notes |
|-------|------|-------|
| isSslEnabled | boolean | |
| stickySessions | StickySessionSettings | |
| pullZoneId | integer | nullable |
| portMappings | ContainerPortMappingRequest[] | minItems: 1, maxItems: 1 |

### AnycastEndpointRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| type | AnycastIpProtocolVersion | yes | |
| portMappings | ContainerPortMappingRequest[] | yes | minItems: 1 |

### AnycastIpProtocolVersion Enum
- `IPv4`

### StickySessionSettings
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| sessionHeaders | string[] | yes | 1-3 items |
| enabled | boolean | no | |
| cookieName | string | no | |

### ContainerPortMappingRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| containerPort | integer | yes | 1-65535 |
| exposedPort | integer | no | 1-65535, nullable |
| protocols | Protocol[] | no | |

### Protocol Enum
- `Tcp`
- `Udp`
- `Sctp`

### VolumeMountRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| name | string | yes | 1-50 characters |
| mountPath | string | yes | minLength: 1; pattern: `^/(?!.*//)(?!.*\.\.)(?!.*\s)([a-zA-Z0-9._-]+/?)+$` |

### ContainerEndpoint (response object)
| Field | Type | Required |
|-------|------|----------|
| displayName | string | yes (minLength: 1) |
| publicHost | string | yes (minLength: 1) |
| type | EndpointType | yes |
| isSslEnabled | boolean | yes |
| pullZoneId | string | yes (minLength: 1) |
| portMappings | EndpointPortMapping[] | yes |
| stickySessions | EndpointStickySession | no |
| internalIpAddresses | EndpointInternalIp[] | no, nullable |
| publicIpAddresses | EndpointInternalIp[] | no, nullable |

### EndpointType Enum
- `CDN`
- `Anycast`
- `PublicIp`

### EndpointPortMapping
| Field | Type | Required |
|-------|------|----------|
| containerPort | integer | yes |
| exposedPort | integer | yes |
| protocols | Protocol[] | yes |

### EndpointStickySession
| Field | Type | Required |
|-------|------|----------|
| enabled | boolean | yes |
| sessionHeaders | string[] | yes |
| cookieName | string | yes (minLength: 1) |

### EndpointInternalIp
| Field | Type | Required |
|-------|------|----------|
| address | string | yes (minLength: 1) |
| region | string | yes (minLength: 1) |

### ContainerVolumeMount
| Field | Type | Required |
|-------|------|----------|
| name | string | yes (minLength: 1) |
| mountPath | string | yes (minLength: 1) |

### HttpStatusCode Enum
Continue, SwitchingProtocols, Processing, EarlyHints, OK, Created, Accepted, NonAuthoritativeInformation, NoContent, ResetContent, PartialContent, MultiStatus, AlreadyReported, IMUsed, MultipleChoices, MovedPermanently, Found, SeeOther, NotModified, UseProxy, Unused, TemporaryRedirect, PermanentRedirect, BadRequest, Unauthorized, PaymentRequired, Forbidden, NotFound, MethodNotAllowed, NotAcceptable, ProxyAuthenticationRequired, RequestTimeout, Conflict, Gone, LengthRequired, PreconditionFailed, RequestEntityTooLarge, RequestUriTooLong, UnsupportedMediaType, RequestedRangeNotSatisfiable, ExpectationFailed, MisdirectedRequest, UnprocessableEntity, Locked, FailedDependency, UpgradeRequired, PreconditionRequired, TooManyRequests, RequestHeaderFieldsTooLarge, UnavailableForLegalReasons, InternalServerError, NotImplemented, BadGateway, ServiceUnavailable, GatewayTimeout, HttpVersionNotSupported, VariantAlsoNegotiates, InsufficientStorage, LoopDetected, NotExtended, NetworkAuthenticationRequired

---

## Shared Error Types

### ProblemDetails
| Field | Type | Notes |
|-------|------|-------|
| type | string | nullable |
| title | string | nullable |
| status | integer (int32) | nullable |
| detail | string | nullable |
| instance | string | nullable |

### ErrorDetails
| Field | Type | Notes |
|-------|------|-------|
| title | string | read-only |
| status | integer (int32) | read-only |
| detail | string | nullable, read-only |
| instance | string | nullable, read-only |
| errors | ValidationError[] | nullable, read-only |

### ValidationError
| Field | Type | Notes |
|-------|------|-------|
| field | string | nullable |
| message | string | |
