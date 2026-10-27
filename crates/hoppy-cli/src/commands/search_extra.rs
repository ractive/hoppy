//! Client-side additions to `hoppy search`.
//!
//! `GET /search` only indexes DNS zones, pull zones and storage zones by name.
//! Container apps and databases are therefore matched here (case-insensitive
//! substring on the name), and the pull zones of matched container apps are
//! added as `cdn` results (see [`super::pull_zone_containers`]). Every lookup is
//! best effort: a failure is a warning on stderr, never an error.

use super::pull_zone_containers::{is_pull_zone_type, list_apps, with_endpoints, zones_for_apps};
use crate::auth::{self, ClientOpts};
use bunny_net_api::containers::AppListItem;
use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::SearchResultItem;
use bunny_net_api::database::types::Database;
use serde::Serialize;
use std::collections::HashSet;

/// Identifier of an extra result: numeric for `cdn` (like `/search` hits),
/// a string for container apps and databases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ExtraId {
    Num(i64),
    Str(String),
}

impl std::fmt::Display for ExtraId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{n}"),
            Self::Str(s) => f.write_str(s),
        }
    }
}

/// A search result that `GET /search` did not return.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExtraResult {
    pub r#type: String,
    pub id: ExtraId,
    pub name: String,
}

/// Case-insensitive substring match; an empty (or blank) query matches nothing.
fn name_matches(name: &str, query: &str) -> bool {
    let q = query.trim();
    !q.is_empty() && name.to_lowercase().contains(&q.to_lowercase())
}

/// Container apps whose name contains `query`.
pub fn match_apps<'a>(apps: &'a [AppListItem], query: &str) -> Vec<&'a AppListItem> {
    apps.iter()
        .filter(|a| name_matches(&a.name, query))
        .collect()
}

/// `container` results for `apps`.
pub fn app_results(apps: &[&AppListItem]) -> Vec<ExtraResult> {
    apps.iter()
        .map(|a| ExtraResult {
            r#type: "container".into(),
            id: ExtraId::Str(a.id.clone()),
            name: a.name.clone(),
        })
        .collect()
}

/// `db` results for the databases whose name contains `query`.
pub fn database_results(dbs: &[Database], query: &str) -> Vec<ExtraResult> {
    dbs.iter()
        .filter(|d| name_matches(&d.name, query))
        .map(|d| ExtraResult {
            r#type: "db".into(),
            id: ExtraId::Str(d.id.clone()),
            name: d.name.clone(),
        })
        .collect()
}

