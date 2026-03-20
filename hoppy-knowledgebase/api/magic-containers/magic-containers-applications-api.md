---
title: Magic Containers Applications API Reference
source: docs.bunny.net
fetched: 2026-03-18
category: api-reference
tags:
  - magic-containers
  - applications
  - bunny-api
base_url: https://api.bunny.net/mc
authentication: AccessKey header (API key)
---

# Magic Containers Applications API Reference

Base URL: `https://api.bunny.net/mc`
Authentication: `AccessKey` header with a valid personal API key.

---

## 1. Get Application

**Method:** `GET`
**Path:** `/apps/{appId}`

### Path Parameters

| Parameter | Type     | Required |
|-----------|----------|----------|
| appId     | string   | Yes      |

### Query Parameters

None.

### Request Body

None.

### Response (200 OK) — Application object

#### Top-Level Fields

| Field                | Type                          | Required |
|----------------------|-------------------------------|----------|
| id                   | string (min 1)                | Yes      |
| name                 | string (min 1)                | Yes      |
| status               | ApplicationStatus enum        | Yes      |
| runtimeType          | ApplicationRuntimeType enum   | Yes      |
| regionSettings       | RegionSettings                | Yes      |
| containerTemplates   | ContainerTemplate[]           | Yes      |
| containerInstances   | ContainerInstance[]            | Yes      |
| volumes              | VolumeTemplate[]              | Yes      |
| displayEndpoint      | DisplayEndpoint               | No       |
| autoScaling          | AutoscalingSettings           | No       |
| networkSettings      | NetworkLimits                 | No       |
| repositorySettings   | RepositorySettings            | No       |

### Status Codes

| Code | Description                     | Schema        |
|------|---------------------------------|---------------|
| 200  | Application retrieved           | Application   |
| 401  | Unauthorized                    | ProblemDetails|
| 404  | Application not found           | ErrorDetails  |
| 500  | Server error                    | ErrorDetails  |

---

## 2. Get Application Overview

**Method:** `GET`
**Path:** `/apps/{appId}/overview`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

None.

### Request Body

None.

### Response (200 OK) — Overview object

| Field                | Type                                    |
|----------------------|-----------------------------------------|
| targetLatency        | DoubleStatusIndicator                   |
| currentLatency       | DoubleStatusIndicator                   |
| activeRegions        | Int32StatusIndicator                    |
| activeInstances      | Int32StatusIndicator                    |
| desiredInstances     | integer                                 |
| status               | ApplicationStatus enum                  |
| averageCPU           | DoubleStatusIndicator                   |
| averageRAM           | DoubleStatusIndicator                   |
| averageVolumesUsage  | DoubleStatusIndicator                   |
| regions              | OverviewRegion[]                        |
| averageLatency       | number (double)                         |
| totalVolumeSizeInGb  | number (double)                         |
| monthlyCost          | number (double)                         |
| latencyChart         | object (map: string -> double)          |

#### DoubleStatusIndicator

| Field       | Type              |
|-------------|-------------------|
| indicator   | number (double)   |
| statusGrade | Grade enum        |

#### Int32StatusIndicator

| Field       | Type         |
|-------------|--------------|
| indicator   | integer      |
| statusGrade | Grade enum   |

#### OverviewRegion

| Field                          | Type                                    |
|--------------------------------|-----------------------------------------|
| region                         | string                                  |
| isRequired                     | boolean                                 |
| instances                      | integer                                 |
| status                         | DeploymentStatus enum                   |
| averageCPU                     | number (double)                         |
| averageRAM                     | number (double)                         |
| averageVolumesUsagePercentage  | number (double)                         |
| requests                       | number (double)                         |
| anycastTraffic                 | number (double)                         |
| pods                           | OverviewPod[]                           |

#### OverviewPod

| Field                          | Type                                    |
|--------------------------------|-----------------------------------------|
| name                           | string                                  |
| status                         | PodStatus enum                          |
| lastHeartBeat                  | date-time (nullable)                    |
| outboundTrafficChart           | object (map: string -> integer, nullable)|
| cpuUsage                       | number (double)                         |
| ramUsage                       | number (double)                         |
| containers                     | OverviewContainer[]                     |
| message                        | string (nullable)                       |
| volumesUtilizationPercentage   | object (map: string -> double)          |

#### OverviewContainer

