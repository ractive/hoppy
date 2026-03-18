use bunny_api_containers::ContainersClient;
use bunny_api_containers::{
    AddApplicationRequest, AddContainerRequest, AutoscalingSettings, CdnEndpointRequest,
    ContainerRegistryRequest, EndpointRequest, GetContainerConfigSuggestionsRequest,
    GetContainerImageDigestRequest, Granularity, ListContainerImageTagsRequest,
    ListContainerImagesRequest, LogForwardingRequest, LogForwardingType, PatchApplicationRequest,
    PatchContainerRequest, PatchVolumeRequest, RegistryCredentials, RegistryType, RuntimeType,
    SearchPublicContainerImagesRequest, SyslogFormat, UpdateRegionSettingsRequest,
};
use std::collections::HashMap;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_APPS_LIST: &str = include_str!("fixtures/apps_list.json");
const FIXTURE_APP_GET: &str = include_str!("fixtures/app_get.json");
const FIXTURE_APP_ADD: &str = include_str!("fixtures/app_add.json");
const FIXTURE_APP_OVERVIEW: &str = include_str!("fixtures/app_overview.json");
const FIXTURE_APP_STATISTICS: &str = include_str!("fixtures/app_statistics.json");
const FIXTURE_AUTOSCALING_GET: &str = include_str!("fixtures/autoscaling_get.json");
const FIXTURE_REGION_SETTINGS_GET: &str = include_str!("fixtures/region_settings_get.json");
const FIXTURE_CONTAINER_GET: &str = include_str!("fixtures/container_get.json");
const FIXTURE_ENDPOINTS_LIST: &str = include_str!("fixtures/endpoints_list.json");
const FIXTURE_ENDPOINT_ADD: &str = include_str!("fixtures/endpoint_add.json");
const FIXTURE_VOLUMES_LIST: &str = include_str!("fixtures/volumes_list.json");
const FIXTURE_VOLUME_UPDATE: &str = include_str!("fixtures/volume_update.json");
const FIXTURE_VOLUME_DETACH: &str = include_str!("fixtures/volume_detach.json");
const FIXTURE_VOLUME_DELETE_ALL: &str = include_str!("fixtures/volume_delete_all.json");
const FIXTURE_VOLUME_DELETE_INSTANCE: &str = include_str!("fixtures/volume_delete_instance.json");
const FIXTURE_REGISTRIES_LIST: &str = include_str!("fixtures/registries_list.json");
const FIXTURE_REGISTRY_GET: &str = include_str!("fixtures/registry_get.json");
const FIXTURE_REGISTRY_ADD: &str = include_str!("fixtures/registry_add.json");
const FIXTURE_REGISTRY_DELETE: &str = include_str!("fixtures/registry_delete.json");
const FIXTURE_CONTAINER_IMAGES_LIST: &str = include_str!("fixtures/container_images_list.json");
const FIXTURE_CONTAINER_IMAGE_TAGS: &str = include_str!("fixtures/container_image_tags.json");
const FIXTURE_IMAGE_DIGEST: &str = include_str!("fixtures/image_digest.json");
const FIXTURE_CONFIG_SUGGESTIONS: &str = include_str!("fixtures/config_suggestions.json");
const FIXTURE_PUBLIC_IMAGES_SEARCH: &str = include_str!("fixtures/public_images_search.json");
const FIXTURE_REGIONS_LIST: &str = include_str!("fixtures/regions_list.json");
const FIXTURE_OPTIMAL_REGION: &str = include_str!("fixtures/optimal_region.json");
const FIXTURE_NODES_LIST: &str = include_str!("fixtures/nodes_list.json");
const FIXTURE_USER_LIMITS: &str = include_str!("fixtures/user_limits.json");
const FIXTURE_LOG_FORWARDING_LIST: &str = include_str!("fixtures/log_forwarding_list.json");
const FIXTURE_LOG_FORWARDING_GET: &str = include_str!("fixtures/log_forwarding_get.json");
const FIXTURE_ERROR_NOT_FOUND: &str = include_str!("fixtures/error_not_found.json");
const FIXTURE_ERROR_VALIDATION: &str = include_str!("fixtures/error_validation.json");

fn test_client(uri: &str) -> ContainersClient {
    ContainersClient::with_base_url("test-api-key", uri)
}

