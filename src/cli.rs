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
        /// Record type (A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA, Redirect, Flatten)
        #[arg(long, value_name = "TYPE")]
        r#type: String,
        /// Record name (subdomain, empty for apex)
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
        /// Record type (A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA)
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
    List,
    /// Get a specific video library
    Get {
        #[arg(long)]
        id: u64,
    },
    /// Create a video library
    Create {
        #[arg(long)]
        name: String,
    },
    /// Update a video library
    Update {
        #[arg(long)]
        id: u64,
    },
    /// Delete a video library
    Delete {
        #[arg(long)]
        id: u64,
    },
}

#[derive(Subcommand)]
pub enum StreamVideoAction {
    /// List videos in a library
    List {
        #[arg(long)]
        library_id: u64,
    },
    /// Get a specific video
    Get {
        #[arg(long)]
        library_id: u64,
        #[arg(long)]
        video_id: String,
    },
    /// Upload a video
    Upload {
        #[arg(long)]
        library_id: u64,
        #[arg(long)]
        file: String,
    },
    /// Delete a video
    Delete {
        #[arg(long)]
        library_id: u64,
        #[arg(long)]
        video_id: String,
    },
}

// -- Shield --

#[derive(Subcommand)]
pub enum ShieldAction {
    /// WAF rules
    Waf,
    /// Rate limiting
    RateLimit,
    /// DDoS protection
    Ddos,
}

// -- Edge Scripting --

#[derive(Subcommand)]
pub enum ScriptAction {
    /// List edge scripts
    List,
    /// Get an edge script
    Get {
        #[arg(long)]
        id: u64,
    },
    /// Create an edge script
    Create {
        #[arg(long)]
        name: String,
    },
    /// Delete an edge script
    Delete {
        #[arg(long)]
        id: u64,
    },
    /// Deploy an edge script
    Deploy {
        #[arg(long)]
        id: u64,
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