| Field            | Type                          |
|------------------|-------------------------------|
| id               | string                        |
| name             | string                        |
| cpuUsage         | number (double)               |
| ramUsage         | number (double)               |
| reason           | string                        |
| message          | string                        |
| status           | ContainerStatus enum          |
| image            | string (nullable)             |
| imageDisplay     | string (nullable)             |
| numberOfRestarts | integer (nullable)            |

### Status Codes

| Code | Description           | Schema        |
|------|-----------------------|---------------|
| 200  | Success               | Overview      |
| 401  | Unauthorized          | ProblemDetails|
| 404  | Application not found | ErrorDetails  |
| 500  | Server error          | ErrorDetails  |

---

## 3. Get Application Statistics

**Method:** `GET`
**Path:** `/apps/{appId}/statistics`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

| Parameter   | Type              | Required | Description              |
|-------------|-------------------|----------|--------------------------|
| fromDate    | string (date-time)| Yes      | Start of time range      |
| toDate      | string (date-time)| No       | End of time range        |
| granularity | Granularity enum  | Yes      | Data aggregation level   |

### Request Body

None.

### Response (200 OK) — Statistics object

| Field                    | Type                                       |
|--------------------------|--------------------------------------------|
| targetLatencyChart       | object (map: string -> double)             |
| activeRegionsChart       | object (map: string -> int64)              |
| latencyChart             | object (map: string -> double)             |
| instancesChart           | object (map: string -> int64)              |
| cpuUsageChart            | object (map: string -> double)             |
| ramUsageChart            | object (map: string -> double)             |
| trafficChart             | object (map: string -> double)             |
| volumesSplitUsageChart   | object (map: string -> map: string -> double)|
| volumesSplitCapacityChart| object (map: string -> map: string -> double)|
| volumesUsageChart        | object (map: string -> double)             |
| volumesCapacityChart     | object (map: string -> double)             |

### Status Codes

| Code | Description           | Schema         |
|------|-----------------------|----------------|
| 200  | Success               | Statistics     |
| 400  | Bad request           | ErrorDetails   |
| 401  | Unauthorized          | ProblemDetails |
| 404  | Application not found | ErrorDetails   |
| 500  | Server error          | ErrorDetails   |

---

## 4. Patch Application

**Method:** `PATCH`
**Path:** `/apps/{appId}`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

None.

### Request Body — PatchApplicationRequest

All fields are **optional** (partial update).

| Field              | Type                          | Constraints                                    |
|--------------------|-------------------------------|------------------------------------------------|
| name               | string                        | 1-100 chars, pattern `[A-Za-z0-9-\s]+`        |
| runtimeType        | ApplicationRuntimeType enum   |                                                |
| autoScaling        | AutoscalingSettings           |                                                |
| regionSettings     | UpdateRegionSettingsRequest   |                                                |
| containerTemplates | ContainerRequest[]            |                                                |
| volumes            | VolumeRequest[]               |                                                |

(See Shared Schemas section below for nested object definitions.)

### Response (200 OK) — AddApplicationResponse

| Field | Type   |
|-------|--------|
| id    | string |

### Status Codes

| Code | Description              | Schema         |
|------|--------------------------|----------------|
| 200  | Success                  | AddApplicationResponse |
| 400  | Bad request              | ErrorDetails   |
| 401  | Unauthorized             | ProblemDetails |
| 403  | Invalid card / suspended | ErrorDetails   |
| 404  | Not found                | ProblemDetails |
| 422  | Unprocessable entity     | ErrorDetails   |
| 500  | Server error             | ErrorDetails   |

---

## 5. Update Application (Full Replace)

**Method:** `PUT`
**Path:** `/apps/{appId}`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

None.

### Request Body — AddApplicationRequest

| Field                          | Type                          | Required | Constraints                                    |
|--------------------------------|-------------------------------|----------|------------------------------------------------|
| name                           | string                        | Yes      | 1-100 chars, pattern `[A-Za-z0-9-\s]+`        |
| runtimeType                    | ApplicationRuntimeType enum   | Yes      |                                                |
| autoScaling                    | AutoscalingSettings           | Yes      |                                                |
| regionSettings                 | UpdateRegionSettingsRequest   | Yes      |                                                |
| terminationGracePeriodSeconds  | integer                       | No       | 1-300                                          |
| repositorySettings             | RepositorySettings            | No       |                                                |
| containerTemplates             | ContainerRequest[]            | No       | min 1 item                                     |
| volumes                        | VolumeRequest[]               | No       |                                                |

(See Shared Schemas section below for nested object definitions.)