// ---------------------------------------------------------------------------
// Applications
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_applications_returns_paginated() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APPS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_applications(None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.items[0].id, "HbqFNk0KZjzYOcp");
    assert_eq!(result.items[0].name, "stova-api");
    assert_eq!(result.items[1].id, "cm7XE04UfPYTKxu");
    let meta = result.meta.unwrap();
    assert_eq!(meta.total_items, 2);
}

#[tokio::test]
async fn list_applications_with_cursor() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("nextCursor", "abc123"))
        .and(query_param("limit", "10"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APPS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_applications(Some("abc123"), Some(10))
        .await
        .unwrap();

    assert!(!result.items.is_empty());
}

#[tokio::test]
async fn get_application_returns_details() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let app = test_client(&server.uri())
        .get_application("l3faCv1fWRHAYNU")
        .await
        .unwrap();

    assert_eq!(app.id, "l3faCv1fWRHAYNU");
    assert_eq!(app.name, "hoppy-test-fixture-tmp");
    assert_eq!(app.container_templates.len(), 1);
    assert_eq!(app.container_templates[0].name, "nginx");
}

#[tokio::test]
async fn get_application_overview() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/overview"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_OVERVIEW, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let overview = test_client(&server.uri())
        .get_application_overview("l3faCv1fWRHAYNU")
        .await
        .unwrap();

    assert_eq!(overview.regions.len(), 1);
    assert_eq!(overview.regions[0].region.as_deref(), Some("DE"));
    assert!(
        overview.average_cpu.is_some(),
        "averageCPU should deserialize"
    );
    assert!(
        overview.average_ram.is_some(),
        "averageRAM should deserialize"
    );
    let chart = overview.latency_chart.unwrap();
    assert_eq!(chart.len(), 3);
}

#[tokio::test]
async fn get_application_statistics() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("fromDate", "2026-03-01"))
        .and(query_param("granularity", "Daily"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_STATISTICS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_application_statistics("l3faCv1fWRHAYNU", "2026-03-01", Granularity::Daily, None)
        .await
        .unwrap();

    let latency = stats.latency_chart.unwrap();
    assert_eq!(latency.len(), 3);
}

#[tokio::test]
async fn get_application_statistics_with_to_date() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("fromDate", "2026-03-01"))
        .and(query_param("granularity", "Hourly"))
        .and(query_param("toDate", "2026-03-03"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_STATISTICS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_application_statistics(
            "l3faCv1fWRHAYNU",
            "2026-03-01",
            Granularity::Hourly,
            Some("2026-03-03"),
        )
        .await
        .unwrap();

    assert!(stats.cpu_usage_chart.is_some());
}

#[tokio::test]
async fn add_application_sends_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "name": "my-app",
            "runtimeType": "shared",
            "autoScaling": {"min": 1, "max": 2},
            "regionSettings": {"requiredRegionIds": ["DE"]}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_ADD, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = AddApplicationRequest {
        name: "my-app".to_owned(),
        runtime_type: RuntimeType::Shared,
        auto_scaling: AutoscalingSettings { min: 1, max: 2 },
        region_settings: UpdateRegionSettingsRequest {
            allowed_region_ids: None,
            required_region_ids: Some(vec!["DE".to_owned()]),
            max_allowed_regions: None,
            node_selectors: None,
        },
        termination_grace_period_seconds: None,
        repository_settings: None,
        container_templates: None,
        volumes: None,
    };

    let resp = test_client(&server.uri())
        .add_application(&body)
        .await
        .unwrap();

    assert_eq!(resp.id, "l3faCv1fWRHAYNU");
}

#[tokio::test]
async fn update_application_sends_body() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "name": "my-app",
            "runtimeType": "shared",
            "autoScaling": {"min": 1, "max": 2},
            "regionSettings": {"requiredRegionIds": ["DE"]}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_ADD, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = AddApplicationRequest {
        name: "my-app".to_owned(),
        runtime_type: RuntimeType::Shared,
        auto_scaling: AutoscalingSettings { min: 1, max: 2 },
        region_settings: UpdateRegionSettingsRequest {
            allowed_region_ids: None,
            required_region_ids: Some(vec!["DE".to_owned()]),
            max_allowed_regions: None,
            node_selectors: None,
        },
        termination_grace_period_seconds: None,
        repository_settings: None,
        container_templates: None,
        volumes: None,
    };

    let resp = test_client(&server.uri())
        .update_application("l3faCv1fWRHAYNU", &body)
        .await
        .unwrap();

    assert_eq!(resp.id, "l3faCv1fWRHAYNU");
}

