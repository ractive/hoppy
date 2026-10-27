//! Magic Containers pull zones for `pull-zone list` / `count`.
//!
//! `GET /pullzone` never returns the pull zones that Magic Containers creates
//! for CDN endpoints (origin type 5), although `GET /pullzone/{id}` and the
//! global search do find them. The OpenAPI spec has no parameter that includes
//! them, so the zone list is completed from the container side:
//! `GET /apps` -> `GET /apps/{id}/endpoints` (each endpoint carries its
//! `pullZoneId`) -> `GET /pullzone/{id}`.
//!
//! An endpoint names only one pull zone, but an app can own more (e.g. a
//! second zone that carries the production hostnames). No API links an app to
//! those, so they are found through the global search, which indexes zone names
//! and hostnames: each app contributes a few search terms (its name, its custom
//! display address), the `cdn` hits are fetched, and a hit is kept only if its
//! `MagicContainersAppId` names one of the apps. See
//! `research/pull-zone-list-magic-containers-2026-10-23.md`.
//!
//! The merge/dedupe logic is pure and unit-tested; the I/O is bounded by
//! [`CONCURRENCY`].

use crate::auth;
use crate::commands::container::parse_pull_zone_id;
use anyhow::{Context, Result};
use bunny_net_api::containers::{AppListItem, ContainersClient, EndpointListItem};
use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::{PullZone, SearchResults};
use futures_util::{StreamExt, TryStreamExt, stream};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// Maximum number of in-flight requests while discovering zones.
const CONCURRENCY: usize = 8;
/// Page size when walking `GET /apps`.
const APP_PAGE_SIZE: i32 = 100;
/// In-flight `GET /search` requests. The search endpoint rate-limits (HTTP 429)
/// far sooner than the others, so the searches run one at a time and a 429 is
/// retried with a short back-off.
const SEARCH_CONCURRENCY: usize = 1;
/// Attempts per search (first try plus retries on HTTP 429).
const SEARCH_ATTEMPTS: u64 = 4;
/// Page size for the per-app global searches.
const SEARCH_PAGE_SIZE: i32 = 100;

/// The container app that owns a pull zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppRef {
    pub id: String,
    pub name: String,
}

/// A pull zone together with the container app that owns it (if any).
///
/// Serialises as the plain `PullZone` object, plus a `ContainerApp` member for
/// Magic Containers zones, so JSON for accounts without containers is unchanged.
#[derive(Debug, Clone, Serialize)]
pub struct ListedZone {
    #[serde(flatten)]
    pub zone: PullZone,
    #[serde(rename = "ContainerApp", skip_serializing_if = "Option::is_none")]
    pub app: Option<AppRef>,
}

impl From<PullZone> for ListedZone {
    fn from(zone: PullZone) -> Self {
        Self { zone, app: None }
    }
}

/// A pull zone id referenced by a container app endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneRef {
    pub pull_zone_id: i64,
    pub app: AppRef,
}

/// Extract the distinct pull zone ids referenced by the endpoints of each app.
/// The first app to reference an id wins; ids of `0`/empty are ignored.
pub fn zone_refs<'a>(apps: impl IntoIterator<Item = &'a AppWithEndpoints>) -> Vec<ZoneRef> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (app, endpoints) in apps {
        for ep in endpoints {
            if let Some(pull_zone_id) = parse_pull_zone_id(ep)
                && seen.insert(pull_zone_id)
            {
                out.push(ZoneRef {
                    pull_zone_id,
                    app: AppRef {
                        id: app.id.clone(),
                        name: app.name.clone(),
                    },
                });
            }
        }
    }
    out
}

/// Result of looking up the container-owned zones.
#[derive(Debug, Default)]
pub struct Discovered {
    /// Zones that were not already known and had to be fetched.
    pub zones: Vec<(PullZone, AppRef)>,
    /// Owners of zones that were already known (and so not fetched again).
    pub known_apps: HashMap<i64, AppRef>,
}

/// Combine the zones returned by `GET /pullzone` with the container-owned
/// zones. Container zones already present in `ordinary` are not duplicated
/// (the existing row is annotated instead). `search` filters the container
/// zones by case-insensitive name substring, mirroring the server-side filter
/// that was applied to `ordinary`.
pub fn merge(ordinary: Vec<PullZone>, found: Discovered, search: Option<&str>) -> Vec<ListedZone> {
    let Discovered {
        zones: container,
        known_apps,
    } = found;
    let mut apps: HashMap<i64, AppRef> = known_apps;
    apps.extend(container.iter().map(|(z, a)| (z.id, a.clone())));
    let mut seen = HashSet::new();
    let mut out: Vec<ListedZone> = Vec::with_capacity(ordinary.len() + container.len());
    for zone in ordinary {
        if seen.insert(zone.id) {
            let app = apps.remove(&zone.id);
            out.push(ListedZone { zone, app });
        }
    }
    let needle = search.map(str::to_lowercase);
    for (zone, app) in container {
        if let Some(n) = &needle
            && !zone.name.to_lowercase().contains(n)
        {
            continue;
        }
        if seen.insert(zone.id) {
            out.push(ListedZone {
                zone,
                app: Some(app),
            });
        }
    }
    out
}