### Response (200 OK) — AddApplicationResponse

| Field | Type   |
|-------|--------|
| id    | string |

### Status Codes

| Code | Description              | Schema         |
|------|--------------------------|----------------|
| 200  | Success                  | AddApplicationResponse |
| 400  | Bad request / ID mismatch| ErrorDetails   |
| 401  | Unauthorized             | ProblemDetails |
| 403  | Invalid card / suspended | ErrorDetails   |
| 404  | Not found                | ProblemDetails |
| 422  | Unprocessable entity     | ErrorDetails   |
| 500  | Server error             | ErrorDetails   |

---

## 6. Deploy Application

**Method:** `POST`
**Path:** `/apps/{appId}/deploy`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

None.

### Request Body

None.

### Response (200 OK)

No body.

### Status Codes

| Code | Description              | Schema         |
|------|--------------------------|----------------|
| 200  | Successfully deployed    | (none)         |
| 401  | Unauthorized             | ProblemDetails |
| 404  | Application not found    | ErrorDetails   |
| 409  | Application suspended    | ErrorDetails   |
| 500  | Server error             | ErrorDetails   |

---

## 7. Undeploy Application

**Method:** `POST`
**Path:** `/apps/{appId}/undeploy`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

None.

### Request Body

None.

### Response (200 OK)

No body.

### Status Codes

| Code | Description              | Schema         |
|------|--------------------------|----------------|
| 200  | Successfully undeployed  | (none)         |
| 401  | Unauthorized             | ProblemDetails |
| 404  | Application not found    | ErrorDetails   |
| 409  | Application suspended    | ErrorDetails   |
| 500  | Server error             | ErrorDetails   |

---

## 8. Restart Application

**Method:** `POST`
**Path:** `/apps/{appId}/restart`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

None.

### Request Body

None.

### Response (200 OK)

No body.

### Status Codes

| Code | Description              | Schema         |
|------|--------------------------|----------------|
| 200  | Restart triggered        | (none)         |
| 401  | Unauthorized             | ProblemDetails |
| 403  | Invalid card / suspended | ProblemDetails |
| 404  | Application not found    | ErrorDetails   |
| 500  | Server error             | ErrorDetails   |

---

## 9. Delete Application

**Method:** `DELETE`
**Path:** `/apps/{appId}`

### Path Parameters

| Parameter | Type   | Required |
|-----------|--------|----------|
| appId     | string | Yes      |

### Query Parameters

None.

### Request Body

None.

### Response (200 OK)

No body.

### Status Codes

| Code | Description              | Schema         |
|------|--------------------------|----------------|
| 200  | Successfully deleted     | (none)         |
| 401  | Unauthorized             | ProblemDetails |
| 403  | Invalid card / suspended | ProblemDetails |
| 500  | Server error             | ErrorDetails   |

---

## Shared Schemas

### Enums

#### ApplicationStatus
`Unknown`, `Active`, `Progressing`, `Inactive`, `Failing`, `Suspended`

#### ApplicationRuntimeType
`Shared`, `Reserved`

#### EndpointType
`CDN`, `Anycast`, `PublicIp`

#### RegionProvisioningType
`Static`, `Dynamic`

#### ImagePullPolicy
`Always`, `IfNotPresent`

#### Protocol
`Tcp`, `Udp`, `Sctp`

#### AnycastIpProtocolVersion
`IPv4`

#### Grade
`CouldBeBetter`, `NotBad`, `DoingGreat`

#### DeploymentStatus
`Unknown`, `Active`, `Progressing`, `Inactive`, `Failing`

#### PodStatus
`NotScheduled`, `Scheduled`, `Ready`, `Deleting`

#### ContainerStatus
`NotStarted`, `Started`, `Ready`

#### Granularity
`Daily`, `Hourly`, `Minute`