#[tokio::test]
async fn patch_application_sends_body() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "name": "renamed-app"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_ADD, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = PatchApplicationRequest {
        name: Some("renamed-app".to_owned()),
        runtime_type: None,
        auto_scaling: None,
        region_settings: None,
        container_templates: None,
        volumes: None,
    };

    let resp = test_client(&server.uri())
        .patch_application("l3faCv1fWRHAYNU", &body)
        .await
        .unwrap();

    assert_eq!(resp.id, "l3faCv1fWRHAYNU");
}

#[tokio::test]
async fn deploy_application() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps/l3faCv1fWRHAYNU/deploy"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .deploy_application("l3faCv1fWRHAYNU")
        .await
        .unwrap();
}

#[tokio::test]
async fn undeploy_application() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps/l3faCv1fWRHAYNU/undeploy"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .undeploy_application("l3faCv1fWRHAYNU")
        .await
        .unwrap();
}

#[tokio::test]
async fn restart_application() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps/l3faCv1fWRHAYNU/restart"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .restart_application("l3faCv1fWRHAYNU")
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_application() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_application("l3faCv1fWRHAYNU")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Autoscaling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_autoscaling() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/autoscaling"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_AUTOSCALING_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let settings = test_client(&server.uri())
        .get_autoscaling("l3faCv1fWRHAYNU")
        .await
        .unwrap();

    assert_eq!(settings.min, 1);
    assert_eq!(settings.max, 1);
}

#[tokio::test]
async fn update_autoscaling() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/apps/l3faCv1fWRHAYNU/autoscaling"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({"min": 1, "max": 3})))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let body = AutoscalingSettings { min: 1, max: 3 };

    test_client(&server.uri())
        .update_autoscaling("l3faCv1fWRHAYNU", &body)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Region settings
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_region_settings() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/region-settings"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_REGION_SETTINGS_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let settings = test_client(&server.uri())
        .get_region_settings("l3faCv1fWRHAYNU")
        .await
        .unwrap();

    assert!(settings.allowed_region_ids.contains(&"DE".to_owned()));
    assert!(settings.required_region_ids.contains(&"DE".to_owned()));
    assert_eq!(settings.max_allowed_regions, Some(5));
}

#[tokio::test]
async fn update_region_settings() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/apps/l3faCv1fWRHAYNU/region-settings"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "allowedRegionIds": ["DE", "AMS"],
            "requiredRegionIds": ["DE"]
        })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateRegionSettingsRequest {
        allowed_region_ids: Some(vec!["DE".to_owned(), "AMS".to_owned()]),
        required_region_ids: Some(vec!["DE".to_owned()]),
        max_allowed_regions: None,
        node_selectors: None,
    };

    test_client(&server.uri())
        .update_region_settings("l3faCv1fWRHAYNU", &body)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Container templates
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_container() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps/l3faCv1fWRHAYNU/containers"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "name": "nginx",
            "imageName": "nginx",
            "imageNamespace": "library",
            "imageTag": "alpine",
            "imageRegistryId": "1155"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CONTAINER_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = AddContainerRequest {
        name: "nginx".to_owned(),
        image_name: "nginx".to_owned(),
        image_namespace: "library".to_owned(),
        image_tag: "alpine".to_owned(),
        image_registry_id: "1155".to_owned(),
        image: None,
        image_digest: None,
        image_pull_policy: None,
        entry_point: None,
        probes: None,
        environment_variables: None,
        endpoints: None,
        volume_mounts: None,
    };

    let ct = test_client(&server.uri())
        .add_container("l3faCv1fWRHAYNU", &body)
        .await
        .unwrap();

    assert_eq!(ct.id, "l3faCv1fWRHAYNU-I7t6");
    assert_eq!(ct.name, "nginx");
}