/// A container app together with its endpoints.
pub type AppWithEndpoints = (AppListItem, Vec<EndpointListItem>);

/// Walk `GET /apps` and `GET /apps/{id}/endpoints`. Uses at most
/// [`CONCURRENCY`] concurrent endpoint requests.
pub async fn walk_apps(containers: &ContainersClient) -> Result<Vec<AppWithEndpoints>> {
    let apps = list_apps(containers).await?;
    with_endpoints(containers, apps).await
}

/// Walk the cursor-paged `GET /apps`.
pub async fn list_apps(containers: &ContainersClient) -> Result<Vec<AppListItem>> {
    let mut apps: Vec<AppListItem> = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let page = containers
            .list_applications(cursor.as_deref(), Some(APP_PAGE_SIZE))
            .await
            .context("failed to list container apps")?;
        apps.extend(page.items);
        match page.cursor {
            Some(next) if !next.is_empty() && cursor.as_deref() != Some(next.as_str()) => {
                cursor = Some(next);
            }
            _ => break,
        }
    }
    Ok(apps)
}

/// Attach the endpoints of each app (at most [`CONCURRENCY`] requests in flight).
pub async fn with_endpoints(
    containers: &ContainersClient,
    apps: Vec<AppListItem>,
) -> Result<Vec<AppWithEndpoints>> {
    stream::iter(apps)
        .map(|app| async move {
            let endpoints = containers
                .list_endpoints(&app.id)
                .await
                .with_context(|| format!("failed to list endpoints of app {}", app.id))?;
            Ok::<_, anyhow::Error>((app, endpoints.items))
        })
        .buffered(CONCURRENCY)
        .try_collect()
        .await
}

/// Search terms that can surface the pull zones of `app` in the global search:
/// its name and, when it is not a plain `*.b-cdn.net` host, its display
/// address. Lower-cased, de-duplicated, empty terms dropped.
pub fn search_terms(app: &AppListItem) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    let mut push = |s: &str| {
        let s = s.trim().to_lowercase();
        if !s.is_empty() && !terms.contains(&s) {
            terms.push(s);
        }
    };
    push(&app.name);
    if let Some(d) = &app.display_endpoint
        && !d.address.to_lowercase().ends_with(".b-cdn.net")
    {
        push(&d.address);
    }
    terms
}

/// Fetch the referenced zones (skipping ids in `skip`). Zones that cannot be
/// fetched (e.g. just deleted) are reported on stderr and left out.
pub async fn fetch_zones(core: &CoreClient, refs: Vec<ZoneRef>, skip: &HashSet<i64>) -> Discovered {
    let (known, to_fetch): (Vec<ZoneRef>, Vec<ZoneRef>) = refs
        .into_iter()
        .partition(|r| skip.contains(&r.pull_zone_id));
    let known_apps = known.into_iter().map(|r| (r.pull_zone_id, r.app)).collect();
    let zones = stream::iter(to_fetch)
        .map(|r| async move {
            match core.get_pull_zone(r.pull_zone_id).await {
                Ok(zone) => Some((zone, r.app)),
                Err(e) => {
                    eprintln!(
                        "warning: could not fetch pull zone {} of container app {}: {e:#}",
                        r.pull_zone_id, r.app.id
                    );
                    None
                }
            }
        })
        .buffered(CONCURRENCY)
        .filter_map(|x| async move { x })
        .collect()
        .await;
    Discovered { zones, known_apps }
}

/// `GET /search` with a linear back-off retry on HTTP 429.
async fn search_with_retry(core: &CoreClient, term: &str) -> Result<SearchResults> {
    let mut attempt = 1;
    loop {
        match core.search(term, None, Some(SEARCH_PAGE_SIZE)).await {
            Err(e) if attempt < SEARCH_ATTEMPTS && format!("{e:#}").contains("HTTP 429") => {
                tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt)).await;
                attempt += 1;
            }
            other => return other,
        }
    }
}