#### HttpStatusCode
`Continue`, `SwitchingProtocols`, `Processing`, `EarlyHints`, `OK`, `Created`, `Accepted`, `NonAuthoritativeInformation`, `NoContent`, `ResetContent`, `PartialContent`, `MultiStatus`, `AlreadyReported`, `IMUsed`, `MultipleChoices`, `MovedPermanently`, `Found`, `SeeOther`, `NotModified`, `UseProxy`, `Unused`, `TemporaryRedirect`, `PermanentRedirect`, `BadRequest`, `Unauthorized`, `PaymentRequired`, `Forbidden`, `NotFound`, `MethodNotAllowed`, `NotAcceptable`, `ProxyAuthenticationRequired`, `RequestTimeout`, `Conflict`, `Gone`, `LengthRequired`, `PreconditionFailed`, `RequestEntityTooLarge`, `RequestUriTooLong`, `UnsupportedMediaType`, `RequestedRangeNotSatisfiable`, `ExpectationFailed`, `MisdirectedRequest`, `UnprocessableEntity`, `Locked`, `FailedDependency`, `UpgradeRequired`, `PreconditionRequired`, `TooManyRequests`, `RequestHeaderFieldsTooLarge`, `UnavailableForLegalReasons`, `InternalServerError`, `NotImplemented`, `BadGateway`, `ServiceUnavailable`, `GatewayTimeout`, `HttpVersionNotSupported`, `VariantAlsoNegotiates`, `InsufficientStorage`, `LoopDetected`, `NotExtended`, `NetworkAuthenticationRequired`

---

### Shared Object Schemas

#### AutoscalingSettings

| Field | Type    | Required | Constraints    |
|-------|---------|----------|----------------|
| min   | integer | Yes      | 1-1000         |
| max   | integer | Yes      | 1-1000         |

#### UpdateRegionSettingsRequest

| Field            | Type            | Required | Notes                        |
|------------------|-----------------|----------|------------------------------|
| allowedRegionIds | string[]        | No       | e.g. `["DE", "UK", "US"]`   |
| requiredRegionIds| string[]        | No       | e.g. `["DE"]`               |
| maxAllowedRegions| integer (int32) | No       |                              |
| nodeSelectors    | object (map: string -> string) | No |                       |

#### RegionSettings (response)

| Field              | Type                          | Required |
|--------------------|-------------------------------|----------|
| allowedRegionIds   | string[] (unique)             | Yes      |
| requiredRegionIds  | string[] (unique)             | Yes      |
| maxAllowedRegions  | integer (nullable)            | No       |
| provisioningType   | RegionProvisioningType enum (nullable) | No |

#### ContainerRequest (for PATCH/PUT request bodies)

| Field              | Type                   | Required | Constraints                                                |
|--------------------|------------------------|----------|------------------------------------------------------------|
| id                 | string                 | No       | Absent = new container; present = update existing          |
| name               | string                 | Yes      | 1-50 chars                                                 |
| image              | string                 | No       | Nullable                                                   |
| imageName          | string                 | Yes      | min 1, pattern `[a-zA-Z0-9]+(?:[./_-]{1,2}[a-zA-Z0-9]+)*`|
| imageNamespace     | string                 | Yes      | min 1, pattern `[a-zA-Z0-9]+(?:[./_-]{1,2}[a-zA-Z0-9]+)*`|
| imageTag           | string                 | Yes      | min 1, pattern `[a-zA-Z0-9]+(?:[._-]{1,2}[a-zA-Z0-9]+)*` |
| imageDigest        | string                 | No       | Nullable, pattern `sha256:[a-z0-9]{64}`                    |
| imageRegistryId    | string                 | Yes      | min 1                                                      |
| imagePullPolicy    | ImagePullPolicy enum   | No       |                                                            |
| entryPoint         | ContainerEntryPoint    | No       |                                                            |
| probes             | ContainerProbes        | No       |                                                            |
| environmentVariables| EnvironmentVariable[] | No       |                                                            |
| endpoints          | EndpointRequest[]      | No       |                                                            |
| volumeMounts       | VolumeMountRequest[]   | No       |                                                            |

#### ContainerTemplate (response)

| Field              | Type                   | Required |
|--------------------|------------------------|----------|
| id                 | string (min 1)         | Yes      |
| name               | string (min 1)         | Yes      |
| packageId          | string (min 1)         | Yes      |
| image              | string (min 1)         | Yes      |
| imageName          | string (min 1)         | Yes      |
| imageNamespace     | string (min 1)         | Yes      |
| imageTag           | string (min 1)         | Yes      |
| imageRegistryId    | string (min 1)         | Yes      |
| imageDigest        | string (min 1)         | Yes      |
| imagePullPolicy    | ImagePullPolicy enum   | Yes      |
| entryPoint         | ContainerEntryPoint    | Yes      |
| probes             | ContainerProbes        | Yes      |
| environmentVariables| EnvironmentVariable[] | Yes      |
| endpoints          | ContainerEndpoint[]    | Yes      |
| volumeMounts       | ContainerVolumeMount[] | Yes      |

#### ContainerEntryPoint