#[tokio::test]
async fn get_container() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/apps/l3faCv1fWRHAYNU/containers/l3faCv1fWRHAYNU-I7t6",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CONTAINER_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let ct = test_client(&server.uri())
        .get_container("l3faCv1fWRHAYNU", "l3faCv1fWRHAYNU-I7t6")
        .await
        .unwrap();

    assert_eq!(ct.id, "l3faCv1fWRHAYNU-I7t6");
    assert_eq!(ct.image, "library/nginx:alpine");
}

#[tokio::test]
async fn patch_container() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(
            "/apps/l3faCv1fWRHAYNU/containers/l3faCv1fWRHAYNU-I7t6",
        ))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "imageTag": "stable"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CONTAINER_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = PatchContainerRequest {
        name: None,
        image: None,
        image_name: None,
        image_namespace: None,
        image_tag: Some("stable".to_owned()),
        image_digest: None,
        image_registry_id: None,
        image_pull_policy: None,
        entry_point: None,
        probes: None,
        environment_variables: None,
        endpoints: None,
        volume_mounts: None,
    };

    let ct = test_client(&server.uri())
        .patch_container("l3faCv1fWRHAYNU", "l3faCv1fWRHAYNU-I7t6", &body)
        .await
        .unwrap();

    assert_eq!(ct.name, "nginx");
}

#[tokio::test]
async fn delete_container() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/apps/l3faCv1fWRHAYNU/containers/l3faCv1fWRHAYNU-I7t6",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_container("l3faCv1fWRHAYNU", "l3faCv1fWRHAYNU-I7t6")
        .await
        .unwrap();
}

#[tokio::test]
async fn set_container_env() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path(
            "/apps/l3faCv1fWRHAYNU/containers/l3faCv1fWRHAYNU-I7t6/env",
        ))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "PORT": "8080",
            "ENV": "production"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CONTAINER_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mut env = HashMap::new();
    env.insert("PORT".to_owned(), "8080".to_owned());
    env.insert("ENV".to_owned(), "production".to_owned());

    let ct = test_client(&server.uri())
        .set_container_env("l3faCv1fWRHAYNU", "l3faCv1fWRHAYNU-I7t6", &env)
        .await
        .unwrap();

    assert_eq!(ct.id, "l3faCv1fWRHAYNU-I7t6");
}

// ---------------------------------------------------------------------------
// Endpoints (networking)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/endpoints"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ENDPOINTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_endpoints("l3faCv1fWRHAYNU")
        .await
        .unwrap();

    assert_eq!(result.items.len(), 0);
    let meta = result.meta.unwrap();
    assert_eq!(meta.total_items, 0);
}

#[tokio::test]
async fn add_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/apps/l3faCv1fWRHAYNU/containers/l3faCv1fWRHAYNU-I7t6/endpoints",
        ))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "displayName": "endpoint-0",
            "cdn": {
                "portMappings": [
                    {"containerPort": 80}
                ]
            }
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ENDPOINT_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    use bunny_api_containers::ContainerPortMappingRequest;
    let body = EndpointRequest {
        display_name: "endpoint-0".to_owned(),
        cdn: Some(CdnEndpointRequest {
            is_ssl_enabled: None,
            sticky_sessions: None,
            pull_zone_id: None,
            port_mappings: Some(vec![ContainerPortMappingRequest {
                container_port: 80,
                exposed_port: None,
                protocols: None,
            }]),
        }),
        anycast: None,
    };

    let resp = test_client(&server.uri())
        .add_endpoint("l3faCv1fWRHAYNU", "l3faCv1fWRHAYNU-I7t6", &body)
        .await
        .unwrap();

    assert_eq!(resp.id, "test-endpoint-id");
}

#[tokio::test]
async fn update_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/apps/l3faCv1fWRHAYNU/endpoints/test-endpoint-id"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "displayName": "endpoint-updated"
        })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let body = EndpointRequest {
        display_name: "endpoint-updated".to_owned(),
        cdn: None,
        anycast: None,
    };

    test_client(&server.uri())
        .update_endpoint("l3faCv1fWRHAYNU", "test-endpoint-id", &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/apps/l3faCv1fWRHAYNU/endpoints/test-endpoint-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_endpoint("l3faCv1fWRHAYNU", "test-endpoint-id")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Volumes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_volumes() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/volumes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VOLUMES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_volumes("l3faCv1fWRHAYNU")
        .await
        .unwrap();

    assert_eq!(result.items.len(), 0);
    let summary = result.summary.unwrap();
    assert_eq!(summary.total_pods, 1);
}

