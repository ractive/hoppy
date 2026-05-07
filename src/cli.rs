use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Long-form `--version` string: includes the build SHA and the bunny API
/// spec date the client crates were generated against. Useful for bug reports.
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (sha=",
    env!("HOPPY_BUILD_SHA"),
    ", bunny-api-spec=",
    env!("HOPPY_BUNNY_API_SPEC_DATE"),
    ")"
);

#[derive(Parser)]
#[command(
    name = "hoppy",
    version = env!("CARGO_PKG_VERSION"),
    long_version = LONG_VERSION,
    about = "CLI for bunny.net services",
    long_about = "A CLI tool for managing bunny.net cloud and edge services.\n\nSet the BUNNY_API_KEY environment variable to authenticate."
)]
pub struct Cli {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub format: OutputFormat,

    /// Enable debug output (shows HTTP requests)
    #[arg(long, global = true)]
    pub debug: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Skip confirmation prompts
    #[arg(long, short = 'y', global = true)]
    pub yes: bool,

    /// Reveal redacted secrets in output (env values, passwords, tokens).
    /// Off by default for safety; opt in explicitly.
    #[arg(long, global = true)]
    pub reveal: bool,

    /// Reveal a specific env-var by name (case-insensitive). Repeatable.
    /// Useful when you want one variable but not all secrets.
    #[arg(long = "reveal-env", value_name = "KEY", global = true)]
    pub reveal_env: Vec<String>,

    /// Record API responses to files in the given directory
    #[arg(long, value_name = "DIR", global = true)]
    pub record: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Text,
}

/// Pull Zone billing/performance tier (`Type` in the bunny.net API).
#[derive(Copy, Clone, ValueEnum)]
pub enum ZoneTier {
    /// `Type=0` — single, optimised global network (default).
    Premium,
    /// `Type=1` — cheaper, optimised for high-traffic static assets.
    Volume,
}

impl From<ZoneTier> for bunny_api_core::types::PullZoneType {
    fn from(t: ZoneTier) -> Self {
        match t {
            ZoneTier::Premium => Self::Premium,
            ZoneTier::Volume => Self::Volume,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage CDN pull zones
    PullZone {
        #[command(subcommand)]
        action: PullZoneAction,
    },

    /// Manage storage zones
    StorageZone {
        #[command(subcommand)]
        action: StorageZoneAction,
    },

    /// Manage storage files
    Storage {
        #[command(subcommand)]
        action: StorageAction,
    },

    /// Manage DNS zones and records
    Dns {
        #[command(subcommand)]
        action: DnsAction,
    },

    /// Manage video streaming
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },

    /// Manage security (WAF, DDoS, rate limiting)
    Shield {
        #[command(subcommand)]
        action: ShieldAction,
    },

    /// Manage edge scripts
    Script {
        #[command(subcommand)]
        action: ScriptAction,
    },

    /// Manage magic containers
    Container {
        #[command(subcommand)]
        action: ContainerAction,
    },

    /// Manage Bunny Databases (libSQL)
    Db {
        #[command(subcommand)]
        action: DbAction,
    },

    /// Authenticate and validate API key
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// View account-level CDN statistics
    Statistics {
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
        /// Filter by pull zone ID
        #[arg(long)]
        pull_zone: Option<i64>,
        /// Show hourly granularity
        #[arg(long)]
        hourly: bool,
    },

    /// Manage video libraries (core API — DRM and transcription stats)
    VideoLibrary {
        #[command(subcommand)]
        action: VideoLibraryAction,
    },

    /// Purge a URL from CDN cache
    Purge {
        /// The URL to purge
        #[arg(long)]
        url: String,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

// -- Auth --

#[derive(Subcommand)]
pub enum AuthAction {
    /// Validate the API key and display account billing info
    Check,
}

// -- Video Library (core API — DRM and transcription stats) --

#[derive(Subcommand)]
pub enum VideoLibraryAction {
    /// Get DRM statistics for a video library
    DrmStatistics {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
    },
    /// Get transcribing statistics for a video library
    TranscribingStatistics {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
    },
}

// -- Pull Zone --

#[derive(Subcommand)]
pub enum PullZoneAction {
    /// List all pull zones
    List {
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,
        /// Items per page
        #[arg(long)]
        per_page: Option<u32>,
    },
    /// Get a specific pull zone
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Create a new pull zone.
    ///
    /// Exactly one of --origin-url or --storage-zone-id must be supplied.
    /// Storage-Zone-backed Pull Zones are the most common static-files
    /// setup; HTTP/HTTPS origins use --origin-url.
    #[command(group = clap::ArgGroup::new("origin")
        .required(true)
        .args(["origin_url", "storage_zone_id"]))]
    #[command(after_help = "EXAMPLES:
  # CDN in front of a public HTTP origin
  hoppy pull-zone create --name my-zone --origin-url https://origin.example.com

  # CDN in front of a Storage Zone (use storage-zone create first)
  hoppy pull-zone create --name my-zone --storage-zone-id 1234

  # After creation, attach a custom hostname and load a free Let's Encrypt cert
  hoppy pull-zone hostname add --id <pz-id> --hostname cdn.example.com
  hoppy pull-zone hostname load-free-cert --hostname cdn.example.com")]
    Create {
        #[arg(long)]
        name: String,
        /// HTTP/HTTPS origin URL the Pull Zone fetches from. Mutually
        /// exclusive with --storage-zone-id.
        #[arg(long, group = "origin")]
        origin_url: Option<String>,
        /// Numeric Storage Zone ID to bind this Pull Zone to. Use
        /// `hoppy storage-zone list` to find IDs. Mutually exclusive with
        /// --origin-url.
        #[arg(long, group = "origin")]
        storage_zone_id: Option<i64>,
        /// Pull Zone billing/performance tier:
        /// `premium` (default — single, optimised network) or
        /// `volume` (cheaper, optimised for high-traffic static assets).
        #[arg(long, value_enum, default_value_t = ZoneTier::Premium)]
        zone_tier: ZoneTier,
    },
    /// Update a pull zone
    Update {
        #[arg(long)]
        id: i64,
        /// HTTP/HTTPS origin URL. Mutually exclusive with --storage-zone-id.
        #[arg(long, conflicts_with = "storage_zone_id")]
        origin_url: Option<String>,
        /// Re-bind the Pull Zone to a different Storage Zone. Mutually
        /// exclusive with --origin-url.
        #[arg(long)]
        storage_zone_id: Option<i64>,
        #[arg(long)]
        monthly_bandwidth_limit: Option<i64>,
        #[arg(long)]
        cache_expiration_time: Option<i64>,
        #[arg(long)]
        zone_security_enabled: Option<bool>,
        #[arg(long)]
        enable_geo_zone_us: Option<bool>,
        #[arg(long)]
        enable_geo_zone_eu: Option<bool>,
        #[arg(long)]
        enable_geo_zone_asia: Option<bool>,
        #[arg(long)]
        enable_geo_zone_sa: Option<bool>,
        #[arg(long)]
        enable_geo_zone_af: Option<bool>,
    },
    /// Delete a pull zone
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// Purge pull zone cache
    Purge {
        #[arg(long)]
        id: i64,
        /// Limit purge to a specific cache tag
        #[arg(long)]
        cache_tag: Option<String>,
    },
    /// Get pull zone statistics
    Statistics {
        #[arg(long)]
        id: i64,
        /// Statistics type: optimizer, origin-shield, safehop
        #[arg(long, value_name = "TYPE")]
        r#type: String,
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
        /// Show hourly granularity
        #[arg(long)]
        hourly: bool,
    },
    /// Manage pull zone hostnames and SSL
    Hostname {
        #[command(subcommand)]
        action: PullZoneHostnameAction,
    },
    /// Manage pull zone edge rules
    EdgeRule {
        #[command(subcommand)]
        action: EdgeRuleAction,
    },
    /// Manage pull zone referrer access control (anti-hotlinking)
    Referrer {
        #[command(subcommand)]
        action: PullZoneReferrerAction,
    },
    /// Manage pull zone IP block list
    Ip {
        #[command(subcommand)]
        action: PullZoneIpAction,
    },
}

#[derive(Subcommand)]
pub enum PullZoneReferrerAction {
    /// List allowed and blocked referrers for a pull zone
    List {
        #[arg(long)]
        id: i64,
    },
    /// Add a hostname to the allowed referrer list
    Allow {
        #[arg(long)]
        id: i64,
        /// Referrer hostname pattern (e.g. example.com or *.example.com)
        #[arg(long)]
        value: String,
    },
    /// Remove a hostname from the allowed referrer list
    RemoveAllowed {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        value: String,
    },
    /// Add a hostname to the blocked referrer list
    Block {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        value: String,
    },
    /// Remove a hostname from the blocked referrer list
    RemoveBlocked {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        value: String,
    },
}

#[derive(Subcommand)]
pub enum PullZoneIpAction {
    /// List blocked IPs for a pull zone
    List {
        #[arg(long)]
        id: i64,
    },
    /// Block an IP address (single IP or CIDR range)
    Block {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        value: String,
    },
    /// Unblock an IP address
    Unblock {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        value: String,
    },
}

#[derive(Subcommand)]
pub enum EdgeRuleAction {
    /// List edge rules on a pull zone
    List {
        #[arg(long)]
        id: i64,
    },
    /// Add an edge rule to a pull zone
    Add {
        #[arg(long)]
        id: i64,
        /// Rule description
        #[arg(long)]
        description: Option<String>,
        /// Action type (e.g. redirect, block-request, set-response-header)
        #[arg(long)]
        action_type: String,
        /// First action parameter (meaning depends on action type)
        #[arg(long)]
        action_param1: Option<String>,
        /// Second action parameter (meaning depends on action type)
        #[arg(long)]
        action_param2: Option<String>,
        /// How triggers are combined: match-any (default), match-all, match-none
        #[arg(long, default_value = "match-any")]
        trigger_matching_type: String,
        /// Trigger in type:pattern1,pattern2 format (repeatable)
        #[arg(long = "trigger")]
        triggers: Vec<String>,
    },
    /// Update an existing edge rule
    Update {
        #[arg(long)]
        id: i64,
        /// GUID of the edge rule to update
        #[arg(long)]
        rule_id: String,
        /// Rule description
        #[arg(long)]
        description: Option<String>,
        /// Action type (e.g. redirect, block-request, set-response-header)
        #[arg(long)]
        action_type: String,
        /// First action parameter (meaning depends on action type)
        #[arg(long)]
        action_param1: Option<String>,
        /// Second action parameter (meaning depends on action type)
        #[arg(long)]
        action_param2: Option<String>,
        /// How triggers are combined: match-any (default), match-all, match-none
        #[arg(long, default_value = "match-any")]
        trigger_matching_type: String,
        /// Trigger in type:pattern1,pattern2 format (repeatable)
        #[arg(long = "trigger")]
        triggers: Vec<String>,
    },
    /// Delete an edge rule from a pull zone
    Delete {
        #[arg(long)]
        id: i64,
        /// GUID of the edge rule to delete
        #[arg(long)]
        rule_id: String,
    },
    /// Enable or disable an edge rule
    Enable {
        #[arg(long)]
        id: i64,
        /// GUID of the edge rule
        #[arg(long)]
        rule_id: String,
        /// Whether to enable (true) or disable (false) the rule
        #[arg(long, action = clap::ArgAction::Set)]
        enabled: bool,
    },
}

#[derive(Subcommand)]
pub enum PullZoneHostnameAction {
    /// Add a custom hostname.
    ///
    /// EXAMPLES:
    ///   # 1. Attach the hostname
    ///   hoppy pull-zone hostname add --id 1001 --hostname cdn.example.com
    ///
    ///   # 2. Issue a free Let's Encrypt cert (CNAME must already point at b-cdn.net)
    ///   hoppy pull-zone hostname load-free-cert --hostname cdn.example.com
    ///
    ///   # 3. Force HTTPS
    ///   hoppy pull-zone hostname force-ssl --id 1001 \
    ///     --hostname cdn.example.com --enabled true
    Add {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        hostname: String,
    },
    /// Remove a custom hostname
    Remove {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        hostname: String,
    },
    /// Load a free Let's Encrypt SSL certificate
    LoadFreeCert {
        /// The hostname to issue the certificate for
        #[arg(long)]
        hostname: String,
    },
    /// Set Force SSL on a hostname
    ForceSsl {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        hostname: String,
        /// Enable or disable Force SSL
        #[arg(long, action = clap::ArgAction::Set)]
        enabled: bool,
    },
    /// Add a custom SSL certificate (certificate and key must be Base64-encoded PEM)
    AddCert {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        hostname: String,
        /// Base64-encoded PEM certificate
        #[arg(long)]
        certificate: String,
        /// Base64-encoded PEM private key
        #[arg(long)]
        key: String,
    },
    /// Remove the SSL certificate from a hostname
    RemoveCert {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        hostname: String,
    },
}

// -- Storage Zone --

#[derive(Subcommand)]
pub enum StorageZoneAction {
    /// List all storage zones
    List {
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,
        /// Items per page
        #[arg(long)]
        per_page: Option<u32>,
    },
    /// Get a specific storage zone
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Create a new storage zone.
    ///
    /// EXAMPLES:
    ///   # Create a zone in Frankfurt
    ///   hoppy storage-zone create --name my-assets --region DE
    ///
    ///   # Create a multi-region zone
    ///   hoppy storage-zone create --name global-assets --region NY \
    ///     --replication-regions DE,SG,SYD
    ///
    ///   # After creation, retrieve credentials with `--reveal`:
    ///   hoppy storage-zone get --reveal --id <id>
    Create {
        #[arg(long)]
        name: String,
        /// Primary region (e.g. DE, NY, LA, SG, SYD)
        #[arg(long)]
        region: String,
        /// Replication regions (comma-separated or repeated flags)
        #[arg(long, value_delimiter = ',')]
        replication_regions: Vec<String>,
        /// Zone tier (0 = Standard, 1 = Edge)
        #[arg(long)]
        zone_tier: Option<i64>,
    },
    /// Update a storage zone
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        rewrite_404_to_200: Option<bool>,
        #[arg(long)]
        custom_404_file_path: Option<String>,
        #[arg(long)]
        origin_url: Option<String>,
    },
    /// Delete a storage zone
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// Get statistics for a storage zone
    Statistics {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
    },
}