| Field            | Type       | Required |
|------------------|------------|----------|
| command          | string     | No       |
| commandArray     | string[]   | No       |
| arguments        | string     | No       |
| argumentsArray   | string[]   | No       |
| workingDirectory | string     | No       |

All fields nullable.

#### ContainerProbes

| Field     | Type           | Required |
|-----------|----------------|----------|
| startup   | ContainerProbe | No       |
| readiness | ContainerProbe | No       |
| liveness  | ContainerProbe | No       |

All fields nullable.

#### ContainerProbe

| Field               | Type              | Required | Constraints        | Default |
|---------------------|-------------------|----------|--------------------|---------|
| initialDelaySeconds | integer           | No       | 1-3600             | 10      |
| periodSeconds       | integer           | No       | 1-3600             | 10      |
| timeoutSeconds      | integer           | No       | 1-3600             | 7       |
| failureThreshold    | integer           | No       | 1-1000             | 3       |
| successThreshold    | integer           | No       | 1-1000             | 1       |
| httpGet             | HttpGetProbe      | No       |                    |         |
| tcpSocket           | TcpSocketProbe    | No       |                    |         |
| grpc                | GrpcProbe         | No       |                    |         |

All fields nullable.

#### HttpGetProbe

| Field    | Type                         |
|----------|------------------------------|
| request  | HttpGetProbeRequestDetails (nullable)  |
| response | HttpGetProbeResponseDetails (nullable) |

#### HttpGetProbeRequestDetails

| Field      | Type              |
|------------|-------------------|
| path       | string (nullable) |
| portNumber | integer, 1-65535 (nullable) |

#### HttpGetProbeResponseDetails

| Field              | Type                      |
|--------------------|---------------------------|
| expectedStatusCode | HttpStatusCode enum (nullable) |

#### TcpSocketProbe

| Field   | Type                              |
|---------|-----------------------------------|
| request | TcpSocketProbeRequestDetails (nullable) |

#### TcpSocketProbeRequestDetails

| Field      | Type                     |
|------------|--------------------------|
| portNumber | integer, 1-65535 (nullable) |

#### GrpcProbe

| Field   | Type                          |
|---------|-------------------------------|
| request | GrpcProbeRequestDetails (nullable) |

#### GrpcProbeRequestDetails

| Field       | Type                     |
|-------------|--------------------------|
| portNumber  | integer, 1-65535 (nullable) |
| serviceName | string (nullable)        |

#### EnvironmentVariable

| Field | Type              | Required |
|-------|-------------------|----------|
| name  | string (min 1)    | Yes      |
| value | string (nullable) | No       |

#### EndpointRequest (for PATCH/PUT request bodies)

| Field       | Type                    | Required |
|-------------|-------------------------|----------|
| displayName | string (min 1, max 50)  | Yes      |
| cdn         | CdnEndpointRequest      | No       |
| anycast     | AnycastEndpointRequest  | No       |

#### CdnEndpointRequest

| Field          | Type                              | Required | Constraints           |
|----------------|-----------------------------------|----------|-----------------------|
| isSslEnabled   | boolean                           | No       |                       |
| stickySessions | StickySessionSettings             | No       |                       |
| pullZoneId     | integer (int32, nullable)         | No       |                       |
| portMappings   | ContainerPortMappingRequest[]     | No       | min 1, max 1 item     |

#### AnycastEndpointRequest

| Field        | Type                              | Required |
|--------------|-----------------------------------|----------|
| type         | AnycastIpProtocolVersion enum     | Yes      |
| portMappings | ContainerPortMappingRequest[]     | Yes      |

#### ContainerPortMappingRequest

| Field         | Type            | Required | Constraints               |
|---------------|-----------------|----------|---------------------------|
| containerPort | integer         | Yes      | 1-65535                   |
| exposedPort   | integer         | No       | 1-65535, nullable         |
| protocols     | Protocol[]      | No       |                           |

#### StickySessionSettings

| Field          | Type      | Required | Constraints     |
|----------------|-----------|----------|-----------------|
| enabled        | boolean   | No       |                 |
| sessionHeaders | string[]  | Yes      | 1-3 items       |
| cookieName     | string    | No       |                 |

#### ContainerEndpoint (response)