#[tokio::test]
async fn update_volume() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/apps/l3faCv1fWRHAYNU/volumes/vol-123"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({"name": "my-vol"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VOLUME_UPDATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = PatchVolumeRequest {
        name: Some("my-vol".to_owned()),
        size: None,
    };

    let resp = test_client(&server.uri())
        .update_volume("l3faCv1fWRHAYNU", "vol-123", &body)
        .await
        .unwrap();

    assert_eq!(resp.name, "my-vol");
    assert!((resp.size - 10.0).abs() < 0.001);
}

#[tokio::test]
async fn detach_volume() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps/l3faCv1fWRHAYNU/volumes/vol-123/detach"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VOLUME_DETACH, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .detach_volume("l3faCv1fWRHAYNU", "vol-123")
        .await
        .unwrap();

    assert_eq!(resp.name, "my-vol");
}

#[tokio::test]
async fn delete_all_volume_instances() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/apps/l3faCv1fWRHAYNU/volumes/vol-123"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VOLUME_DELETE_ALL, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .delete_all_volume_instances("l3faCv1fWRHAYNU", "vol-123")
        .await
        .unwrap();

    assert_eq!(resp.ids.len(), 2);
    assert_eq!(resp.ids[0], "inst-1");
}

#[tokio::test]
async fn delete_volume_instance() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/apps/l3faCv1fWRHAYNU/volumes/vol-123/instances/inst-1",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VOLUME_DELETE_INSTANCE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .delete_volume_instance("l3faCv1fWRHAYNU", "vol-123", "inst-1")
        .await
        .unwrap();

    assert_eq!(resp.id, "inst-1");
}

// ---------------------------------------------------------------------------
// Container registries
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_registries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/registries"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRIES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri()).list_registries().await.unwrap();

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0].display_name, "GitHub Packages testuser");
    assert_eq!(result.items[0].id, Some(9999));
}

#[tokio::test]
async fn get_registry() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRY_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let reg = test_client(&server.uri()).get_registry(9999).await.unwrap();

    assert_eq!(reg.id, Some(9999));
    assert_eq!(reg.display_name, "GitHub Packages testuser");
    assert_eq!(reg.host_name, "ghcr.io");
    assert_eq!(reg.is_public, Some(false));
}

#[tokio::test]
async fn add_registry() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/registries"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "displayName": "My Registry",
            "passwordCredentials": {
                "userName": "testuser",
                "password": "test-password"
            }
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRY_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = ContainerRegistryRequest {
        display_name: "My Registry".to_owned(),
        registry_type: None,
        password_credentials: Some(RegistryCredentials {
            user_name: "testuser".to_owned(),
            password: "test-password".to_owned(),
        }),
    };

    let resp = test_client(&server.uri())
        .add_registry(&body)
        .await
        .unwrap();

    assert_eq!(resp.id, Some(9999));
}

#[tokio::test]
async fn update_registry() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "displayName": "Updated Registry",
            "type": "GitHub"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRY_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = ContainerRegistryRequest {
        display_name: "Updated Registry".to_owned(),
        registry_type: Some(RegistryType::GitHub),
        password_credentials: None,
    };

    let resp = test_client(&server.uri())
        .update_registry(9999, &body)
        .await
        .unwrap();

    assert_eq!(resp.id, Some(9999));
}

#[tokio::test]
async fn delete_registry() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRY_DELETE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .delete_registry(9999)
        .await
        .unwrap();

    use bunny_api_containers::RemoveContainerRegistryStatus;
    assert_eq!(resp.status, RemoveContainerRegistryStatus::Removed);
}

#[tokio::test]
async fn list_container_images() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/registries/images"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({"registryId": "1155"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_CONTAINER_IMAGES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = ListContainerImagesRequest {
        registry_id: "1155".to_owned(),
    };

    let images = test_client(&server.uri())
        .list_container_images(&body)
        .await
        .unwrap();

    assert_eq!(images.len(), 3);
    assert_eq!(images[0].id, "nginx");
    assert_eq!(images[0].namespace, "library");
}