// -- Storage (file operations) --

#[derive(Subcommand)]
pub enum StorageAction {
    /// Upload a file
    Upload {
        /// Storage zone name
        #[arg(long)]
        zone: String,
        /// Remote path (e.g. images/photo.jpg)
        #[arg(long)]
        remote_path: String,
        /// Local file to upload
        #[arg(long)]
        file: String,
        /// Storage region hostname prefix (e.g. storage, la, sg, syd)
        #[arg(long, default_value = "storage")]
        region: String,
    },
    /// Download a file
    Download {
        /// Storage zone name
        #[arg(long)]
        zone: String,
        /// Remote path (e.g. images/photo.jpg)
        #[arg(long)]
        remote_path: String,
        /// Local path to write the file (defaults to stdout)
        #[arg(long)]
        output: Option<String>,
        /// Storage region hostname prefix (e.g. storage, la, sg, syd)
        #[arg(long, default_value = "storage")]
        region: String,
    },
    /// List files
    Ls {
        /// Storage zone name
        #[arg(long)]
        zone: String,
        /// Remote directory path (empty for root)
        #[arg(long, default_value = "")]
        path: String,
        /// Storage region hostname prefix (e.g. storage, la, sg, syd)
        #[arg(long, default_value = "storage")]
        region: String,
    },
    /// Delete a file
    Rm {
        /// Storage zone name
        #[arg(long)]
        zone: String,
        /// Remote path (e.g. images/photo.jpg)
        #[arg(long)]
        remote_path: String,
        /// Storage region hostname prefix (e.g. storage, la, sg, syd)
        #[arg(long, default_value = "storage")]
        region: String,
    },
}

// -- DNS --

#[derive(Subcommand)]
pub enum DnsAction {
    /// Manage DNS zones
    Zone {
        #[command(subcommand)]
        action: DnsZoneAction,
    },
    /// Manage DNS records
    Record {
        #[command(subcommand)]
        action: DnsRecordAction,
    },
}

