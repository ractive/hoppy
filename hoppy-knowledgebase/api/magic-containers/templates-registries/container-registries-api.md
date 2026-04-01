---
title: Magic Containers - Container Registries API
type: api-reference
source: https://docs.bunny.net/api-reference/magic-containers/containerregistries
base_url: https://api.bunny.net/mc
auth_header: AccessKey
fetched: 2026-03-18
status: active
---

# Container Registries API

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header with personal API key.

---

## 1. List Container Registries

**GET** `/registries`

### Path Parameters
None.

### Query Parameters
None.

### Request Body
None.

### Response 200 - ListContainerRegistriesResponse
| Field | Type | Notes |
|-------|------|-------|
| items | ContainerRegistry[] | |
| meta | ListMeta | |
| cursor | string | nullable |

#### ListMeta
| Field | Type |
|-------|------|
| totalItems | integer (int64) |

#### ContainerRegistry
| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | integer (int64) | no | read-only |
| accountId | string | no | nullable |
| userId | string | no | nullable |
| namespaceId | string | yes | |
| displayName | string | yes | |
| hostName | string | yes | |
| userName | string | no | nullable |
| firstPasswordSymbols | string | no | nullable |
| lastPasswordSymbols | string | no | nullable |
| createdAt | string (date-time) | yes | |
| isPublic | boolean | no | nullable |
| lastUpdatedAt | string (date-time) | no | nullable |

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container registries were successfully retrieved | ListContainerRegistriesResponse |
| 401 | User is unauthorized | ProblemDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 2. Get Container Registry

**GET** `/registries/{registryId}`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| registryId | integer (int64) | yes |

### Query Parameters
None.

### Request Body
None.

### Response 200 - ContainerRegistry
Same schema as in List Container Registries above.

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container registry was successfully retrieved | ContainerRegistry |
| 401 | User is unauthorized | ProblemDetails |
| 404 | Container registry not found | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 3. Update Container Registry

**PUT** `/registries/{registryId}`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| registryId | integer (int64) | yes |

### Query Parameters
None.

### Request Body - ContainerRegistryRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| displayName | string | yes | minLength: 1 |
| type | RegistryType | no | enum |
| passwordCredentials | Credentials | no | |

#### RegistryType Enum
- `DockerHub`
- `GitHub`

#### Credentials
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| userName | string | yes | minLength: 1 |
| password | string | yes | minLength: 1 |

### Response 200 - SaveContainerRegistryResult
| Field | Type | Notes |
|-------|------|-------|
| id | integer (int64) | nullable |
| error | string | nullable |
| status | SavedContainerRegistryStatus | enum |

#### SavedContainerRegistryStatus Enum
- `Saved`
- `SecretsValidationFailed`
- `UnknownErrorOccured`
- `NotFound`
- `InvalidInput`

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container registry was updated | SaveContainerRegistryResult |
| 400 | Bad request | ErrorDetails |
| 401 | User is unauthorized | ProblemDetails |
| 404 | Container registry not found | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 4. Delete Container Registry

**DELETE** `/registries/{registryId}`

### Path Parameters
| Name | Type | Required |
|------|------|----------|
| registryId | integer (int64) | yes |

### Query Parameters
None.

### Request Body
None.

### Response 200 - RemoveContainerRegistryResult
| Field | Type | Notes |
|-------|------|-------|
| status | RemoveContainerRegistryResponseStatus | enum |
| applications | string[] | nullable |

#### RemoveContainerRegistryResponseStatus Enum
- `NotFound`
- `InUse`
- `Removed`

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container registry was deleted or conflict returned | RemoveContainerRegistryResult |
| 401 | User is unauthorized | ProblemDetails |
| 409 | Registry in use by applications | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 5. List Container Images

**POST** `/registries/images`

### Path Parameters
None.

### Query Parameters
None.

### Request Body - ListContainerImagesRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| registryId | string | yes | minLength: 1; values: "dockerhub", "github", or private registry ID |

### Response 200 - ContainerImage[]
| Field | Type |
|-------|------|
| id | string |
| namespace | string |

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container images were successfully retrieved | ContainerImage[] |
| 401 | User is unauthorized | ProblemDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 6. List Container Image Tags

**POST** `/registries/tags`

### Path Parameters
None.

### Query Parameters
None.

### Request Body - ListContainerImageTagsRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| registryId | string | yes | minLength: 1; values: "dockerhub", "github", or private registry ID |
| imageName | string | yes | min: 1, max: 100 characters |
| imageNamespace | string | yes | min: 1, max: 100 characters |

### Response 200 - ContainerImageTag[]
| Field | Type |
|-------|------|
| name | string |

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container image tags were successfully retrieved | ContainerImageTag[] |
| 401 | User is unauthorized | ProblemDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 7. Get Container Image Digest

**POST** `/registries/digest`

### Path Parameters
None.

### Query Parameters
None.

### Request Body - GetContainerImageDigestByTagRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| registryId | string | yes | minLength: 1; values: "dockerhub", "github", or private registry ID |
| imageName | string | yes | min: 1, max: 100 characters |
| imageNamespace | string | yes | min: 1, max: 100 characters |
| tag | string | yes | min: 1, max: 100 characters |

### Response 200 - ImageTagInfo
| Field | Type |
|-------|------|
| imageNamespace | string |
| image | string |
| tag | string |
| digest | string |

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Container image tag digest retrieved successfully | ImageTagInfo |
| 401 | User is unauthorized | ProblemDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 8. Get Container Config Suggestions

**POST** `/registries/config-suggestions`

### Path Parameters
None.

### Query Parameters
None.

### Request Body - GetContainerConfigSuggestionsRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| registryId | string | yes | minLength: 1; values: "dockerhub", "github", or private registry ID |
| imageName | string | yes | min: 1, max: 100 characters |
| imageNamespace | string | yes | min: 1, max: 100 characters |
| tag | string | yes | min: 1, max: 100 characters |

### Response 200 - ContainerConfigSuggestions
| Field | Type | Notes |
|-------|------|-------|
| endpointSuggestions | EndpointRequest[] | |
| environmentVariablesSuggestions | EnvironmentVariableSuggestion[] | |
| appName | string | nullable |
| description | string | nullable |
| instructions | string | nullable |
| registryUrl | string | nullable |

#### EnvironmentVariableSuggestion
| Field | Type |
|-------|------|
| name | string |
| defaultValue | string (nullable) |
| description | string (nullable) |
| required | boolean |

For EndpointRequest and nested types, see the Shared Types section below.

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Configuration suggestions retrieved successfully | ContainerConfigSuggestions |
| 401 | User is unauthorized | ProblemDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

---

## 9. Search Public Container Images

**POST** `/registries/public-images/search`

### Path Parameters
None.

### Query Parameters
None.

### Request Body - SearchPublicContainerImagesRequest
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| registryId | string | yes | 1-100 characters; values: "dockerhub", "github", or private registry ID |
| prefix | string | yes | 1-100 characters |
| size | integer | no | 1-100 |
| page | integer | no | 1-100 |

### Response 200 - ContainerImage[]
| Field | Type |
|-------|------|
| id | string |
| namespace | string |

### Status Codes
| Code | Description | Response Schema |
|------|-------------|-----------------|
| 200 | Successfully retrieved public images | ContainerImage[] |
| 401 | User is unauthorized | ProblemDetails |
| 422 | Unprocessable entity | ErrorDetails |
| 500 | Unexpected server error | ErrorDetails |

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