#[tokio::test]
async fn list_container_image_tags() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/registries/tags"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "registryId": "1155",
            "imageName": "nginx",
            "imageNamespace": "library"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_CONTAINER_IMAGE_TAGS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = ListContainerImageTagsRequest {
        registry_id: "1155".to_owned(),
        image_name: "nginx".to_owned(),
        image_namespace: "library".to_owned(),
    };

    let tags = test_client(&server.uri())
        .list_container_image_tags(&body)
        .await
        .unwrap();

    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0].name, "latest");
}

#[tokio::test]
async fn get_container_image_digest() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/registries/digest"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "registryId": "1155",
            "imageName": "nginx",
            "imageNamespace": "library",
            "tag": "alpine"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_IMAGE_DIGEST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = GetContainerImageDigestRequest {
        registry_id: "1155".to_owned(),
        image_name: "nginx".to_owned(),
        image_namespace: "library".to_owned(),
        tag: "alpine".to_owned(),
    };

    let info = test_client(&server.uri())
        .get_container_image_digest(&body)
        .await
        .unwrap();

    assert!(info.digest.as_deref().unwrap().starts_with("sha256:"));
    assert_eq!(info.tag.as_deref(), Some("alpine"));
}

#[tokio::test]
async fn get_config_suggestions() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/registries/config-suggestions"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "registryId": "1155",
            "imageName": "nginx",
            "imageNamespace": "library",
            "tag": "alpine"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CONFIG_SUGGESTIONS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = GetContainerConfigSuggestionsRequest {
        registry_id: "1155".to_owned(),
        image_name: "nginx".to_owned(),
        image_namespace: "library".to_owned(),
        tag: "alpine".to_owned(),
    };

    let suggestions = test_client(&server.uri())
        .get_config_suggestions(&body)
        .await
        .unwrap();

    assert_eq!(suggestions.app_name.as_deref(), Some("NGINX"));
    assert!(!suggestions.environment_variables_suggestions.is_empty());
}

#[tokio::test]
async fn search_public_images() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/registries/public-images/search"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "registryId": "1155",
            "prefix": "nginx"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_PUBLIC_IMAGES_SEARCH, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = SearchPublicContainerImagesRequest {
        registry_id: "1155".to_owned(),
        prefix: "nginx".to_owned(),
        size: None,
        page: None,
    };

    let images = test_client(&server.uri())
        .search_public_images(&body)
        .await
        .unwrap();

    assert_eq!(images.len(), 3);
    assert_eq!(images[1].id, "nginx");
    assert_eq!(images[1].namespace, "library");
}

// ---------------------------------------------------------------------------
// Regions & nodes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_regions() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/regions"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGIONS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_regions(None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0].id, "DE");
    assert!(result.items[0].has_anycast_support);
}

#[tokio::test]
async fn list_regions_with_cursor() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/regions"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param(
            "nextCursor",
            "eyJBUElWZXJzaW9uIjoibWV0YS5ibnkubmV0L3YxIn0",
        ))
        .and(query_param("limit", "5"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGIONS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_regions(Some("eyJBUElWZXJzaW9uIjoibWV0YS5ibnkubmV0L3YxIn0"), Some(5))
        .await
        .unwrap();

    assert!(!result.items.is_empty());
}

#[tokio::test]
async fn get_optimal_region() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/regions/optimal"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_OPTIMAL_REGION, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .get_optimal_region(None)
        .await
        .unwrap();

    assert_eq!(resp.region.id, "DE");
    assert!(resp.region.has_anycast_support);
}

#[tokio::test]
async fn get_optimal_region_with_cdn_token() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/regions/optimal"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("cdnServerToken", "my-cdn-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_OPTIMAL_REGION, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .get_optimal_region(Some("my-cdn-token"))
        .await
        .unwrap();

    assert_eq!(resp.region.id, "DE");
}

#[tokio::test]
async fn list_nodes() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/nodes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_NODES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_nodes(None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0], "104.166.147.46");
    let meta = result.meta.unwrap();
    assert_eq!(meta.total_items, 131);
}

// ---------------------------------------------------------------------------
// Pods
// ---------------------------------------------------------------------------