#[derive(Subcommand)]
pub enum DnsZoneAction {
    /// List DNS zones
    List {
        /// Filter by domain
        #[arg(long)]
        search: Option<String>,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,
        /// Items per page
        #[arg(long)]
        per_page: Option<u32>,
    },
    /// Get a specific DNS zone
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Create a DNS zone
    Create {
        #[arg(long)]
        domain: String,
    },
    /// Update a DNS zone
    Update {
        #[arg(long)]
        id: i64,
        /// Enable custom nameservers
        #[arg(long)]
        custom_nameservers_enabled: Option<bool>,
        /// Primary nameserver
        #[arg(long)]
        nameserver1: Option<String>,
        /// Secondary nameserver
        #[arg(long)]
        nameserver2: Option<String>,
        /// SOA email address
        #[arg(long)]
        soa_email: Option<String>,
        /// Enable query logging
        #[arg(long)]
        logging_enabled: Option<bool>,
        /// Enable IP anonymization in logs
        #[arg(long)]
        logging_ip_anonymization_enabled: Option<bool>,
    },
    /// Delete a DNS zone
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// Get statistics for a DNS zone
    Statistics {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
    },
    /// Export DNS zone as a BIND zone file
    Export {
        #[arg(long)]
        id: i64,
    },
    /// Import DNS records from a BIND zone file
    Import {
        #[arg(long)]
        id: i64,
        /// Path to zone file (reads from stdin if omitted)
        #[arg(long)]
        file: Option<String>,
    },
    /// Manage DNSSEC for a DNS zone
    Dnssec {
        #[command(subcommand)]
        action: DnsDnssecAction,
    },
    /// Issue a free wildcard TLS certificate for a DNS zone.
    ///
    /// The zone must be properly delegated to bunny.net nameservers — the
    /// certificate authority needs to validate the domain via DNS challenge.
    /// If the zone isn't delegated, the API returns an error.
    IssueCert {
        #[arg(long)]
        id: i64,
    },
    /// Scan a zone (or domain) for pre-existing DNS records and view results.
    ///
    /// Scans run asynchronously: `scan start` triggers a job and returns
    /// immediately, `scan results` fetches the latest job's findings.
    Scan {
        #[command(subcommand)]
        action: DnsScanAction,
    },
}

