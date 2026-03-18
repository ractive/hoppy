mod client;
mod types;

pub use client::ContainersClient;
pub use types::{
    // Error types
    ErrorDetails, ProblemDetails, ValidationError,
    // Enums
    AnycastIpProtocolVersion, ApplicationStatus, ContainerStatus, DeploymentStatus, EndpointType,
    Granularity, Grade, ImagePullPolicy, LogForwardingType, PodStatus, Protocol,
    RegionProvisioningType, RegistryType, RemoveContainerRegistryStatus, RuntimeType,
    SavedContainerRegistryStatus, SyslogFormat, VolumeInstanceStatus,
    // Pagination
    CursorList, ListMeta,
    // Applications
    AddApplicationRequest, AddApplicationResponse, AppListItem, Application,
    ApplicationOverview, ApplicationStatistics, AutoscalingSettings, ContainerInstance,
    DisplayEndpoint, NetworkLimits, PatchApplicationRequest, RegionSettings, RepositorySettings,
    UpdateRegionSettingsRequest,
    // Container templates
    AddContainerRequest, ContainerEntryPoint, ContainerProbe, ContainerProbes,
    ContainerRequest, ContainerTemplate, ContainerVolumeMount, EnvironmentVariable, GrpcProbe,
    GrpcProbeRequest, HttpGetProbe, HttpGetProbeRequest, HttpGetProbeResponse,
    PatchContainerRequest, TcpSocketProbe, TcpSocketProbeRequest,
    // Endpoints (networking)
    AnycastEndpointRequest, CdnEndpointRequest, ContainerEndpoint, ContainerPortMappingRequest,
    EndpointInternalIp, EndpointListItem, EndpointPortMapping, EndpointRequest,
    EndpointStickySession, SaveEndpointResponse, StickySessionSettings,
    // Volumes
    DeleteAllVolumeInstancesResponse, DeleteVolumeInstanceResponse, DetachVolumeResponse,
    ListVolumesSummary, ListVolumesResponse, PatchVolumeRequest, UpdateVolumeResponse,
    VolumeInList, VolumeInstance, VolumeMountRequest, VolumeRequest, VolumeTemplate,
    // Container registries
    ContainerConfigSuggestions, ContainerImage, ContainerImageTag, ContainerRegistry,
    ContainerRegistryRequest, EnvironmentVariableSuggestion, GetContainerConfigSuggestionsRequest,
    GetContainerImageDigestRequest, ImageTagInfo, ListContainerImageTagsRequest,
    ListContainerImagesRequest, RegistryCredentials, RemoveContainerRegistryResult,
    SaveContainerRegistryResult, SearchPublicContainerImagesRequest,
    // Overview
    DoubleStatusIndicator, Int32StatusIndicator, OverviewContainer, OverviewPod, OverviewRegion,
    // Regions & nodes
    OptimalBaseRegionResponse, Region,
    // User limits
    UserLimits,
    // Log forwarding
    ListLogForwardingResponse, LogForwardingConfiguration, LogForwardingRequest,
};