/// Pull zone ids of type `cdn` returned by the global search for the terms of
/// `apps`. A failing search is reported on stderr and skipped.
async fn search_candidates(core: &CoreClient, apps: &[&AppListItem]) -> Vec<i64> {
    let mut terms: Vec<String> = Vec::new();
    for app in apps {
        for term in search_terms(app) {
            if !terms.contains(&term) {
                terms.push(term);
            }
        }
    }
    let hits: Vec<Vec<i64>> = stream::iter(terms)
        .map(|term| async move {
            match search_with_retry(core, &term).await {
                Ok(res) => res
                    .search_results
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|r| {
                        r.result_type
                            .as_deref()
                            .is_some_and(|t| t.eq_ignore_ascii_case("cdn"))
                    })
                    .map(|r| r.id)
                    .collect(),
                Err(e) => {
                    eprintln!("warning: could not search for pull zones of '{term}': {e:#}");
                    Vec::new()
                }
            }
        })
        .buffered(SEARCH_CONCURRENCY)
        .collect()
        .await;
    let mut seen = HashSet::new();
    hits.into_iter()
        .flatten()
        .filter(|id| seen.insert(*id))
        .collect()
}

/// Fetch search candidates and keep those that belong to one of `apps` (by
/// `MagicContainersAppId`).
async fn fetch_verified(
    core: &CoreClient,
    ids: Vec<i64>,
    apps: &HashMap<&str, AppRef>,
) -> Vec<(PullZone, AppRef)> {
    stream::iter(ids)
        .map(|id| async move {
            match core.get_pull_zone(id).await {
                Ok(zone) => {
                    let app = zone
                        .magic_containers_app_id
                        .as_deref()
                        .and_then(|a| apps.get(a))?
                        .clone();
                    Some((zone, app))
                }
                Err(e) => {
                    eprintln!("warning: could not fetch pull zone {id}: {e:#}");
                    None
                }
            }
        })
        .buffered(CONCURRENCY)
        .filter_map(|x| async move { x })
        .collect()
        .await
}

/// Discover the pull zones of the given apps (`only`: app ids, `None` = all).
/// Zones named by an endpoint are taken directly; further zones are found via
/// the global search and verified against `MagicContainersAppId`. Ids in
/// `skip` (already listed) are not fetched again.
pub async fn zones_for_apps(
    core: &CoreClient,
    walk: &[AppWithEndpoints],
    only: Option<&HashSet<String>>,
    skip: &HashSet<i64>,
) -> Discovered {
    let selected: Vec<&AppWithEndpoints> = walk
        .iter()
        .filter(|(a, _)| only.is_none_or(|o| o.contains(&a.id)))
        .collect();
    let refs = zone_refs(selected.iter().copied());
    let by_id: HashMap<&str, AppRef> = selected
        .iter()
        .map(|(a, _)| {
            (
                a.id.as_str(),
                AppRef {
                    id: a.id.clone(),
                    name: a.name.clone(),
                },
            )
        })
        .collect();
    let apps: Vec<&AppListItem> = selected.iter().map(|(a, _)| a).collect();

    let ref_ids: HashSet<i64> = refs.iter().map(|r| r.pull_zone_id).collect();
    let mut found = fetch_zones(core, refs, skip).await;
    let candidates: Vec<i64> = search_candidates(core, &apps)
        .await
        .into_iter()
        .filter(|id| !ref_ids.contains(id) && !skip.contains(id))
        .collect();
    found
        .zones
        .extend(fetch_verified(core, candidates, &by_id).await);
    found
}