| Field              | Type                        | Required |
|--------------------|-----------------------------|----------|
| displayName        | string (min 1)              | Yes      |
| publicHost         | string (min 1)              | Yes      |
| type               | EndpointType enum           | Yes      |
| isSslEnabled       | boolean                     | Yes      |
| pullZoneId         | string (min 1)              | Yes      |
| portMappings       | EndpointPortMapping[]       | Yes      |
| stickySessions     | EndpointStickySession       | No       |
| internalIpAddresses| EndpointInternalIp[]        | No       |
| publicIpAddresses  | EndpointInternalIp[]        | No       |

#### EndpointPortMapping (response)

| Field         | Type        | Required |
|---------------|-------------|----------|
| containerPort | integer     | Yes      |
| exposedPort   | integer     | Yes      |
| protocols     | Protocol[]  | Yes      |

#### EndpointStickySession (response)

| Field          | Type      | Required |
|----------------|-----------|----------|
| enabled        | boolean   | Yes      |
| sessionHeaders | string[]  | Yes      |
| cookieName     | string (min 1) | Yes |

#### EndpointInternalIp

| Field   | Type           | Required |
|---------|----------------|----------|
| address | string (min 1) | Yes      |
| region  | string (min 1) | Yes      |

#### DisplayEndpoint (response)

| Field   | Type              | Required |
|---------|-------------------|----------|
| id      | string (min 1)    | Yes      |
| address | string (min 1)    | Yes      |
| type    | EndpointType enum | Yes      |

#### VolumeRequest (for PATCH/PUT request bodies)

| Field | Type    | Required | Constraints     |
|-------|---------|----------|-----------------|
| name  | string  | Yes      | 1-50 chars      |
| size  | integer | Yes      | 1-100           |

#### VolumeMountRequest (for PATCH/PUT request bodies)

| Field     | Type   | Required | Constraints                                                        |
|-----------|--------|----------|--------------------------------------------------------------------|
| name      | string | Yes      | 1-50 chars                                                         |
| mountPath | string | Yes      | Pattern: `^/(?!.*//)(?!.*\.\.)(?!.*\s)([a-zA-Z0-9._-]+/?)+$`     |

#### VolumeTemplate (response)

| Field | Type            | Required |
|-------|-----------------|----------|
| name  | string (min 1)  | Yes      |
| size  | number (double) | Yes      |

#### ContainerVolumeMount (response)

| Field     | Type           | Required |
|-----------|----------------|----------|
| name      | string (min 1) | Yes      |
| mountPath | string (min 1) | Yes      |

#### ContainerInstance (response)

| Field         | Type           | Required |
|---------------|----------------|----------|
| id            | string (min 1) | Yes      |
| templateId    | string (min 1) | Yes      |
| podId         | string (min 1) | Yes      |
| nodeIpAddress | string (min 1) | Yes      |

#### NetworkLimits (response)

| Field                  | Type                  |
|------------------------|-----------------------|
| ingressBandwidthLimit  | integer (int64, nullable) |
| egressBandwidthLimit   | integer (int64, nullable) |

#### RepositorySettings

| Field              | Type              |
|--------------------|-------------------|
| templateRepository | string (nullable) |
| repositoryName     | string (nullable) |
| owner              | string (nullable) |

---

### Error Schemas

#### ProblemDetails

| Field    | Type                  |
|----------|-----------------------|
| type     | string (nullable)     |
| title    | string (nullable)     |
| status   | integer (nullable)    |
| detail   | string (nullable)     |
| instance | string (nullable)     |

#### ErrorDetails

| Field    | Type                          |
|----------|-------------------------------|
| title    | string (read-only)            |
| status   | integer (read-only)           |
| detail   | string (nullable, read-only)  |
| instance | string (nullable, read-only)  |
| errors   | ValidationError[] (nullable, read-only) |

#### ValidationError

| Field   | Type              |
|---------|-------------------|
| field   | string (nullable) |
| message | string            |

#### AddApplicationResponse

| Field | Type   |
|-------|--------|
| id    | string |

## Related
- [[api/bunny-api-overview]] — overview of all bunny.net APIs
- [[api/magic-containers/magic-containers-api-endpoints]] — endpoint management
- [[api/magic-containers/magic-containers-api-autoscaling-regions]] — autoscaling & regions
- [[api/magic-containers/magic-containers-api-volumes]] — volume management
- [[api/magic-containers/magic-containers-api-log-forwarding]] — log forwarding
- [[api/magic-containers/magic-containers-api-misc]] — limits, nodes, regions, pods
- [[api/magic-containers/templates-registries/containers-api]] — container templates
- [[api/magic-containers/templates-registries/container-registries-api]] — registries
