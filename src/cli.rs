use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "hoppy",
    version,
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

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Text,
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

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
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
    /// Create a new pull zone
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        origin_url: String,
    },
    /// Update a pull zone
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        origin_url: Option<String>,
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
    /// Create a new storage zone
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
}

#[derive(Subcommand)]
pub enum DnsRecordAction {
    /// List records in a DNS zone
    List {
        #[arg(long)]
        zone_id: i64,
    },
    /// Add a DNS record
    Add {
        #[arg(long)]
        zone_id: i64,
        /// Record type (A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA, Redirect, Flatten, PullZone, Script; case-insensitive)
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
    /// Delete a video
    Delete {
        #[arg(long)]
        library_id: i64,
        #[arg(long)]
        video_id: String,
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
}

// -- Containers --

#[derive(Subcommand)]
pub enum ContainerAction {
    /// List containers
    List,
    /// Get a specific container
    Get {
        #[arg(long)]
        id: String,
    },
    /// Create a container
    Create {
        #[arg(long)]
        name: String,
    },
    /// Delete a container
    Delete {
        #[arg(long)]
        id: String,
    },
    /// View container logs
    Logs {
        #[arg(long)]
        id: String,
    },
}