/// Best-effort discovery of the container-owned pull zones. Failures (no
/// access to the containers API, network errors) are reported on stderr and
/// yield no zones, so `pull-zone list` keeps working.
pub async fn discover_zones(
    core: &CoreClient,
    opts: &auth::ClientOpts<'_>,
    skip: &HashSet<i64>,
) -> Discovered {
    let walk = match auth::containers_client_with_reveal(opts) {
        Ok(containers) => match walk_apps(&containers).await {
            Ok(walk) => walk,
            Err(e) => {
                eprintln!(
                    "warning: could not look up Magic Containers pull zones ({e:#}); \
                     the list may omit them (pass --no-container-zones to silence this)"
                );
                return Discovered::default();
            }
        },
        Err(e) => {
            eprintln!("warning: could not look up Magic Containers pull zones ({e:#})");
            return Discovered::default();
        }
    };
    zones_for_apps(core, &walk, None, skip).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone(id: i64, name: &str) -> PullZone {
        serde_json::from_value(serde_json::json!({ "Id": id, "Name": name }))
            .expect("minimal PullZone deserialises")
    }

    fn app(id: &str, name: &str) -> AppListItem {
        serde_json::from_value(serde_json::json!({
            "id": id, "name": name, "status": "active"
        }))
        .expect("AppListItem deserialises")
    }

    fn endpoint(id: &str, pull_zone_id: &str) -> EndpointListItem {
        serde_json::from_value(serde_json::json!({
            "id": id, "displayName": "cdn", "publicHost": "x.b-cdn.net",
            "type": "cdn", "isSslEnabled": true, "pullZoneId": pull_zone_id,
            "portMappings": []
        }))
        .expect("EndpointListItem deserialises")
    }

    fn found(zones: Vec<(PullZone, AppRef)>) -> Discovered {
        Discovered {
            zones,
            known_apps: HashMap::new(),
        }
    }

    fn aref(id: &str, name: &str) -> AppRef {
        AppRef {
            id: id.into(),
            name: name.into(),
        }
    }

    #[test]
    fn zone_refs_dedupes_and_ignores_empty_and_zero() {
        let apps = vec![
            (
                app("a1", "first"),
                vec![
                    endpoint("e1", "100"),
                    endpoint("e2", "0"),
                    endpoint("e3", ""),
                ],
            ),
            (
                app("a2", "second"),
                vec![endpoint("e4", "100"), endpoint("e5", "200")],
            ),
        ];
        assert_eq!(
            zone_refs(&apps),
            vec![
                ZoneRef {
                    pull_zone_id: 100,
                    app: aref("a1", "first")
                },
                ZoneRef {
                    pull_zone_id: 200,
                    app: aref("a2", "second")
                },
            ]
        );
    }

    #[test]
    fn search_terms_use_name_and_custom_address_once() {
        let mut a = app("a1", "Study2Go");
        assert_eq!(search_terms(&a), vec!["study2go"]);
        a.display_endpoint = serde_json::from_value(serde_json::json!({
            "id": "e", "address": "mc-x.b-cdn.net", "type": "cdn"
        }))
        .expect("DisplayEndpoint deserialises");
        assert_eq!(search_terms(&a), vec!["study2go"]);
        a.display_endpoint = serde_json::from_value(serde_json::json!({
            "id": "e", "address": "Study2Go", "type": "cdn"
        }))
        .expect("DisplayEndpoint deserialises");
        assert_eq!(search_terms(&a), vec!["study2go"]);
        a.display_endpoint = serde_json::from_value(serde_json::json!({
            "id": "e", "address": "www.study2go.ch", "type": "cdn"
        }))
        .expect("DisplayEndpoint deserialises");
        assert_eq!(search_terms(&a), vec!["study2go", "www.study2go.ch"]);
    }

    #[test]
    fn merge_appends_container_zones_and_marks_them() {
        let merged = merge(
            vec![zone(1, "plain")],
            found(vec![(zone(2, "mc-abc"), aref("a1", "app"))]),
            None,
        );
        assert_eq!(merged.len(), 2);
        assert!(merged[0].app.is_none());
        assert_eq!(merged[1].zone.id, 2);
        assert_eq!(merged[1].app, Some(aref("a1", "app")));
    }

    #[test]
    fn merge_does_not_duplicate_zones_already_listed() {
        let merged = merge(
            vec![zone(2, "mc-abc")],
            found(vec![(zone(2, "mc-abc"), aref("a1", "app"))]),
            None,
        );
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].app, Some(aref("a1", "app")));
    }

    #[test]
    fn merge_annotates_known_zones_from_known_apps() {
        let merged = merge(
            vec![zone(2, "mc-abc")],
            Discovered {
                zones: Vec::new(),
                known_apps: HashMap::from([(2, aref("a1", "app"))]),
            },
            None,
        );
        assert_eq!(merged[0].app, Some(aref("a1", "app")));
    }

    #[test]
    fn merge_applies_search_to_container_zones_only() {
        let merged = merge(
            vec![zone(1, "other")],
            found(vec![
                (zone(2, "mc-abc"), aref("a1", "app")),
                (zone(3, "mc-xyz"), aref("a2", "app2")),
            ]),
            Some("ABC"),
        );
        let ids: Vec<i64> = merged.iter().map(|l| l.zone.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn listed_zone_json_only_has_container_app_for_container_zones() {
        let plain = serde_json::to_value(ListedZone::from(zone(1, "p"))).expect("serialises");
        assert!(plain.get("ContainerApp").is_none());
        let mc = serde_json::to_value(ListedZone {
            zone: zone(2, "mc"),
            app: Some(aref("a1", "app")),
        })
        .expect("serialises");
        assert_eq!(mc["ContainerApp"]["Id"], "a1");
        assert_eq!(mc["Id"], 2);
    }
}