/// Drop extras already present in the server hits (same type and id) and
/// duplicates among the extras themselves.
pub fn dedupe(extra: Vec<ExtraResult>, server: &[SearchResultItem]) -> Vec<ExtraResult> {
    let mut seen: HashSet<(String, String)> = server
        .iter()
        .map(|s| {
            (
                s.result_type.clone().unwrap_or_default().to_lowercase(),
                s.id.to_string(),
            )
        })
        .collect();
    extra
        .into_iter()
        .filter(|e| seen.insert((e.r#type.to_lowercase(), e.id.to_string())))
        .collect()
}

fn is_cdn(item: &SearchResultItem) -> bool {
    item.result_type.as_deref().is_some_and(is_pull_zone_type)
}

async fn container_results(
    core: &CoreClient,
    opts: &ClientOpts<'_>,
    query: &str,
    server: &[SearchResultItem],
) -> Vec<ExtraResult> {
    let containers = match auth::containers_client_with_reveal(opts) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: could not search container apps ({e:#})");
            return Vec::new();
        }
    };
    let apps = match list_apps(&containers).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("warning: could not search container apps ({e:#})");
            return Vec::new();
        }
    };
    let matched: Vec<AppListItem> = match_apps(&apps, query).into_iter().cloned().collect();
    if matched.is_empty() {
        return Vec::new();
    }
    let mut out = app_results(&matched.iter().collect::<Vec<_>>());
    match with_endpoints(&containers, matched).await {
        Ok(walk) => {
            let skip: HashSet<i64> = server.iter().filter(|s| is_cdn(s)).map(|s| s.id).collect();
            let found = zones_for_apps(core, &walk, &skip).await;
            out.extend(found.zones.into_iter().map(|(zone, _)| ExtraResult {
                r#type: "cdn".into(),
                id: ExtraId::Num(zone.id),
                name: zone.name,
            }));
        }
        Err(e) => eprintln!("warning: could not look up pull zones of matched apps ({e:#})"),
    }
    out
}

async fn db_results(opts: &ClientOpts<'_>, query: &str) -> Vec<ExtraResult> {
    let client = match auth::database_client_with_reveal(opts) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: could not search databases ({e:#})");
            return Vec::new();
        }
    };
    match client.list_databases(None).await {
        Ok(resp) => database_results(&resp.databases, query),
        Err(e) => {
            eprintln!("warning: could not search databases ({e:#})");
            Vec::new()
        }
    }
}

/// Container apps, databases and the pull zones of matched apps that match
/// `query` and are not already among the server `hits`.
pub async fn extra_results(
    core: &CoreClient,
    opts: &ClientOpts<'_>,
    query: &str,
    server: &[SearchResultItem],
) -> Vec<ExtraResult> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let (mut containers, dbs) = tokio::join!(
        container_results(core, opts, query, server),
        db_results(opts, query)
    );
    containers.extend(dbs);
    dedupe(containers, server)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(id: &str, name: &str) -> AppListItem {
        serde_json::from_value(serde_json::json!({
            "id": id, "name": name, "status": "active"
        }))
        .expect("AppListItem deserialises")
    }

    fn hit(ty: &str, id: i64) -> SearchResultItem {
        SearchResultItem {
            result_type: Some(ty.into()),
            id,
            name: Some("x".into()),
        }
    }

    #[test]
    fn is_cdn_accepts_live_and_fixture_type_names() {
        assert!(is_cdn(&hit("cdn", 1)));
        assert!(is_cdn(&hit("PullZone", 1)));
        assert!(!is_cdn(&hit("DnsZone", 1)));
    }

    #[test]
    fn name_matching_is_case_insensitive_substring_and_ignores_blank() {
        assert!(name_matches("My-Study2Go-app", "study2GO"));
        assert!(!name_matches("other", "study2go"));
        assert!(!name_matches("anything", "  "));
        assert!(!name_matches("anything", ""));
    }

    #[test]
    fn match_apps_filters_by_name() {
        let apps = vec![app("a1", "study2go"), app("a2", "tab-mcp")];
        let m = match_apps(&apps, "STUDY");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].id, "a1");
        let r = app_results(&m);
        assert_eq!(r[0].r#type, "container");
        assert_eq!(r[0].id, ExtraId::Str("a1".into()));
    }

    #[test]
    fn dedupe_drops_server_hits_and_repeats() {
        let extra = vec![
            ExtraResult {
                r#type: "cdn".into(),
                id: ExtraId::Num(1),
                name: "a".into(),
            },
            ExtraResult {
                r#type: "cdn".into(),
                id: ExtraId::Num(2),
                name: "b".into(),
            },
            ExtraResult {
                r#type: "cdn".into(),
                id: ExtraId::Num(2),
                name: "b".into(),
            },
        ];
        let out = dedupe(extra, &[hit("cdn", 1), hit("dns", 2)]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, ExtraId::Num(2));
    }

    #[test]
    fn extra_result_serialises_pascal_case_with_native_id() {
        let v = serde_json::to_value(ExtraResult {
            r#type: "cdn".into(),
            id: ExtraId::Num(7),
            name: "n".into(),
        })
        .expect("serialises");
        assert_eq!(v, serde_json::json!({"Type": "cdn", "Id": 7, "Name": "n"}));
        let v = serde_json::to_value(ExtraResult {
            r#type: "db".into(),
            id: ExtraId::Str("abc".into()),
            name: "n".into(),
        })
        .expect("serialises");
        assert_eq!(v["Id"], "abc");
    }
}