#[derive(Subcommand)]
pub enum DnsDnssecAction {
    /// Enable DNSSEC and display the DS record details to copy to your registrar.
    Enable {
        #[arg(long)]
        id: i64,
    },
    /// Disable DNSSEC.
    ///
    /// WARNING: if DS records are still configured at your registrar,
    /// disabling DNSSEC at bunny.net will break resolution. Remove the DS
    /// records from your registrar first.
    Disable {
        #[arg(long)]
        id: i64,
    },
    /// Show the current DNSSEC status (read from the DNS zone metadata).
    Status {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum DnsScanAction {
    /// Trigger a background record-discovery scan.
    ///
    /// Provide either `--id <zone-id>` for an existing zone or `--domain
    /// <domain>` to scan before creating the zone (but not both).
    Start {
        /// DNS Zone ID (use this for existing zones)
        #[arg(long, conflicts_with = "domain")]
        id: Option<i64>,
        /// Domain name (use this for pre-zone-creation scans)
        #[arg(long, conflicts_with = "id")]
        domain: Option<String>,
    },
    /// Show the latest scan results for a zone.
    Results {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum DnsRecordAction {
    /// List records in a DNS zone
    List {
        #[arg(long)]
        zone_id: i64,
    },
    /// Add a DNS record.
    ///
    /// EXAMPLES:
    ///   hoppy dns record add --zone-id 50001 --type A    --value 192.0.2.1
    ///   hoppy dns record add --zone-id 50001 --type CNAME --name www --value example.com
    ///   hoppy dns record add --zone-id 50001 --type MX    --value mail.example.com --priority 10
    ///   hoppy dns record add --zone-id 50001 --type CAA   --value letsencrypt.org --tag issue --flags 0
    ///
    /// Tip: for Magic-Container-backed Pull Zones, use a `CNAME` record
    /// pointing at the `b-cdn.net` hostname instead of `--type PullZone`
    /// (the latter only accepts standard, non-managed Pull Zone IDs and
    /// will return "pull zone ID is not valid" otherwise).
    Add {
        #[arg(long)]
        zone_id: i64,
        /// Record type (case-insensitive). Commonly used: A, AAAA, CNAME,
        /// TXT, MX, SRV, CAA, PTR, NS. Also accepted by the bunny API:
        /// Redirect, Flatten, PullZone, Script, SVCB, HTTPS, TLSA. Some
        /// of these are smart-routing-only (`Flatten`, `PullZone`) and may
        /// fail with the API's own error if the zone or value is incompatible.
        #[arg(long, value_name = "TYPE")]
        r#type: String,
        /// Record name (subdomain, omit for apex)
        #[arg(long)]
        name: Option<String>,
        /// Record value (IP address, hostname, text, etc.)
        #[arg(long)]
        value: String,
        /// TTL in seconds
        #[arg(long)]
        ttl: Option<i32>,
        /// Priority (for MX, SRV)
        #[arg(long)]
        priority: Option<i32>,
        /// Weight (for weighted/smart routing)
        #[arg(long)]
        weight: Option<i32>,
        /// Port (for SRV records)
        #[arg(long)]
        port: Option<i32>,
        /// Flags (for CAA records)
        #[arg(long)]
        flags: Option<u8>,
        /// Tag (for CAA records, e.g. "issue", "issuewild")
        #[arg(long)]
        tag: Option<String>,
        /// Comment
        #[arg(long)]
        comment: Option<String>,
    },
    /// Update a DNS record
    Update {
        #[arg(long)]
        zone_id: i64,
        #[arg(long)]
        record_id: i64,
        /// Record type (A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA, Redirect, Flatten, PullZone, Script; case-insensitive)
        #[arg(long, value_name = "TYPE")]
        r#type: String,
        /// Record value
        #[arg(long)]
        value: String,
        /// Record name (subdomain)
        #[arg(long)]
        name: Option<String>,
        /// TTL in seconds
        #[arg(long)]
        ttl: Option<i32>,
        /// Priority (for MX, SRV)
        #[arg(long)]
        priority: Option<i32>,
        /// Weight
        #[arg(long)]
        weight: Option<i32>,
        /// Comment
        #[arg(long)]
        comment: Option<String>,
    },
    /// Delete a DNS record
    Delete {
        #[arg(long)]
        zone_id: i64,
        #[arg(long)]
        record_id: i64,
    },
}

// -- Stream --

#[derive(Subcommand)]
pub enum StreamAction {
    /// Manage video libraries
    Library {
        #[command(subcommand)]
        action: StreamLibraryAction,
    },
    /// Manage videos
    Video {
        #[command(subcommand)]
        action: StreamVideoAction,
    },
    /// Manage video collections
    Collection {
        #[command(subcommand)]
        action: StreamCollectionAction,
    },
}

#[derive(Subcommand)]
pub enum StreamLibraryAction {
    /// List video libraries
    List {
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,
        /// Items per page
        #[arg(long)]
        per_page: Option<u32>,
    },
    /// Get a specific video library
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Create a video library
    Create {
        #[arg(long)]
        name: String,
    },
    /// Update a video library
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        allow_direct_play: Option<bool>,
        #[arg(long)]
        enable_mp4_fallback: Option<bool>,
        #[arg(long)]
        has_watermark: Option<bool>,
    },
    /// Delete a video library
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// Get statistics for a video library
    Statistics {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
        /// Show hourly granularity
        #[arg(long)]
        hourly: bool,
        /// Filter by video GUID
        #[arg(long)]
        video_guid: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum StreamVideoAction {
    /// List videos in a library
    List {
        #[arg(long)]
        library_id: i64,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,
        /// Items per page
        #[arg(long)]
        items_per_page: Option<u32>,
        /// Filter by title
        #[arg(long)]
        search: Option<String>,
        /// Filter by collection GUID
        #[arg(long)]
        collection: Option<String>,
        /// Sort order
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Get a specific video
    Get {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
    },
    /// Upload a video file (two-step: create + upload binary)
    Upload {
        #[arg(long)]
        library_id: i64,
        /// Local file path to upload
        #[arg(long)]
        file: String,
        /// Title for the video (defaults to filename)
        #[arg(long)]
        title: Option<String>,
        /// Collection GUID to assign the video to
        #[arg(long)]
        collection_id: Option<String>,
    },
    /// Update video title or collection
    Update {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        /// New title for the video
        #[arg(long)]
        title: Option<String>,
        /// Collection GUID to assign the video to
        #[arg(long)]
        collection_id: Option<String>,
    },
    /// Fetch (ingest) a video from a remote URL
    Fetch {
        #[arg(long)]
        library_id: i64,
        /// Public URL to pull the video from
        #[arg(long)]
        url: String,
        /// Title for the created video
        #[arg(long)]
        title: Option<String>,
    },
    /// Delete a video
    Delete {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
    },
    /// Manage video captions
    Caption {
        #[command(subcommand)]
        action: StreamCaptionAction,
    },
    /// Trigger transcription / translation for a video
    Transcribe {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        /// Force re-transcribe even if captions exist
        #[arg(long)]
        force: bool,
        /// Source language (ISO 639-1, e.g. `en`)
        #[arg(long)]
        language: Option<String>,
        /// Target languages for translation (ISO 639-1, can repeat)
        #[arg(long = "target-language")]
        target_languages: Vec<String>,
        /// Auto-generate title
        #[arg(long)]
        generate_title: bool,
        /// Auto-generate description
        #[arg(long)]
        generate_description: bool,
        /// Auto-generate chapters
        #[arg(long)]
        generate_chapters: bool,
        /// Auto-generate moments
        #[arg(long)]
        generate_moments: bool,
    },
    /// Get the engagement heatmap for a video
    Heatmap {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
    },
    /// Re-encode a video (optionally for a specific codec)
    Reencode {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        /// Optional output codec (x264, vp9, hevc, av1)
        #[arg(long)]
        codec: Option<String>,
    },
    /// Repackage a video's HLS/DASH manifests
    Repackage {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        /// Discard previous file versions (default: keep)
        #[arg(long)]
        discard_originals: bool,
    },
    /// Trigger smart-generate (AI title/description/chapters/moments)
    SmartGenerate {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        #[arg(long)]
        language: Option<String>,
        #[arg(long)]
        generate_title: bool,
        #[arg(long)]
        generate_description: bool,
        #[arg(long)]
        generate_chapters: bool,
        #[arg(long)]
        generate_moments: bool,
    },
    /// Set the thumbnail for a video from a URL
    SetThumbnail {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        #[arg(long)]
        thumbnail_url: String,
    },
    /// Manage video resolutions/encodings
    Resolutions {
        #[command(subcommand)]
        action: StreamResolutionsAction,
    },
    /// Show the storage breakdown for a video
    Storage {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
    },
}

#[derive(Subcommand)]
pub enum StreamResolutionsAction {
    /// List configured/available resolutions for a video
    List {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
    },
    /// Cleanup video resolutions/files (destructive — confirmation required unless --yes or --dry-run)
    Cleanup {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        /// Comma-separated list of resolutions to delete (e.g. `720p,480p`)
        #[arg(long)]
        resolutions: Option<String>,
        /// Delete every rendition not in the library's configured resolutions
        #[arg(long)]
        delete_non_configured: bool,
        /// Delete the original uploaded file
        #[arg(long)]
        delete_original: bool,
        /// Delete MP4 fallback files
        #[arg(long)]
        delete_mp4_files: bool,
        /// Preview only — do not actually delete anything
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum StreamCaptionAction {
    /// Add a caption track to a video
    Add {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        /// BCP 47 language code (e.g. en, de, fr)
        #[arg(long)]
        srclang: String,
        /// Path to SRT subtitle file
        #[arg(long)]
        file: String,
    },
    /// Delete a caption track from a video
    Delete {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
        /// BCP 47 language code
        #[arg(long)]
        srclang: String,
    },
}

#[derive(Subcommand)]
pub enum StreamCollectionAction {
    /// List collections in a library
    List {
        #[arg(long)]
        library_id: i64,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,
        /// Items per page
        #[arg(long)]
        items_per_page: Option<u32>,
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Sort order
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Get a specific collection
    Get {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        collection_id: String,
    },
    /// Create a new collection
    Create {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        name: String,
    },
    /// Update a collection
    Update {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        collection_id: String,
        /// New name for the collection
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a collection
    Delete {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        collection_id: String,
    },
}

// -- Shield --

#[derive(Subcommand)]
pub enum ShieldAction {
    /// Manage Shield Zones
    Zone {
        #[command(subcommand)]
        action: ShieldZoneAction,
    },
    /// Manage WAF rules
    Waf {
        #[command(subcommand)]
        action: ShieldWafAction,
    },
    /// Manage rate limit rules
    RateLimit {
        #[command(subcommand)]
        action: ShieldRateLimitAction,
    },
    /// Manage access lists
    AccessList {
        #[command(subcommand)]
        action: ShieldAccessListAction,
    },
    /// Manage bot detection
    BotDetection {
        #[command(subcommand)]
        action: ShieldBotDetectionAction,
    },
    /// View Shield Zone metrics
    Metrics {
        #[command(subcommand)]
        action: ShieldMetricsAction,
    },
}

#[derive(Subcommand)]
pub enum ShieldMetricsAction {
    /// Get metrics overview for a Shield Zone
    Overview {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get detailed metrics for a Shield Zone (time-series breakdown)
    Detailed {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get rate limit metrics for all rules in a Shield Zone
    RateLimits {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get rate limit metrics for a single rule
    RateLimit {
        #[arg(long)]
        id: i64,
    },
    /// Get WAF rule metrics for a specific rule in a Shield Zone
    WafRule {
        #[arg(long)]
        shield_zone_id: i64,
        #[arg(long)]
        rule_id: i64,
    },
    /// Get bot detection metrics for a Shield Zone
    BotDetection {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get upload scanning metrics for a Shield Zone
    UploadScanning {
        #[arg(long)]
        shield_zone_id: i64,
    },
}

#[derive(Subcommand)]
pub enum ShieldZoneAction {
    /// List all Shield Zones
    List,
    /// Get a Shield Zone by ID
    Get {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get a Shield Zone by Pull Zone ID
    GetByPullzone {
        #[arg(long)]
        pull_zone_id: i64,
    },
    /// Create a Shield Zone for a Pull Zone
    Create {
        #[arg(long)]
        pull_zone_id: i64,
    },
    /// Update a Shield Zone's configuration
    Update {
        #[arg(long)]
        shield_zone_id: i64,
        /// Enable or disable WAF
        #[arg(long)]
        waf_enabled: Option<bool>,
        /// WAF execution mode (0 = Disabled, 1 = Enabled)
        #[arg(long)]
        waf_execution_mode: Option<u8>,
        /// DDoS shield sensitivity (0 = Disabled, 1 = Low, 2 = Medium, 3 = High, 4 = VeryHigh)
        #[arg(long)]
        ddos_sensitivity: Option<u8>,
        /// DDoS execution mode (0 = Disabled, 1 = Enabled)
        #[arg(long)]
        ddos_execution_mode: Option<u8>,
        /// DDoS challenge window duration in seconds
        #[arg(long)]
        ddos_challenge_window: Option<i32>,
        /// Enable or disable learning mode
        #[arg(long)]
        learning_mode: Option<bool>,
    },
}

#[derive(Subcommand)]
pub enum ShieldWafAction {
    /// List available WAF profiles
    Profiles,
    /// List custom WAF rules for a Shield Zone
    ListRules {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get a custom WAF rule by ID
    GetRule {
        #[arg(long)]
        id: i64,
    },
    /// Add a custom WAF rule
    AddRule {
        #[arg(long)]
        shield_zone_id: i64,
        /// Rule name
        #[arg(long)]
        name: Option<String>,
        /// Action type (1 = Block, 2 = LogOnly, 3 = Challenge, 4 = ChallengeInterstitial, 5 = Allow)
        #[arg(long)]
        action_type: u8,
        /// Operator type (0 = Eq, 1 = NotEq, 2 = Contains, 3 = NotContains, 4 = Begins, 5 = Ends, 6 = Regex, 7 = NotRegex, 8 = Lt, 9 = Gt, 12 = Pm, 14 = PmFromFile, 15 = IpMatch, 17 = GeoLookup, 18 = ValidateUrlEncoding)
        #[arg(long)]
        operator_type: u8,
        /// Severity type (0 = Low, 1 = Medium, 2 = High)
        #[arg(long)]
        severity_type: u8,
        /// Value to match against
        #[arg(long)]
        value: Option<String>,
    },
    /// Update a custom WAF rule
    UpdateRule {
        #[arg(long)]
        id: i64,
        /// Rule name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a custom WAF rule
    DeleteRule {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ShieldRateLimitAction {
    /// List rate limit rules for a Shield Zone
    List {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get a rate limit rule by ID
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Create a rate limit rule
    Create {
        #[arg(long)]
        shield_zone_id: i64,
        /// Rule name
        #[arg(long)]
        name: Option<String>,
        /// Action type (1 = Block, 2 = LogOnly, 3 = Challenge)
        #[arg(long)]
        action_type: u8,
        /// Operator type (0–18; same values as WAF rules)
        #[arg(long)]
        operator_type: u8,
        /// Severity type (0 = Low, 1 = Medium, 2 = High)
        #[arg(long)]
        severity_type: u8,
        /// Value to match against
        #[arg(long)]
        value: Option<String>,
        /// Number of requests before triggering the rule
        #[arg(long)]
        request_count: i32,
        /// Counter key type (0 = Global, 1 = PerIp, 2 = PerCountry, 3 = PerAsn, 4 = PerHeader, 5 = PerCookie, 6 = PerQuery, 7 = PerFingerprint)
        #[arg(long)]
        counter_key_type: u8,
        /// Counting timeframe in seconds (1, 10, 60, 300, 900, 3600)
        #[arg(long)]
        timeframe: u16,
        /// Block duration in seconds (30, 60, 300, 900, 1800, 3600)
        #[arg(long)]
        block_time: u16,
    },
    /// Update a rate limit rule
    Update {
        #[arg(long)]
        id: i64,
        /// Rule name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a rate limit rule
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ShieldAccessListAction {
    /// Get all access lists (managed + custom) for a Shield Zone
    List {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Get a custom access list by ID
    Get {
        #[arg(long)]
        shield_zone_id: i64,
        #[arg(long)]
        id: i64,
    },
    /// Create a custom access list
    Create {
        #[arg(long)]
        shield_zone_id: i64,
        /// List name
        #[arg(long)]
        name: String,
        /// List type (0 = Ip, 1 = Country, 2 = Asn, 3 = Hostname, 4 = UserAgent, 5 = Custom)
        #[arg(long)]
        r#type: u8,
        /// List content (newline-separated entries)
        #[arg(long)]
        content: String,
    },
    /// Update a custom access list
    Update {
        #[arg(long)]
        shield_zone_id: i64,
        #[arg(long)]
        id: i64,
        /// List name
        #[arg(long)]
        name: Option<String>,
        /// List content (newline-separated entries)
        #[arg(long)]
        content: Option<String>,
    },
    /// Delete a custom access list
    Delete {
        #[arg(long)]
        shield_zone_id: i64,
        #[arg(long)]
        id: i64,
    },
    /// Update access list configuration (enabled/action)
    UpdateConfig {
        #[arg(long)]
        shield_zone_id: i64,
        /// Configuration ID
        #[arg(long)]
        configuration_id: i64,
        /// Enable or disable the access list
        #[arg(long)]
        is_enabled: Option<bool>,
        /// Action (0 = NoAction, 1 = Block, 2 = Allow, 3 = LogOnly, 4 = Challenge, 5 = ChallengeInterstitial)
        #[arg(long)]
        action: Option<u8>,
    },
}

#[derive(Subcommand)]
pub enum ShieldBotDetectionAction {
    /// Get bot detection configuration for a Shield Zone
    Get {
        #[arg(long)]
        shield_zone_id: i64,
    },
    /// Update bot detection configuration
    Update {
        #[arg(long)]
        shield_zone_id: i64,
        /// Execution mode (0 = Disabled, 1 = Enabled)
        #[arg(long)]
        execution_mode: Option<u8>,
        /// Request integrity sensitivity (0 = Disabled, 1 = Low, 2 = Medium, 3 = High)
        #[arg(long)]
        request_integrity_sensitivity: Option<u8>,
        /// IP address reputation sensitivity (0 = Disabled, 1 = Low, 2 = Medium, 3 = High)
        #[arg(long)]
        ip_address_sensitivity: Option<u8>,
        /// Browser fingerprint sensitivity (0 = Disabled, 1 = Low, 2 = Medium, 3 = High)
        #[arg(long)]
        fingerprint_sensitivity: Option<u8>,
        /// Browser fingerprint aggression (0 = Disabled, 1 = VeryLow, 2 = Low, 3 = Medium, 4 = High)
        #[arg(long)]
        fingerprint_aggression: Option<u8>,
        /// Enable complex browser fingerprinting
        #[arg(long)]
        fingerprint_complex_enabled: Option<bool>,
    },
}

// -- Edge Scripting --

#[derive(Subcommand)]
pub enum ScriptAction {
    /// List edge scripts
    List {
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<i32>,
        /// Items per page
        #[arg(long)]
        per_page: Option<i32>,
    },
    /// Get an edge script by ID
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Create a new edge script
    Create {
        #[arg(long)]
        name: String,
        /// Script type (0 = Dns, 1 = Cdn, 2 = Middleware)
        #[arg(long)]
        script_type: u8,
        /// Initial source code
        #[arg(long)]
        code: Option<String>,
        /// Create a linked pull zone automatically
        #[arg(long)]
        create_linked_pull_zone: bool,
        /// Name for the linked pull zone
        #[arg(long)]
        linked_pull_zone_name: Option<String>,
    },
    /// Update an edge script
    Update {
        #[arg(long)]
        id: i64,
        /// New name for the script
        #[arg(long)]
        name: Option<String>,
        /// Script type (0 = Dns, 1 = Cdn, 2 = Middleware)
        #[arg(long)]
        script_type: Option<u8>,
    },
    /// Delete an edge script
    Delete {
        #[arg(long)]
        id: i64,
        /// Also delete linked pull zones
        #[arg(long)]
        delete_linked_pull_zones: bool,
    },
    /// Manage script source code
    Code {
        #[command(subcommand)]
        action: ScriptCodeAction,
    },
    /// Publish a new release of the script
    Publish {
        #[arg(long)]
        id: i64,
        /// Release note
        #[arg(long)]
        note: Option<String>,
    },
    /// Manage script releases
    Release {
        #[command(subcommand)]
        action: ScriptReleaseAction,
    },
    /// Manage script environment variables
    Variable {
        #[command(subcommand)]
        action: ScriptVariableAction,
    },
    /// Manage script secrets
    Secret {
        #[command(subcommand)]
        action: ScriptSecretAction,
    },
    /// Get script usage statistics
    Statistics {
        #[arg(long)]
        id: i64,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        date_to: Option<String>,
        /// Return hourly breakdowns
        #[arg(long)]
        hourly: bool,
    },
    /// Rotate the deployment key for a script
    RotateDeploymentKey {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ScriptCodeAction {
    /// Get the current draft source code
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Update the draft source code
    Update {
        #[arg(long)]
        id: i64,
        /// Inline source code
        #[arg(long, conflicts_with = "file")]
        code: Option<String>,
        /// Read source code from a file
        #[arg(long, conflicts_with = "code")]
        file: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ScriptReleaseAction {
    /// List all releases for a script
    List {
        #[arg(long)]
        id: i64,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<i32>,
        /// Items per page
        #[arg(long)]
        per_page: Option<i32>,
    },
    /// Get the active (live) release for a script
    GetActive {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ScriptVariableAction {
    /// List environment variables for a script
    List {
        #[arg(long)]
        id: i64,
    },
    /// Add an environment variable to a script
    Add {
        #[arg(long)]
        id: i64,
        /// Variable name
        #[arg(long)]
        name: String,
        /// Mark this variable as required
        #[arg(long)]
        required: bool,
        /// Default value for the variable
        #[arg(long)]
        default_value: Option<String>,
    },
    /// Update an environment variable
    Update {
        #[arg(long)]
        id: i64,
        /// Variable ID
        #[arg(long)]
        variable_id: i64,
        /// Mark required (true/false)
        #[arg(long)]
        required: Option<bool>,
        /// New default value
        #[arg(long)]
        default_value: Option<String>,
    },
    /// Delete an environment variable
    Delete {
        #[arg(long)]
        id: i64,
        /// Variable ID
        #[arg(long)]
        variable_id: i64,
    },
    /// Upsert (create or update by name) an environment variable
    Upsert {
        #[arg(long)]
        id: i64,
        /// Variable name
        #[arg(long)]
        name: String,
        /// Mark this variable as required
        #[arg(long)]
        required: Option<bool>,
        /// Default value for the variable
        #[arg(long)]
        default_value: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ScriptSecretAction {
    /// List secrets for a script
    List {
        #[arg(long)]
        id: i64,
    },
    /// Add a secret to a script
    Add {
        #[arg(long)]
        id: i64,
        /// Secret name
        #[arg(long)]
        name: String,
        /// Secret value
        #[arg(long)]
        value: String,
    },
    /// Update a secret's value
    Update {
        #[arg(long)]
        id: i64,
        /// Secret ID
        #[arg(long)]
        secret_id: i64,
        /// New secret value
        #[arg(long)]
        value: String,
    },
    /// Delete a secret
    Delete {
        #[arg(long)]
        id: i64,
        /// Secret ID
        #[arg(long)]
        secret_id: i64,
    },
    /// Upsert (create or update by name) a secret
    Upsert {
        #[arg(long)]
        id: i64,
        /// Secret name
        #[arg(long)]
        name: String,
        /// Secret value
        #[arg(long)]
        value: String,
    },
}

// -- Containers --

#[derive(Subcommand)]
pub enum ContainerAction {
    /// Manage applications
    App {
        #[command(subcommand)]
        action: ContainerAppAction,
    },
    /// Manage container templates within an application
    Template {
        #[command(subcommand)]
        action: ContainerTemplateAction,
    },
    /// Manage application endpoints
    Endpoint {
        #[command(subcommand)]
        action: ContainerEndpointAction,
    },
    /// Manage application volumes
    Volume {
        #[command(subcommand)]
        action: ContainerVolumeAction,
    },
    /// Manage container registries
    Registry {
        #[command(subcommand)]
        action: ContainerRegistryAction,
    },
    /// Manage and query regions
    Region {
        #[command(subcommand)]
        action: ContainerRegionAction,
    },
    /// List available nodes
    Node {
        #[command(subcommand)]
        action: ContainerNodeAction,
    },
    /// Manage pods
    Pod {
        #[command(subcommand)]
        action: ContainerPodAction,
    },
    /// Show account limits for Magic Containers
    Limits,
    /// Manage log forwarding configurations
    LogForwarding {
        #[command(subcommand)]
        action: ContainerLogForwardingAction,
    },

    /// Shortcut for `container app list` — mirrors `pull-zone list` etc.
    /// `app` is the canonical subcommand; this alias is provided for symmetry.
    List {
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        limit: Option<i32>,
    },
    /// Shortcut for `container app get`. `app` is the canonical subcommand.
    Get {
        #[arg(long)]
        id: String,
    },
    /// Shortcut for `container app delete`. `app` is the canonical subcommand.
    Delete {
        #[arg(long)]
        id: String,
        #[arg(long, conflicts_with = "no_cascade")]
        cascade: bool,
        #[arg(long)]
        no_cascade: bool,
    },
}

#[derive(Subcommand)]
pub enum ContainerAppAction {
    /// List all applications
    List {
        /// Cursor for the next page
        #[arg(long)]
        cursor: Option<String>,
        /// Maximum number of results
        #[arg(long)]
        limit: Option<i32>,
    },
    /// Get a specific application
    Get {
        #[arg(long)]
        id: String,
    },
    /// Create a new application
    Create {
        /// Application name
        #[arg(long)]
        name: String,
        /// Runtime type (Shared or Reserved)
        #[arg(long)]
        runtime_type: String,
        /// Minimum number of instances
        #[arg(long)]
        min: i32,
        /// Maximum number of instances
        #[arg(long)]
        max: i32,
        /// Region IDs (may be repeated)
        #[arg(long = "region")]
        regions: Vec<String>,
        /// Container image name (e.g. "nginx")
        #[arg(long)]
        image_name: Option<String>,
        /// Container image namespace (e.g. "library")
        #[arg(long)]
        image_namespace: Option<String>,
        /// Container image tag (e.g. "alpine")
        #[arg(long)]
        image_tag: Option<String>,
        /// Container image registry ID (e.g. "1155" for DockerHub)
        #[arg(long)]
        registry_id: Option<String>,
        /// Initial environment variable for the embedded container template
        /// (KEY=VALUE). Repeatable. Requires the image flags above; applied
        /// via a follow-up `template env --replace-all` after creation.
        #[arg(long = "env")]
        env: Vec<String>,
        /// Return only `{"id": "..."}` (legacy behaviour). By default the
        /// full application document is returned for `--format json`.
        #[arg(long)]
        minimal: bool,
    },
    /// Update an application
    Update {
        #[arg(long)]
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New runtime type
        #[arg(long)]
        runtime_type: Option<String>,
        /// New minimum instances
        #[arg(long)]
        min: Option<i32>,
        /// New maximum instances
        #[arg(long)]
        max: Option<i32>,
    },
    /// Deploy an application
    Deploy {
        #[arg(long)]
        id: String,
    },
    /// Undeploy (suspend) an application
    Undeploy {
        #[arg(long)]
        id: String,
    },
    /// Restart all pods in an application
    Restart {
        #[arg(long)]
        id: String,
    },
    /// Delete an application.
    ///
    /// Refuses by default if the app has auto-managed Pull Zones (created
    /// for CDN endpoints) — pass `--cascade` to also delete them, or
    /// `--no-cascade` to delete only the app and print the orphan IDs.
    Delete {
        #[arg(long)]
        id: String,
        /// Also delete every auto-managed Pull Zone owned by this app.
        #[arg(long, conflicts_with = "no_cascade")]
        cascade: bool,
        /// Delete only the app; print the orphan Pull Zone IDs for manual
        /// cleanup. Mutually exclusive with `--cascade`.
        #[arg(long)]
        no_cascade: bool,
    },
    /// Show live overview for an application
    Overview {
        #[arg(long)]
        id: String,
    },
    /// Show statistics for an application
    Statistics {
        #[arg(long)]
        id: String,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: String,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
        /// Statistics granularity (Daily, Hourly, Minute)
        #[arg(long)]
        granularity: Option<String>,
    },
    /// Get autoscaling settings for an application
    AutoscalingGet {
        #[arg(long)]
        app_id: String,
    },
    /// Update autoscaling settings for an application
    AutoscalingUpdate {
        #[arg(long)]
        app_id: String,
        /// Minimum number of instances
        #[arg(long)]
        min: i32,
        /// Maximum number of instances
        #[arg(long)]
        max: i32,
    },
    /// Get region settings for an application
    RegionSettingsGet {
        #[arg(long)]
        app_id: String,
    },
    /// Update region settings for an application
    RegionSettingsUpdate {
        #[arg(long)]
        app_id: String,
        /// Allowed region IDs (may be repeated)
        #[arg(long = "allowed-region")]
        allowed_region_ids: Option<Vec<String>>,
        /// Required region IDs (may be repeated)
        #[arg(long = "required-region")]
        required_region_ids: Option<Vec<String>>,
        /// Maximum number of allowed regions
        #[arg(long)]
        max_allowed_regions: Option<i32>,
    },
}

#[derive(Subcommand)]
pub enum ContainerTemplateAction {
    /// Get a container template
    Get {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        container_id: String,
    },
    /// Add a container template to an application
    Add {
        #[arg(long)]
        app_id: String,
        /// Container name
        #[arg(long)]
        name: String,
        /// Docker image name
        #[arg(long)]
        image_name: String,
        /// Docker image namespace
        #[arg(long)]
        image_namespace: String,
        /// Docker image tag
        #[arg(long)]
        image_tag: String,
        /// Registry ID
        #[arg(long)]
        registry_id: String,
    },
    /// Update a container template
    Update {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        container_id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New image tag
        #[arg(long)]
        image_tag: Option<String>,
        /// New image name
        #[arg(long)]
        image_name: Option<String>,
        /// New image namespace
        #[arg(long)]
        image_namespace: Option<String>,
        /// New registry ID
        #[arg(long)]
        registry_id: Option<String>,
    },
    /// Delete a container template
    Delete {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        container_id: String,
    },
    /// Manage environment variables for a container template.
    ///
    /// By default this command edits the env-var set granularly via
    /// `--add` / `--remove` / `--update`. To replace the whole set, pass
    /// `--replace-all` with one or more `--env KEY=VAL`. To wipe every
    /// var, pass `--clear`. A bare invocation with no flags is rejected
    /// to prevent accidental wipes.
    #[command(long_about = "\
Manage environment variables for a container template.

Granular operations (default — preserve existing vars):
  hoppy container template env --app-id <a> --container-id <c> --add KEY=VAL
  hoppy container template env --app-id <a> --container-id <c> --remove KEY
  hoppy container template env --app-id <a> --container-id <c> --update KEY=VAL

Show current values (redacted by default — use --reveal to see them):
  hoppy container template env --app-id <a> --container-id <c> --list

Destructive operations (require explicit intent):
  --replace-all  with one or more --env KEY=VAL  → replace the whole set
  --clear                                        → wipe every var
")]
    Env {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        container_id: String,
        /// Add or update an env var (KEY=VALUE). Repeatable. Idempotent —
        /// existing vars not named here are preserved.
        #[arg(long = "add")]
        add: Vec<String>,
        /// Alias of --add (KEY=VALUE). Repeatable.
        #[arg(long = "update")]
        update: Vec<String>,
        /// Remove an env var by name. Repeatable. Missing names are ignored.
        #[arg(long = "remove")]
        remove: Vec<String>,
        /// Replace the entire env-var set. Combine with one or more `--env
        /// KEY=VALUE`. Destructive — drops any var not named here.
        #[arg(long)]
        replace_all: bool,
        /// Wipe every env var. Cannot be combined with --add / --remove /
        /// --update / --replace-all / --env / --list.
        #[arg(long)]
        clear: bool,
        /// List the current env vars (names only by default; values follow
        /// the redaction layer). Cannot be combined with mutating flags.
        #[arg(long)]
        list: bool,
        /// KEY=VALUE pairs (only used with --replace-all).
        #[arg(long = "env")]
        env: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ContainerEndpointAction {
    /// List endpoints for an application
    List {
        #[arg(long)]
        app_id: String,
    },
    /// Add an endpoint to an application
    Add {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        container_id: String,
        /// Display name for the endpoint
        #[arg(long)]
        name: String,
        /// Container port to expose
        #[arg(long)]
        container_port: i32,
        /// Publicly exposed port
        #[arg(long)]
        exposed_port: Option<i32>,
        /// Use CDN endpoint type
        #[arg(long, conflicts_with = "anycast")]
        cdn: bool,
        /// Use Anycast endpoint type
        #[arg(long, conflicts_with = "cdn")]
        anycast: bool,
    },
    /// Update an endpoint
    Update {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        endpoint_id: String,
        /// Display name for the endpoint
        #[arg(long)]
        name: String,
        /// Container port
        #[arg(long)]
        container_port: i32,
        /// Publicly exposed port
        #[arg(long)]
        exposed_port: Option<i32>,
        /// Use CDN endpoint type
        #[arg(long, conflicts_with = "anycast")]
        cdn: bool,
        /// Use Anycast endpoint type
        #[arg(long, conflicts_with = "cdn")]
        anycast: bool,
    },
    /// Delete an endpoint
    Delete {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        endpoint_id: String,
    },
}

#[derive(Subcommand)]
pub enum ContainerVolumeAction {
    /// List volumes for an application
    List {
        #[arg(long)]
        app_id: String,
    },
    /// Update a volume
    Update {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        volume_id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New size in GB
        #[arg(long)]
        size: Option<i32>,
    },
    /// Detach a volume from all pods
    Detach {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        volume_id: String,
    },
    /// Delete all instances of a volume
    Delete {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        volume_id: String,
    },
    /// Delete a single volume instance
    DeleteInstance {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        volume_id: String,
        #[arg(long)]
        instance_id: String,
    },
}

#[derive(Subcommand)]
pub enum ContainerRegistryAction {
    /// List all container registries
    List,
    /// Get a specific container registry
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Create a container registry
    Create {
        /// Display name
        #[arg(long)]
        name: String,
        /// Registry type (DockerHub or GitHub)
        #[arg(long)]
        registry_type: Option<String>,
        /// Username for authentication
        #[arg(long)]
        username: Option<String>,
        /// Password for authentication
        #[arg(long)]
        password: Option<String>,
    },
    /// Update a container registry
    Update {
        #[arg(long)]
        id: i64,
        /// New display name
        #[arg(long)]
        name: String,
        /// New username
        #[arg(long)]
        username: Option<String>,
        /// New password
        #[arg(long)]
        password: Option<String>,
    },
    /// Delete a container registry
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// List tags for a container image
    ImageTags {
        /// Registry ID
        #[arg(long)]
        registry_id: String,
        /// Image name
        #[arg(long)]
        image_name: String,
        /// Image namespace (e.g. "library" for Docker Hub official images)
        #[arg(long)]
        image_namespace: String,
    },
    /// Get the digest for a container image tag
    ImageDigest {
        /// Registry ID
        #[arg(long)]
        registry_id: String,
        /// Image name
        #[arg(long)]
        image_name: String,
        /// Image namespace
        #[arg(long)]
        image_namespace: String,
        /// Image tag
        #[arg(long)]
        tag: String,
    },
    /// Get configuration suggestions for a container image
    ConfigSuggestions {
        /// Registry ID
        #[arg(long)]
        registry_id: String,
        /// Image name
        #[arg(long)]
        image_name: String,
        /// Image namespace
        #[arg(long)]
        image_namespace: String,
        /// Image tag
        #[arg(long)]
        tag: String,
    },
    /// Search public container images
    SearchPublic {
        /// Registry ID
        #[arg(long)]
        registry_id: String,
        /// Search prefix/query
        #[arg(long)]
        query: String,
        /// Maximum results
        #[arg(long)]
        size: Option<i32>,
        /// Page number
        #[arg(long)]
        page: Option<i32>,
    },
}

#[derive(Subcommand)]
pub enum ContainerRegionAction {
    /// List available regions
    List {
        /// Cursor for the next page
        #[arg(long)]
        cursor: Option<String>,
        /// Maximum number of results
        #[arg(long)]
        limit: Option<i32>,
    },
    /// Get the optimal base region
    Optimal,
}

#[derive(Subcommand)]
pub enum ContainerNodeAction {
    /// List available nodes
    List {
        /// Cursor for the next page
        #[arg(long)]
        cursor: Option<String>,
        /// Maximum number of results
        #[arg(long)]
        limit: Option<i32>,
    },
}

#[derive(Subcommand)]
pub enum ContainerPodAction {
    /// Recreate a pod
    Recreate {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        pod_id: String,
    },
}

#[derive(Subcommand)]
pub enum ContainerLogForwardingAction {
    /// List all log forwarding configurations
    List,
    /// Get log forwarding configuration for an application
    Get {
        #[arg(long)]
        app_id: String,
    },
    /// Create a log forwarding configuration
    Create {
        #[arg(long)]
        app_id: String,
        /// Transport type (SyslogUdp or SyslogTcp)
        #[arg(long)]
        forwarding_type: String,
        /// Syslog endpoint host
        #[arg(long)]
        endpoint: String,
        /// Syslog endpoint port
        #[arg(long)]
        port: i32,
        /// Syslog format (SyslogRfc3164 or SyslogRfc5424)
        #[arg(long = "syslog-format")]
        syslog_format: String,
        /// Optional authentication token
        #[arg(long)]
        token: Option<String>,
        /// Enable immediately
        #[arg(long)]
        enabled: bool,
    },
    /// Update a log forwarding configuration
    Update {
        #[arg(long)]
        app_id: String,
        /// Transport type (SyslogUdp or SyslogTcp)
        #[arg(long)]
        forwarding_type: String,
        /// Syslog endpoint host
        #[arg(long)]
        endpoint: String,
        /// Syslog endpoint port
        #[arg(long)]
        port: i32,
        /// Syslog format (SyslogRfc3164 or SyslogRfc5424)
        #[arg(long = "syslog-format")]
        syslog_format: String,
        /// Optional authentication token
        #[arg(long)]
        token: Option<String>,
        /// Enable the configuration
        #[arg(long)]
        enabled: bool,
    },
    /// Delete a log forwarding configuration
    Delete {
        #[arg(long)]
        app_id: String,
    },
}

// -- Database (libSQL) --

#[derive(Subcommand)]
pub enum DbAction {
    /// List all databases (v1)
    List {
        /// Filter by group ID
        #[arg(long)]
        group_id: Option<String>,
    },
    /// Get a database (v1)
    Get {
        /// Database ID (db_<ulid>)
        #[arg(long)]
        id: String,
    },
    /// Create a database (v1).
    ///
    /// EXAMPLES:
    ///   hoppy db create --slug my-app --group group_01HX...
    ///
    /// Slug must be lowercase, start with a letter, max 24 chars
    /// (`^[a-z][a-z0-9-]{0,23}$`). Long slugs return "Internal error"
    /// upstream — hoppy validates locally before hitting the API.
    Create {
        /// Database slug (lowercase, max 24 chars, see --help)
        #[arg(
            long,
            long_help = "Database slug — must match `^[a-z][a-z0-9-]{0,23}$`. \
The bunny API silently fails on long slugs ('Internal error' 500). \
Hoppy validates locally before the API call."
        )]
        slug: String,
        /// Database group ID (group_<ulid>) to create the DB in
        #[arg(long)]
        group: String,
    },
    /// Delete a database (v1)
    Delete {
        /// Database ID
        #[arg(long)]
        id: String,
    },
    /// Fork a database into a new slug (preview)
    Fork {
        /// Source database ID
        #[arg(long)]
        id: String,
        /// New slug for the fork
        #[arg(long)]
        target: String,
        /// Destination group (defaults to the source's group)
        #[arg(long)]
        group: Option<String>,
    },
    /// Destructive: restore a database to a previous generation (preview)
    Restore {
        /// Database ID
        #[arg(long)]
        id: String,
        /// Generation UUID to restore (see `db versions`)
        #[arg(long)]
        version: String,
    },
    /// List database generation versions (preview)
    Versions {
        /// Database ID
        #[arg(long)]
        id: String,
        /// Limit the number of generations returned
        #[arg(long)]
        limit: Option<u64>,
    },
    /// Ping a database with `SELECT 1` via its libSQL data plane.
    ///
    /// EXAMPLES:
    ///   # Mints a short-lived read-only token automatically
    ///   hoppy db ping --id db_01HX...
    ///
    ///   # Provide a JWT instead of minting one
    ///   hoppy db ping --id db_01HX... --token-file ~/.bunny/db.jwt
    Ping {
        /// Database ID
        #[arg(long)]
        id: String,
        /// File containing a libSQL JWT (skips the implicit token mint)
        #[arg(long, value_name = "PATH")]
        token_file: Option<String>,
    },
    /// Get statistics for a database (v2)
    Statistics {
        #[arg(long)]
        id: String,
        /// Start of the time window (RFC 3339)
        #[arg(long)]
        from: String,
        /// End of the time window (RFC 3339)
        #[arg(long)]
        to: String,
    },
    /// Get aggregated usage for a database (v2)
    Usage {
        #[arg(long)]
        id: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    /// Get account-level active database usage (v2)
    ActiveUsage,
    /// Get live metrics for one or more databases
    Live {
        /// Database IDs (repeatable)
        #[arg(long = "id", value_name = "DB_ID")]
        ids: Vec<String>,
    },
    /// v2 endpoints (gated; some are broken upstream)
    V2 {
        #[command(subcommand)]
        action: DbV2Action,
    },
    /// Manage database groups
    Group {
        #[command(subcommand)]
        action: DbGroupAction,
    },
    /// Manage database auth tokens
    Token {
        #[command(subcommand)]
        action: DbTokenAction,
    },
    /// Manage / inspect bunny database config (regions, limits, optimal)
    Config {
        #[command(subcommand)]
        action: DbConfigAction,
    },
}

#[derive(Subcommand)]
pub enum DbV2Action {
    /// List databases (v2)
    List {
        #[arg(long, default_value_t = 1)]
        page: u32,
        #[arg(long)]
        per_page: Option<u32>,
        #[arg(long)]
        search: Option<String>,
    },
    /// Get a database (v2)
    Get {
        #[arg(long)]
        id: String,
    },
    /// Create a database (v2). NOTE: returns 500 upstream as of 2026-05-05.
    ///
    /// long_help: storage-region uses flat regions (eu-west-1, us-east-1).
    /// primary-region/replicas-region use compute codes (DE, FR, AMS, …).
    Create {
        #[arg(long)]
        name: String,
        /// Storage region, e.g. `eu-west-1` (see `db config show`)
        #[arg(long)]
        storage_region: String,
        /// Primary region (compute code, repeatable). Examples: DE, FR, AMS.
        #[arg(long = "primary-region", value_name = "REGION")]
        primary_regions: Vec<String>,
        /// Replica region (compute code, repeatable). Examples: UK, NY.
        #[arg(long = "replicas-region", value_name = "REGION")]
        replicas_regions: Vec<String>,
    },
    /// Delete a database (v2)
    Delete {
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum DbGroupAction {
    /// List database groups
    List {
        #[arg(long)]
        search: Option<String>,
    },
    /// Get a database group
    Get {
        #[arg(long)]
        id: String,
    },
    /// Create a database group.
    ///
    /// long_help: storage-region uses flat regions (eu-west-1, us-east-1).
    /// primary-region/replicas-region use compute codes (DE, FR, AMS, UK, …).
    Create {
        /// Display name (max 64 chars)
        #[arg(long)]
        display_name: String,
        /// Storage region (e.g. eu-west-1; see `db config show`)
        #[arg(long)]
        storage_region: String,
        /// Primary region (compute code, repeatable). Examples: DE, FR, AMS.
        #[arg(long = "primary-region", value_name = "REGION")]
        primary_regions: Vec<String>,
        /// Replica region (compute code, repeatable). Examples: UK, NY.
        #[arg(long = "replicas-region", value_name = "REGION")]
        replicas_regions: Vec<String>,
    },
    /// Delete a database group
    Delete {
        #[arg(long)]
        id: String,
    },
    /// Get statistics for a database group
    Stats {
        #[arg(long)]
        id: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    /// Get aggregated usage for a database group
    Usage {
        #[arg(long)]
        id: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    /// Get live metrics for one or more groups
    Live {
        #[arg(long = "id", value_name = "GROUP_ID")]
        ids: Vec<String>,
    },
    /// Generate a new auth token for a whole group
    GenerateKeys {
        #[arg(long)]
        id: String,
        /// Token scope: full-access (default) or read-only
        #[arg(long, value_enum, default_value_t = TokenAuthorization::FullAccess)]
        authorization: TokenAuthorization,
        /// Optional expiry timestamp (RFC 3339)
        #[arg(long)]
        expires_at: Option<String>,
    },
    /// Invalidate every auth token for a group
    InvalidateKeys {
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum DbTokenAction {
    /// Mint a JWT for a database (v1).
    ///
    /// EXAMPLES:
    ///   hoppy db token mint --db-id db_01HX... --authorization full-access
    ///   hoppy db token mint --db-id db_01HX... --authorization read-only --expires-at 2026-12-31T23:59:59Z
    ///
    /// By default the JWT is redacted in the output (length + scope only).
    /// Pass `--reveal` (the global flag) to print the raw token.
    Mint {
        #[arg(long)]
        db_id: String,
        /// Token scope: full-access or read-only
        #[arg(long, value_enum, default_value_t = TokenAuthorization::FullAccess)]
        authorization: TokenAuthorization,
        /// Optional expiry timestamp (RFC 3339)
        #[arg(long)]
        expires_at: Option<String>,
    },
    /// Invalidate every auth token for a database (v1)
    Invalidate {
        #[arg(long)]
        db_id: String,
    },
    /// Mint a JWT for a database (v2)
    GenerateV2 {
        #[arg(long)]
        db_id: String,
        #[arg(long, value_enum, default_value_t = TokenAuthorization::FullAccess)]
        authorization: TokenAuthorization,
        #[arg(long)]
        expires_at: Option<String>,
    },
    /// Revoke every auth token for a database (v2)
    RevokeV2 {
        #[arg(long)]
        db_id: String,
    },
}

#[derive(Subcommand)]
pub enum DbConfigAction {
    /// Show available storage and compute regions
    Show,
    /// Show account database limits
    Limits,
    /// Get the optimal multi-region recommendation for the user
    Optimal,
    /// Get the optimal single-region recommendation
    OptimalSingle,
}

/// Token scope for `db token mint` and `db group generate-keys`.
#[derive(Copy, Clone, ValueEnum)]
pub enum TokenAuthorization {
    /// Full read+write access (`full-access` over the wire).
    FullAccess,
    /// Read-only access (`read-only` over the wire).
    ReadOnly,
}

impl From<TokenAuthorization> for bunny_api_database::types::Authorization {
    fn from(t: TokenAuthorization) -> Self {
        match t {
            TokenAuthorization::FullAccess => Self::FullAccess,
            TokenAuthorization::ReadOnly => Self::ReadOnly,
        }
    }
}