#[tokio::test]
async fn recreate_pod() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps/l3faCv1fWRHAYNU/pods/eDf3ws1l6Sjtfc/recreate"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .recreate_pod("l3faCv1fWRHAYNU", "eDf3ws1l6Sjtfc")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// User limits
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_user_limits() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/limits"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_USER_LIMITS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let limits = test_client(&server.uri()).get_user_limits().await.unwrap();

    assert_eq!(limits.max_number_of_applications, 10);
    assert_eq!(limits.existing_number_of_applications, 2);
    assert_eq!(limits.max_number_of_volumes_per_application, 2);
}

// ---------------------------------------------------------------------------
// Log forwarding
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_log_forwarding() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/log/forwarding"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_LOG_FORWARDING_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_log_forwarding()
        .await
        .unwrap();

    assert_eq!(result.items.len(), 0);
}

#[tokio::test]
async fn get_log_forwarding() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/log/forwarding/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LOG_FORWARDING_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let cfg = test_client(&server.uri())
        .get_log_forwarding("test-app-id")
        .await
        .unwrap();

    assert_eq!(cfg.id, "lf-123");
    assert_eq!(cfg.app, "test-app-id");
    assert_eq!(cfg.endpoint, "logs.example.com");
    assert_eq!(cfg.port, 514);
    assert!(cfg.enabled);
}

#[tokio::test]
async fn create_log_forwarding() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/log/forwarding"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "app": "test-app-id",
            "type": "SyslogTcp",
            "endpoint": "logs.example.com",
            "port": 514,
            "format": "SyslogRfc5424",
            "enabled": true
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LOG_FORWARDING_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = LogForwardingRequest {
        app: "test-app-id".to_owned(),
        forwarding_type: LogForwardingType::SyslogTcp,
        endpoint: "logs.example.com".to_owned(),
        port: 514,
        token: None,
        format: SyslogFormat::SyslogRfc5424,
        enabled: true,
    };

    let cfg = test_client(&server.uri())
        .create_log_forwarding(&body)
        .await
        .unwrap();

    assert_eq!(cfg.id, "lf-123");
}

#[tokio::test]
async fn update_log_forwarding() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/log/forwarding/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "app": "test-app-id",
            "type": "SyslogTcp",
            "endpoint": "logs.example.com",
            "port": 514,
            "format": "SyslogRfc5424",
            "enabled": false
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LOG_FORWARDING_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = LogForwardingRequest {
        app: "test-app-id".to_owned(),
        forwarding_type: LogForwardingType::SyslogTcp,
        endpoint: "logs.example.com".to_owned(),
        port: 514,
        token: None,
        format: SyslogFormat::SyslogRfc5424,
        enabled: false,
    };

    let cfg = test_client(&server.uri())
        .update_log_forwarding("test-app-id", &body)
        .await
        .unwrap();

    assert_eq!(cfg.app, "test-app-id");
}

#[tokio::test]
async fn delete_log_forwarding() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/log/forwarding/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_log_forwarding("test-app-id")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unauthorized_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_applications(None, None)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("401"), "unexpected error: {err}");
}

#[tokio::test]
async fn not_found_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/nonexistent999"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_ERROR_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_application("nonexistent999")
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("Not Found"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn validation_error_returns_details() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/apps"))
        .respond_with(
            ResponseTemplate::new(400).set_body_raw(FIXTURE_ERROR_VALIDATION, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = AddApplicationRequest {
        name: "bad-app".to_owned(),
        runtime_type: RuntimeType::Shared,
        auto_scaling: AutoscalingSettings { min: 1, max: 1 },
        region_settings: UpdateRegionSettingsRequest {
            allowed_region_ids: None,
            required_region_ids: Some(vec!["DE".to_owned()]),
            max_allowed_regions: None,
            node_selectors: None,
        },
        termination_grace_period_seconds: None,
        repository_settings: None,
        container_templates: None,
        volumes: None,
    };

    let err = test_client(&server.uri())
        .add_application(&body)
        .await
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("Validation Error") || msg.contains("containerTemplates"),
        "unexpected error: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Debug mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn debug_mode_makes_request() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APPS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // With debug enabled the request should still succeed.
    let result = ContainersClient::with_base_url("test-api-key", server.uri())
        .with_debug(true)
        .list_applications(None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}
