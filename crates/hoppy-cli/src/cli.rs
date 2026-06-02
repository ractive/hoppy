use bunny_net_api::core::types::{
    LogAnonymizationType, OptimizerWatermarkPosition, PermaCacheType,
    PullZoneLogForwarderProtocolType, PullZoneTierType, ShieldDDosProtectionType,
    StickySessionType,
};
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Build the `-V` / `--version` string from compile-time provenance. clap
/// prepends the binary name when printing, so the user sees e.g.
/// `hoppy 0.3.0 (abc123def456 2026-05-26)`. With SHA + date populated this
/// returns `"0.3.0 (abc123def456 2026-05-26)"`; with both empty
/// (e.g. `CARGO_HOPPY_FORCE_NO_GIT=1`) it returns bare `CARGO_PKG_VERSION`.
pub fn build_version_string() -> String {
    format_version(
        env!("CARGO_PKG_VERSION"),
        env!("HOPPY_BUILD_VERSION_SHA"),
        env!("HOPPY_BUILD_DATE"),
    )
}

/// Parse the CLI, rewriting clap's confusing "unexpected argument '-1'" error
/// (and similar negative-number trip-wires) into a human-readable hint.
///
/// clap parses tokens like `-1`, `-2.5`, or `-12h` as short flags rather than
/// values, so `hoppy container app create --min -1 ...` fails with
/// `error: unexpected argument '-1' found` even though the user obviously
/// meant `--min` to receive `-1`. This wrapper detects that case and emits a
/// message that points at the preceding `--<flag>` and the `--flag=value`
/// workaround.
pub fn parse_or_exit() -> Cli {
    match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => exit_with_friendly_error(err, std::env::args().collect()),
    }
}

fn exit_with_friendly_error(err: clap::Error, argv: Vec<String>) -> ! {
    if let Some(rendered) = rewrite_negative_value_error(&err, &argv) {
        let mut cmd = Cli::command();
        let new_err = cmd.error(ErrorKind::InvalidValue, rendered);
        new_err.exit()
    } else {
        err.exit()
    }
}

/// Return a friendlier error message when clap rejected a negative-looking
/// value (`-1`, `-2.5`, ...) that almost certainly belongs to the preceding
/// `--<flag>`. Returns `None` if the error is something else.
fn rewrite_negative_value_error(err: &clap::Error, argv: &[String]) -> Option<String> {
    if err.kind() != ErrorKind::UnknownArgument {
        return None;
    }
    let invalid = match err.get(ContextKind::InvalidArg)? {
        ContextValue::String(s) => s.clone(),
        ContextValue::Strings(v) => v.first()?.clone(),
        _ => return None,
    };
    if !looks_like_negative_number(&invalid) {
        return None;
    }
    let preceding_flag = preceding_long_flag(argv, &invalid);
    Some(format_negative_value_hint(
        &invalid,
        preceding_flag.as_deref(),
    ))
}

fn looks_like_negative_number(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'-' {
        return false;
    }
    // Accept `-1`, `-1.5`, `-.5`, `-1e3` etc.; require at least one digit.
    let rest = &s[1..];
    let has_digit = rest.chars().any(|c| c.is_ascii_digit());
    let only_numeric = rest
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-');
    has_digit && only_numeric
}

/// Walk argv and find the long flag (e.g. `--min`) that immediately precedes
/// the offending negative-looking token. Returns the flag name without the
/// leading `--`, or `None` if no such flag is found.
fn preceding_long_flag(argv: &[String], invalid: &str) -> Option<String> {
    let idx = argv.iter().position(|a| a == invalid)?;
    if idx == 0 {
        return None;
    }
    let prev = &argv[idx - 1];
    let stripped = prev.strip_prefix("--")?;
    if stripped.is_empty() || stripped.contains('=') {
        return None;
    }
    Some(stripped.to_string())
}

fn format_negative_value_hint(invalid: &str, preceding: Option<&str>) -> String {
    match preceding {
        Some(flag) => format!(
            "the value '{invalid}' looks like a negative number, but clap parsed it as a flag.\n\n\
             '--{flag}' expected a value. To pass a negative value, use the '=' form:\n    \
             --{flag}={invalid}\n\n\
             If '--{flag}' is a count or ID, negative values are not accepted — pass a non-negative number instead."
        ),
        None => format!(
            "the value '{invalid}' looks like a negative number, but clap parsed it as a flag.\n\n\
             To pass a negative value to a long flag, use the '=' form: --<flag>={invalid}"
        ),
    }
}

fn format_version(pkg: &str, sha: &str, date: &str) -> String {
    if sha.is_empty() {
        pkg.to_owned()
    } else if date.is_empty() {
        format!("{pkg} ({sha})")
    } else {
        format!("{pkg} ({sha} {date})")
    }
}

fn version_str() -> &'static str {
    use std::sync::OnceLock;
    static V: OnceLock<String> = OnceLock::new();
    V.get_or_init(build_version_string).as_str()
}

#[derive(Parser)]
#[command(
    name = "hoppy",
    version = version_str(),
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

    /// Suppress non-essential output. On predicate commands
    /// (`auth check`, `db ping`) the stdout payload is suppressed on
    /// success — the exit code carries the signal. On data commands the
    /// primary payload still prints; only hints/progress bars are hidden.
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

    /// Suppress drill-down hints (next-step suggestions) on stderr.
    /// `--format json` implies this so machine output stays clean.
    #[arg(long, global = true)]
    pub no_hints: bool,

    /// Record API responses as JSON fixtures under `<DIR>/<domain>/` (one
    /// subdirectory per service: core, compute, containers, database,
    /// shield, storage, stream). Writes are idempotent — unchanged files
    /// are left alone. May also be set via `HOPPY_RECORD_DIR=<DIR>`.
    #[arg(long, value_name = "DIR", global = true)]
    pub record: Option<String>,

    /// Disable PII redaction in `--record` fixtures. Off by default.
    /// The raw output may contain billing balances, emails, payment IDs,
    /// and signed URLs — do not commit it.
    /// May also be set via `HOPPY_NO_REDACT=1`.
    #[arg(long, global = true, env = "HOPPY_NO_REDACT")]
    pub no_redact: bool,

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

impl From<ZoneTier> for bunny_net_api::core::types::PullZoneType {
    fn from(t: ZoneTier) -> Self {
        match t {
            ZoneTier::Premium => Self::Premium,
            ZoneTier::Volume => Self::Volume,
        }
    }
}

/// Watermark position for the Optimizer (kebab-case names for the CLI).
#[derive(Copy, Clone, ValueEnum)]
pub enum OptimizerWatermarkPositionArg {
    /// Top-left corner (value 0)
    TopLeft,
    /// Top-right corner (value 1)
    TopRight,
    /// Bottom-left corner (value 2)
    BottomLeft,
    /// Bottom-right corner (value 3)
    BottomRight,
    /// Centered (value 4)
    Center,
}

impl From<OptimizerWatermarkPositionArg> for OptimizerWatermarkPosition {
    fn from(a: OptimizerWatermarkPositionArg) -> Self {
        match a {
            OptimizerWatermarkPositionArg::TopLeft => Self::TopLeft,
            OptimizerWatermarkPositionArg::TopRight => Self::TopRight,
            OptimizerWatermarkPositionArg::BottomLeft => Self::BottomLeft,
            OptimizerWatermarkPositionArg::BottomRight => Self::BottomRight,
            OptimizerWatermarkPositionArg::Center => Self::Center,
        }
    }
}

/// Protocol for CDN log forwarding to a remote syslog endpoint.
#[derive(Copy, Clone, ValueEnum)]
pub enum PullZoneLogForwardingProtocolArg {
    /// UDP transport (value 0).
    Udp,
    /// TCP transport (value 1).
    Tcp,
    /// TLS-encrypted TCP (value 2).
    TcpEncrypted,
    /// Datadog HTTP ingestion (value 3).
    Datadog,
}

impl From<PullZoneLogForwardingProtocolArg> for PullZoneLogForwarderProtocolType {
    fn from(a: PullZoneLogForwardingProtocolArg) -> Self {
        match a {
            PullZoneLogForwardingProtocolArg::Udp => Self::Udp,
            PullZoneLogForwardingProtocolArg::Tcp => Self::Tcp,
            PullZoneLogForwardingProtocolArg::TcpEncrypted => Self::TcpEncrypted,
            PullZoneLogForwardingProtocolArg::Datadog => Self::DataDog,
        }
    }
}

/// How the last octet of a logged IP address is anonymised.
#[derive(Copy, Clone, ValueEnum)]
pub enum LogAnonymizationTypeArg {
    /// Replace the last octet with a single digit (value 0).
    OneDigit,
    /// Drop the last octet entirely (value 1).
    Drop,
}

impl From<LogAnonymizationTypeArg> for LogAnonymizationType {
    fn from(a: LogAnonymizationTypeArg) -> Self {
        match a {
            LogAnonymizationTypeArg::OneDigit => Self::OneDigit,
            LogAnonymizationTypeArg::Drop => Self::Drop,
        }
    }
}

/// Perma-Cache mode for a pull zone.
#[derive(Copy, Clone, ValueEnum)]
pub enum PermaCacheTypeArg {
    /// Automatically retain content in the linked storage zone (value 0).
    Automatic,
    /// Only retain content explicitly pushed to the storage zone (value 1).
    Manual,
}

impl From<PermaCacheTypeArg> for PermaCacheType {
    fn from(a: PermaCacheTypeArg) -> Self {
        match a {
            PermaCacheTypeArg::Automatic => Self::Automatic,
            PermaCacheTypeArg::Manual => Self::Manual,
        }
    }
}

/// Sticky session affinity mode for a pull zone.
#[derive(Copy, Clone, ValueEnum)]
pub enum StickySessionTypeArg {
    /// No sticky sessions (value 0).
    None,
    /// Sticky sessions via a cookie (value 1).
    Cookie,
}

impl From<StickySessionTypeArg> for StickySessionType {
    fn from(a: StickySessionTypeArg) -> Self {
        match a {
            StickySessionTypeArg::None => Self::None,
            StickySessionTypeArg::Cookie => Self::Cookie,
        }
    }
}

/// Pull Zone billing / network tier (`Type` in the bunny.net API).
#[derive(Copy, Clone, ValueEnum)]
pub enum PullZoneTierTypeArg {
    /// Standard global network (value 0).
    Standard,
    /// Volume tier for high-traffic static assets (value 1).
    Volume,
}

impl From<PullZoneTierTypeArg> for PullZoneTierType {
    fn from(a: PullZoneTierTypeArg) -> Self {
        match a {
            PullZoneTierTypeArg::Standard => Self::Standard,
            PullZoneTierTypeArg::Volume => Self::Volume,
        }
    }
}

/// Shield DDoS protection mode (`ShieldDDosProtectionType` in the bunny.net API).
#[derive(Copy, Clone, ValueEnum)]
pub enum ShieldDDosProtectionTypeArg {
    /// Monitor suspected DDoS traffic but do not block it (value 0).
    DetectOnly,
    /// Actively block DDoS traffic with standard mitigation (value 1).
    ActiveStandard,
    /// Actively block DDoS traffic with aggressive mitigation (value 2).
    ActiveAggressive,
}

impl From<ShieldDDosProtectionTypeArg> for ShieldDDosProtectionType {
    fn from(a: ShieldDDosProtectionTypeArg) -> Self {
        match a {
            ShieldDDosProtectionTypeArg::DetectOnly => Self::DetectOnly,
            ShieldDDosProtectionTypeArg::ActiveStandard => Self::ActiveStandard,
            ShieldDDosProtectionTypeArg::ActiveAggressive => Self::ActiveAggressive,
        }
    }
}

/// Edge script type (`ScriptType` in the Compute API).
#[derive(Copy, Clone, ValueEnum)]
pub enum ScriptTypeArg {
    /// DNS-layer script (value 0).
    Dns,
    /// CDN script (value 1).
    Cdn,
    /// Middleware script (value 2).
    Middleware,
}

impl From<ScriptTypeArg> for u8 {
    fn from(a: ScriptTypeArg) -> Self {
        match a {
            ScriptTypeArg::Dns => 0,
            ScriptTypeArg::Cdn => 1,
            ScriptTypeArg::Middleware => 2,
        }
    }
}

/// Storage zone tier (`ZoneTier` in the Core API).
#[derive(Copy, Clone, ValueEnum)]
pub enum StorageZoneTierArg {
    /// Standard tier (value 0).
    Standard,
    /// Edge tier (value 1).
    Edge,
}

impl From<StorageZoneTierArg> for i64 {
    fn from(a: StorageZoneTierArg) -> Self {
        match a {
            StorageZoneTierArg::Standard => 0,
            StorageZoneTierArg::Edge => 1,
        }
    }
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)] // clap subcommand enums are boxed by clap internals
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
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
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
        /// Video library ID
        #[arg(long)]
        id: i64,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        date_to: Option<String>,
    },
    /// Get transcribing statistics for a video library
    TranscribingStatistics {
        /// Video library ID
        #[arg(long)]
        id: i64,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        date_to: Option<String>,
    },
}

// -- Pull Zone --

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)] // Update has many Option<T> flags; clap handles boxing
pub enum PullZoneAction {
    /// List all pull zones
    List {
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Page number (1-based)
        #[arg(long, conflicts_with = "all")]
        page: Option<u32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        per_page: Option<u32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a specific pull zone
    Get {
        /// Pull zone ID
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
        /// Pull Zone name. Becomes the hostname `<name>.b-cdn.net` and must be globally unique across bunny.net. Lowercase letters, digits, and hyphens only.
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
    #[command(after_help = "EXAMPLES:
  # Enable Optimizer with WebP, CSS/JS minify, and image quality 80
  hoppy pull-zone update --id <id> --optimizer-enabled true --optimizer-webp true \\
    --optimizer-minify-css true --optimizer-minify-js true --optimizer-image-quality 80

  # After enabling, read Optimizer usage stats
  hoppy pull-zone statistics --id <id> --type optimizer")]
    Update {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// HTTP/HTTPS origin URL. Mutually exclusive with --storage-zone-id.
        #[arg(long, conflicts_with = "storage_zone_id")]
        origin_url: Option<String>,
        /// Re-bind the Pull Zone to a different Storage Zone. Mutually
        /// exclusive with --origin-url.
        #[arg(long)]
        storage_zone_id: Option<i64>,
        /// Monthly bandwidth limit in bytes (0 = unlimited).
        #[arg(long)]
        monthly_bandwidth_limit: Option<i64>,
        /// Default cache TTL in seconds (-1 = honour origin headers).
        #[arg(long)]
        cache_expiration_time: Option<i64>,
        /// Enable Zone Security (token-signed URLs).
        #[arg(long)]
        zone_security_enabled: Option<bool>,
        /// Serve from US PoPs.
        #[arg(long)]
        enable_geo_zone_us: Option<bool>,
        /// Serve from EU PoPs.
        #[arg(long)]
        enable_geo_zone_eu: Option<bool>,
        /// Serve from Asia PoPs.
        #[arg(long)]
        enable_geo_zone_asia: Option<bool>,
        /// Serve from South America PoPs.
        #[arg(long)]
        enable_geo_zone_sa: Option<bool>,
        /// Serve from Africa PoPs.
        #[arg(long)]
        enable_geo_zone_af: Option<bool>,

        // ── Log forwarding ───────────────────────────────────────────────────
        /// Enable or disable CDN log forwarding to a remote syslog endpoint.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Log forwarding")]
        log_forwarding_enabled: Option<bool>,
        /// Syslog endpoint hostname.
        #[arg(long, value_name = "HOST", help_heading = "Log forwarding")]
        log_forwarding_hostname: Option<String>,
        /// Syslog endpoint port (1–65535).
        #[arg(long, value_name = "PORT", value_parser = clap::value_parser!(u16).range(1..), help_heading = "Log forwarding")]
        log_forwarding_port: Option<u16>,
        /// Authentication token for the syslog endpoint. Treated as a secret —
        /// redacted in JSON output unless `--reveal` is set.
        #[arg(long, value_name = "TOKEN", help_heading = "Log forwarding")]
        log_forwarding_token: Option<String>,
        /// Transport protocol: udp, tcp, tcp-encrypted, datadog.
        #[arg(long, value_name = "PROTO", help_heading = "Log forwarding")]
        log_forwarding_protocol: Option<PullZoneLogForwardingProtocolArg>,
        /// Save permanent CDN logs to a storage zone (set --logging-storage-zone-id).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Log forwarding")]
        logging_save_to_storage: Option<bool>,
        /// Storage zone ID that receives permanent logs.
        #[arg(long, value_name = "ID", help_heading = "Log forwarding")]
        logging_storage_zone_id: Option<i64>,

        // ── Optimizer ────────────────────────────────────────────────────────
        /// Enable or disable the Bunny Optimizer for this Pull Zone.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_enabled: Option<bool>,
        /// Let Optimizer auto-select settings based on content type.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_automatic_optimization: Option<bool>,
        /// Maximum image width (px) served to desktop visitors.
        #[arg(long, value_name = "PX", help_heading = "Optimizer")]
        optimizer_desktop_max_width: Option<i32>,
        /// Maximum image width (px) served to mobile visitors.
        #[arg(long, value_name = "PX", help_heading = "Optimizer")]
        optimizer_mobile_max_width: Option<i32>,
        /// JPEG/WebP quality (0–100) for desktop images.
        #[arg(long, value_name = "0-100", help_heading = "Optimizer")]
        optimizer_image_quality: Option<i32>,
        /// JPEG/WebP quality (0–100) for mobile images.
        #[arg(long, value_name = "0-100", help_heading = "Optimizer")]
        optimizer_mobile_image_quality: Option<i32>,
        /// Convert images to WebP when the browser supports it.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_webp: Option<bool>,
        /// Upscale images smaller than the max-width to fill the requested size.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_upscaling: Option<bool>,
        /// Minify CSS files on the fly.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_minify_css: Option<bool>,
        /// Minify JavaScript files on the fly.
        /// Serialises to `OptimizerMinifyJavaScript` on the wire.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_minify_js: Option<bool>,
        /// Enable the image manipulation engine (resize, crop, etc. via URL params).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_manipulation_engine: Option<bool>,
        /// JSON map of named class definitions (class-name → URL params).
        /// See https://docs.bunny.net/docs/optimizer-classes for format.
        /// Example: `'{"thumb":"width=200,quality=80"}'`
        #[arg(long, value_name = "JSON", help_heading = "Optimizer")]
        optimizer_classes: Option<String>,
        /// Force the use of classes defined in --optimizer-classes.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_force_classes: Option<bool>,
        /// Enable watermark overlay on images.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_watermark: Option<bool>,
        /// URL of the watermark image.
        #[arg(long, value_name = "URL", help_heading = "Optimizer")]
        optimizer_watermark_url: Option<String>,
        /// Position of the watermark: top-left, top-right, bottom-left, bottom-right, center.
        #[arg(long, value_name = "POS", help_heading = "Optimizer")]
        optimizer_watermark_position: Option<OptimizerWatermarkPositionArg>,
        /// Watermark offset from the edge as a percentage (0.0–100.0).
        #[arg(long, value_name = "PCT", help_heading = "Optimizer")]
        optimizer_watermark_offset: Option<f64>,
        /// Minimum image size (px) required for a watermark to be applied.
        #[arg(long, value_name = "PX", help_heading = "Optimizer")]
        optimizer_watermark_min_image_size: Option<i32>,
        /// Enable Optimizer Static HTML (WordPress caching).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_static_html: Option<bool>,
        /// WordPress installation path used by Static HTML caching.
        #[arg(long, value_name = "PATH", help_heading = "Optimizer")]
        optimizer_static_html_wp_path: Option<String>,
        /// Cookie name that bypasses Static HTML caching for logged-in users.
        #[arg(long, value_name = "NAME", help_heading = "Optimizer")]
        optimizer_static_html_wp_bypass_cookie: Option<String>,
        /// Pre-render HTML for crawlers (may increase origin load).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_prerender_html: Option<bool>,
        /// Route Optimizer traffic through a dedicated tunnel for origin privacy.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Optimizer")]
        optimizer_tunnel: Option<bool>,

        // ── Security / compliance (iter-44) ──────────────────────────────────
        /// Allow TLS 1.0 to the CDN edge. Disable for PCI/SOC2 compliance.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        enable_tls1: Option<bool>,
        /// Allow TLS 1.1 to the CDN edge. Disable for PCI/SOC2 compliance.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        enable_tls1_1: Option<bool>,
        /// Automatically provision an SSL certificate for custom hostnames.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        enable_auto_ssl: Option<bool>,
        /// Disable Let's Encrypt provisioning (use only your own uploaded certs).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        disable_lets_encrypt: Option<bool>,
        /// Verify the origin's SSL certificate before serving cached content.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        verify_origin_ssl: Option<bool>,
        /// Send an `Access-Control-Allow-Origin` (CORS) header on responses.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        enable_access_control_origin_header: Option<bool>,
        /// Comma-separated file extensions that receive the CORS header
        /// (e.g. `woff,woff2,ttf`). Pass an empty string to clear.
        #[arg(
            long,
            value_name = "EXTS",
            value_delimiter = ',',
            help_heading = "Security / compliance"
        )]
        access_control_origin_header_extensions: Option<Vec<String>>,
        /// Include a hash of the client's remote IP in the Zone Security
        /// token, binding signed URLs to a single IP.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        zone_security_include_hash_remote_ip: Option<bool>,
        /// Sign every origin request with AWS Signature V4 (required for
        /// private S3-compatible origins).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        aws_signing_enabled: Option<bool>,
        /// AWS Access Key ID for SigV4 origin signing.
        /// WARNING: passed on the command line appears in shell history;
        /// prefer an env var or a wrapper script.
        #[arg(long, value_name = "KEY", help_heading = "Security / compliance")]
        aws_signing_key: Option<String>,
        /// AWS Secret Access Key for SigV4 origin signing.
        /// WARNING: passed on the command line appears in shell history;
        /// prefer an env var or a wrapper script.
        #[arg(long, value_name = "SECRET", help_heading = "Security / compliance")]
        aws_signing_secret: Option<String>,
        /// AWS region name for SigV4 origin signing (e.g. `us-east-1`).
        #[arg(long, value_name = "REGION", help_heading = "Security / compliance")]
        aws_signing_region_name: Option<String>,
        /// Anonymise client IPs in CDN logs (GDPR-friendly).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Security / compliance")]
        logging_ip_anonymization_enabled: Option<bool>,
        /// How the last IP octet is anonymised: `one-digit` or `drop`.
        #[arg(long, value_name = "TYPE", help_heading = "Security / compliance")]
        log_anonymization_type: Option<LogAnonymizationTypeArg>,

        // ── Vary headers (iter-45) ───────────────────────────────────────────
        /// Vary cache entries by client WebP support (serve WebP variants when available).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Vary headers")]
        enable_webp_vary: Option<bool>,
        /// Vary cache entries by client AVIF support (serve AVIF variants when available).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Vary headers")]
        enable_avif_vary: Option<bool>,
        /// Vary cache entries by the `Cookie` request header (combine with
        /// `--cookie-vary-parameters` to scope to specific cookies).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Vary headers")]
        enable_cookie_vary: Option<bool>,
        /// Vary cache entries by the client's country code (GeoIP).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Vary headers")]
        enable_country_code_vary: Option<bool>,
        /// Vary cache entries by the client's country + state/region code.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Vary headers")]
        enable_country_state_code_vary: Option<bool>,
        /// Vary cache entries by the requested `Host` header.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Vary headers")]
        enable_hostname_vary: Option<bool>,
        /// Vary cache entries by mobile vs desktop user-agent detection.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Vary headers")]
        enable_mobile_vary: Option<bool>,

        // ── Performance / caching (iter-45) ──────────────────────────────────
        /// Enable Cache Slice — serve large files in parallel range slices.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        enable_cache_slice: Option<bool>,
        /// Enable Smart Cache — bunny.net heuristics override cache headers.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        enable_smart_cache: Option<bool>,
        /// Enable Safe Hop — retry failed origin requests via a healthy region.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        enable_safe_hop: Option<bool>,
        /// Strip query strings from the cache key (cache as if no query string).
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        ignore_query_strings: Option<bool>,
        /// Sort query string parameters before computing the cache key,
        /// so `?a=1&b=2` and `?b=2&a=1` share one cache entry.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        enable_query_string_ordering: Option<bool>,
        /// Comma-separated query string parameters to include in the cache
        /// key (e.g. `v,locale`). Pass an empty string to clear.
        #[arg(
            long,
            value_name = "PARAMS",
            value_delimiter = ',',
            help_heading = "Performance / caching"
        )]
        query_string_vary_parameters: Option<Vec<String>>,
        /// Comma-separated cookies to include in the cache key
        /// (e.g. `session,locale`). Pass an empty string to clear.
        #[arg(
            long,
            value_name = "COOKIES",
            value_delimiter = ',',
            help_heading = "Performance / caching"
        )]
        cookie_vary_parameters: Option<Vec<String>>,
        /// Serve stale content while a fresh copy is fetched in the background.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        use_stale_while_updating: Option<bool>,
        /// Serve stale content if the origin is unreachable.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        use_stale_while_offline: Option<bool>,
        /// Refresh cache entries in the background before they expire.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        use_background_update: Option<bool>,
        /// Override `Cache-Control` max-age (seconds) for edge caching. 0 = disable override.
        #[arg(
            long,
            value_name = "SECONDS",
            value_parser = clap::value_parser!(i64).range(0..),
            help_heading = "Performance / caching"
        )]
        cache_control_max_age_override: Option<i64>,
        /// Override `Cache-Control: public` max-age (seconds). 0 = disable override.
        #[arg(
            long,
            value_name = "SECONDS",
            value_parser = clap::value_parser!(i64).range(0..),
            help_heading = "Performance / caching"
        )]
        cache_control_public_max_age_override: Option<i64>,
        /// Override `Cache-Control` max-age (seconds) sent to the browser. 0 = disable override.
        #[arg(
            long,
            value_name = "SECONDS",
            value_parser = clap::value_parser!(i64).range(0..),
            help_heading = "Performance / caching"
        )]
        cache_control_browser_max_age_override: Option<i64>,
        /// Cache error responses (4xx/5xx) briefly to shield the origin from
        /// thundering-herd retries.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Performance / caching")]
        cache_error_responses: Option<bool>,
        /// Perma-Cache target storage zone ID — content is permanently
        /// retained in this storage zone.
        #[arg(long, value_name = "ID", help_heading = "Performance / caching")]
        perma_cache_storage_zone_id: Option<i64>,
        /// Perma-Cache mode: `automatic` (retain everything) or `manual`
        /// (only what's explicitly pushed).
        #[arg(long, value_name = "TYPE", help_heading = "Performance / caching")]
        perma_cache_type: Option<PermaCacheTypeArg>,

        // ── Origin host / DNS (iter-46) ──────────────────────────────────────
        /// Override the `Host` header sent to the origin server.
        #[arg(long, value_name = "HOST", help_heading = "Origin host / DNS")]
        origin_host_header: Option<String>,
        /// Forward the original client `Host` header to the origin.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin host / DNS")]
        add_host_header: Option<bool>,
        /// Add a `Link: <url>; rel="canonical"` header to responses.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin host / DNS")]
        add_canonical_header: Option<bool>,
        /// DNS origin port override.
        #[arg(
            long,
            value_name = "PORT",
            value_parser = clap::value_parser!(i32).range(1..=65535),
            help_heading = "Origin host / DNS"
        )]
        dns_origin_port: Option<i32>,
        /// DNS origin scheme override (e.g. `https`).
        #[arg(long, value_name = "SCHEME", help_heading = "Origin host / DNS")]
        dns_origin_scheme: Option<String>,
        /// Follow HTTP redirects returned by the origin.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin host / DNS")]
        follow_redirects: Option<bool>,

        // ── Origin timeouts / retries (iter-46) ──────────────────────────────
        /// Seconds to wait for a TCP connection to the origin (1–60 s).
        #[arg(
            long,
            value_name = "SECONDS",
            value_parser = clap::value_parser!(i32).range(1..=60),
            help_heading = "Origin timeouts / retries"
        )]
        origin_connect_timeout: Option<i32>,
        /// Seconds to wait for the origin to send the first response byte (1–120 s).
        #[arg(
            long,
            value_name = "SECONDS",
            value_parser = clap::value_parser!(i32).range(1..=120),
            help_heading = "Origin timeouts / retries"
        )]
        origin_response_timeout: Option<i32>,
        /// Number of times to retry a failed origin request (0–5).
        #[arg(
            long,
            value_name = "COUNT",
            value_parser = clap::value_parser!(i32).range(0..=5),
            help_heading = "Origin timeouts / retries"
        )]
        origin_retries: Option<i32>,
        /// Retry when the origin returns a 5xx error response.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin timeouts / retries")]
        origin_retry_5xx_responses: Option<bool>,
        /// Retry when a connection to the origin times out.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin timeouts / retries")]
        origin_retry_connection_timeout: Option<bool>,
        /// Seconds to wait between retry attempts (0–60 s).
        #[arg(
            long,
            value_name = "SECONDS",
            value_parser = clap::value_parser!(i32).range(0..=60),
            help_heading = "Origin timeouts / retries"
        )]
        origin_retry_delay: Option<i32>,
        /// Retry when the origin response times out.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin timeouts / retries")]
        origin_retry_response_timeout: Option<bool>,

        // ── Origin shield (iter-46) ───────────────────────────────────────────
        /// Enable the origin shield — cache miss requests are funnelled through
        /// a single shield node to reduce origin load.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin shield")]
        enable_origin_shield: Option<bool>,
        /// Limit the number of concurrent requests forwarded to the shield node.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Origin shield")]
        origin_shield_enable_concurrency_limit: Option<bool>,
        /// Maximum concurrent requests the shield will forward to the origin.
        #[arg(long, value_name = "COUNT", help_heading = "Origin shield")]
        origin_shield_max_concurrent_requests: Option<i32>,
        /// Maximum number of requests queued at the shield node.
        #[arg(long, value_name = "COUNT", help_heading = "Origin shield")]
        origin_shield_max_queued_requests: Option<i32>,
        /// Seconds a queued shield request waits before being dropped.
        #[arg(long, value_name = "SECONDS", help_heading = "Origin shield")]
        origin_shield_queue_max_wait_time: Option<i32>,
        /// ISO code of the PoP to use as origin shield (e.g. `FR`, `NY`).
        #[arg(long, value_name = "CODE", help_heading = "Origin shield")]
        origin_shield_zone_code: Option<String>,

        // ── Routing / sticky sessions (iter-46) ──────────────────────────────
        /// Coalesce concurrent cache-miss requests for the same URL into a
        /// single upstream request.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Routing / sticky sessions")]
        enable_request_coalescing: Option<bool>,
        /// Seconds to wait for a coalesced request to complete before sending
        /// a new request to the origin (1–60 s).
        #[arg(
            long,
            value_name = "SECONDS",
            value_parser = clap::value_parser!(i32).range(1..=60),
            help_heading = "Routing / sticky sessions"
        )]
        request_coalescing_timeout: Option<i32>,
        /// Comma-separated routing filter names to apply.
        #[arg(
            long,
            value_name = "FILTERS",
            value_delimiter = ',',
            help_heading = "Routing / sticky sessions"
        )]
        routing_filters: Option<Vec<String>>,
        /// Sticky session mode: `none` or `cookie`.
        #[arg(long, value_name = "TYPE", help_heading = "Routing / sticky sessions")]
        sticky_session_type: Option<StickySessionTypeArg>,
        /// Name of the cookie used for sticky session affinity.
        #[arg(long, value_name = "NAME", help_heading = "Routing / sticky sessions")]
        sticky_session_cookie_name: Option<String>,
        /// Comma-separated request headers used to identify a sticky session client.
        #[arg(
            long,
            value_name = "HEADERS",
            value_delimiter = ',',
            help_heading = "Routing / sticky sessions"
        )]
        sticky_session_client_headers: Option<Vec<String>>,
        /// Pull Zone billing tier: `standard` or `volume`.
        #[arg(long, value_name = "TIER", help_heading = "Routing / sticky sessions")]
        pull_zone_tier_type: Option<PullZoneTierTypeArg>,

        // ── Firewall / rate limiting (iter-47) ───────────────────────────────
        /// ISO-3166-1 alpha-2 country codes to block (e.g. `CN,RU`).
        #[arg(
            long,
            value_name = "CODES",
            value_delimiter = ',',
            help_heading = "Firewall / rate limiting"
        )]
        blocked_countries: Option<Vec<String>>,
        /// ISO-3166-1 alpha-2 country codes to redirect to a cheaper PoP.
        #[arg(
            long,
            value_name = "CODES",
            value_delimiter = ',',
            help_heading = "Firewall / rate limiting"
        )]
        budget_redirected_countries: Option<Vec<String>>,
        /// IP addresses or CIDR ranges to block (e.g. `1.2.3.4,10.0.0.0/8`).
        #[arg(
            long,
            value_name = "IPS",
            value_delimiter = ',',
            help_heading = "Firewall / rate limiting"
        )]
        blocked_ips: Option<Vec<String>>,
        /// Allowed referrer domains (anti-hotlinking allowlist).
        #[arg(
            long,
            value_name = "DOMAINS",
            value_delimiter = ',',
            help_heading = "Firewall / rate limiting"
        )]
        allowed_referrers: Option<Vec<String>>,
        /// Blocked referrer domains (anti-hotlinking blocklist).
        #[arg(
            long,
            value_name = "DOMAINS",
            value_delimiter = ',',
            help_heading = "Firewall / rate limiting"
        )]
        blocked_referrers: Option<Vec<String>>,
        /// Block requests with no `Referer` header.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Firewall / rate limiting")]
        block_none_referrer: Option<bool>,
        /// Block HTTP POST requests to this pull zone.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Firewall / rate limiting")]
        block_post_requests: Option<bool>,
        /// Block requests to the root path (`/`) of this pull zone.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Firewall / rate limiting")]
        block_root_path_access: Option<bool>,
        /// Strip cookies from requests and responses.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Firewall / rate limiting")]
        disable_cookies: Option<bool>,
        /// Enable Shield DDoS protection for this pull zone.
        #[arg(long, action = clap::ArgAction::Set, help_heading = "Firewall / rate limiting")]
        shield_ddos_protection_enabled: Option<bool>,
        /// Shield DDoS protection mode: `detect-only`, `active-standard`, `active-aggressive`.
        #[arg(long, value_name = "MODE", help_heading = "Firewall / rate limiting")]
        shield_ddos_protection_type: Option<ShieldDDosProtectionTypeArg>,
        /// Maximum burst of requests above the rate limit before throttling.
        #[arg(
            long,
            value_name = "REQUESTS",
            value_parser = clap::value_parser!(i32).range(0..),
            help_heading = "Firewall / rate limiting"
        )]
        burst_size: Option<i32>,
        /// Maximum number of requests per second allowed from a single IP.
        #[arg(
            long,
            value_name = "REQ_PER_SEC",
            value_parser = clap::value_parser!(i32).range(0..),
            help_heading = "Firewall / rate limiting"
        )]
        request_limit: Option<i32>,
        /// Bytes served before rate limiting kicks in for a connection. Accepts
        /// fractional values to match the API's `double` type.
        #[arg(long, value_name = "BYTES", help_heading = "Firewall / rate limiting")]
        limit_rate_after: Option<f64>,
        /// Bytes per second allowed after `--limit-rate-after` bytes have been served.
        /// Accepts fractional values to match the API's `double` type.
        #[arg(
            long,
            value_name = "BYTES_PER_SEC",
            help_heading = "Firewall / rate limiting"
        )]
        limit_rate_per_second: Option<f64>,
        /// Maximum number of concurrent connections per IP address.
        #[arg(
            long,
            value_name = "COUNT",
            value_parser = clap::value_parser!(i32).range(0..),
            help_heading = "Firewall / rate limiting"
        )]
        connection_limit_per_ip_count: Option<i32>,
        /// Maximum number of concurrent WebSocket connections.
        #[arg(
            long,
            value_name = "COUNT",
            value_parser = clap::value_parser!(i32).range(0..),
            help_heading = "Firewall / rate limiting"
        )]
        max_web_socket_connections: Option<i32>,
    },
    /// Delete a pull zone
    Delete {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
    },
    /// Purge pull zone cache
    Purge {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Limit purge to a specific cache tag
        #[arg(long)]
        cache_tag: Option<String>,
    },
    /// Get pull zone statistics
    Statistics {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Statistics type: optimizer, origin-shield, safehop
        #[arg(long, value_name = "TYPE")]
        r#type: String,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
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
        /// Pull zone ID
        #[arg(long)]
        id: i64,
    },
    /// Add a hostname to the allowed referrer list
    Allow {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Referrer hostname pattern (e.g. example.com or *.example.com)
        #[arg(long)]
        value: String,
    },
    /// Remove a hostname from the allowed referrer list
    RemoveAllowed {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Referrer hostname pattern to remove
        #[arg(long)]
        value: String,
    },
    /// Add a hostname to the blocked referrer list
    Block {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Referrer hostname pattern (e.g. example.com or *.example.com)
        #[arg(long)]
        value: String,
    },
    /// Remove a hostname from the blocked referrer list
    RemoveBlocked {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Referrer hostname pattern to remove
        #[arg(long)]
        value: String,
    },
}

#[derive(Subcommand)]
pub enum PullZoneIpAction {
    /// List blocked IPs for a pull zone
    List {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
    },
    /// Block an IP address (single IP or CIDR range)
    Block {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// IP address or CIDR range to block (e.g. 1.2.3.4 or 10.0.0.0/8)
        #[arg(long)]
        value: String,
    },
    /// Unblock an IP address
    Unblock {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// IP address or CIDR range to unblock
        #[arg(long)]
        value: String,
    },
}

#[derive(Subcommand)]
pub enum EdgeRuleAction {
    /// List edge rules on a pull zone
    List {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
    },
    /// Add an edge rule to a pull zone
    Add {
        /// Pull zone ID
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
        /// Pull zone ID
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
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// GUID of the edge rule to delete
        #[arg(long)]
        rule_id: String,
    },
    /// Enable or disable an edge rule
    Enable {
        /// Pull zone ID
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
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Custom hostname to attach (e.g. cdn.example.com)
        #[arg(long)]
        hostname: String,
    },
    /// Remove a custom hostname
    Remove {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Custom hostname to remove
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
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Hostname to configure Force SSL on
        #[arg(long)]
        hostname: String,
        /// Enable or disable Force SSL
        #[arg(long, action = clap::ArgAction::Set)]
        enabled: bool,
    },
    /// Add a custom SSL certificate (certificate and key must be Base64-encoded PEM)
    AddCert {
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Hostname the certificate applies to
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
        /// Pull zone ID
        #[arg(long)]
        id: i64,
        /// Hostname to remove the certificate from
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
        #[arg(long, conflicts_with = "all")]
        page: Option<u32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        per_page: Option<u32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a specific storage zone
    Get {
        /// Storage zone ID
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
        /// Storage zone name (used as the hostname prefix for the storage endpoint).
        #[arg(long)]
        name: String,
        /// Primary region (e.g. DE, NY, LA, SG, SYD)
        #[arg(long)]
        region: String,
        /// Replication regions (comma-separated or repeated flags)
        #[arg(long, value_delimiter = ',')]
        replication_regions: Vec<String>,
        /// Storage zone tier
        #[arg(long, value_enum)]
        zone_tier: Option<StorageZoneTierArg>,
    },
    /// Update a storage zone
    Update {
        /// Storage zone ID
        #[arg(long)]
        id: i64,
        /// Rewrite 404 responses to 200 (useful for SPAs).
        #[arg(long)]
        rewrite_404_to_200: Option<bool>,
        /// Path to a custom 404 file served in place of the default error page.
        #[arg(long)]
        custom_404_file_path: Option<String>,
        /// HTTP/HTTPS origin URL for the storage zone (when used as an origin mirror).
        #[arg(long)]
        origin_url: Option<String>,
    },
    /// Delete a storage zone
    Delete {
        /// Storage zone ID
        #[arg(long)]
        id: i64,
    },
    /// Get statistics for a storage zone
    Statistics {
        /// Storage zone ID
        #[arg(long)]
        id: i64,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
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
        /// Local file path to write the downloaded file (defaults to stdout if omitted)
        #[arg(long)]
        file: Option<String>,
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
        remote_path: String,
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
        #[arg(long, conflicts_with = "all")]
        page: Option<u32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        per_page: Option<u32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a specific DNS zone
    Get {
        /// DNS zone ID
        #[arg(long)]
        id: i64,
    },
    /// Create a DNS zone
    Create {
        /// Domain name for the new DNS zone (e.g. example.com)
        #[arg(long)]
        domain: String,
    },
    /// Update a DNS zone
    Update {
        /// DNS zone ID
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
        /// DNS zone ID
        #[arg(long)]
        id: i64,
    },
    /// Get statistics for a DNS zone
    Statistics {
        /// DNS zone ID
        #[arg(long)]
        id: i64,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        date_to: Option<String>,
    },
    /// Export DNS zone as a BIND zone file (text and table both emit raw BIND; --format json wraps it in {"Bind": ...})
    Export {
        /// DNS zone ID
        #[arg(long)]
        id: i64,
    },
    /// Import DNS records from a BIND zone file
    Import {
        /// DNS zone ID
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
        /// DNS zone ID
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
        /// DNS zone ID
        #[arg(long)]
        id: i64,
    },
    /// Disable DNSSEC.
    ///
    /// WARNING: if DS records are still configured at your registrar,
    /// disabling DNSSEC at bunny.net will break resolution. Remove the DS
    /// records from your registrar first.
    Disable {
        /// DNS zone ID
        #[arg(long)]
        id: i64,
    },
    /// Show the current DNSSEC status (read from the DNS zone metadata).
    Status {
        /// DNS zone ID
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
    #[command(group = clap::ArgGroup::new("scan_target").required(true).args(["id", "domain"]))]
    Start {
        /// DNS Zone ID (use this for existing zones)
        #[arg(long, conflicts_with = "domain")]
        id: Option<i64>,
        /// Domain name (use this for pre-zone-creation scans)
        #[arg(long, conflicts_with = "id")]
        domain: Option<String>,
    },
    /// Show the latest scan results for a zone.
    ///
    /// Provide either `--id <zone-id>` for an existing zone or `--domain
    /// <domain>` (looked up via the zone list). At least one is required;
    /// they are mutually exclusive.
    ///
    /// Note: the bunny.net API only exposes scan results keyed by zone id.
    /// `--domain` resolves to a zone id by searching the zone list, so it
    /// only works once the zone has been created. For a pre-onboarding
    /// scan, create the zone first via `hoppy dns zone create --domain
    /// <d>` and then re-run this command.
    #[command(group = clap::ArgGroup::new("scan_results_target").required(true).args(["id", "domain"]))]
    Results {
        /// DNS Zone ID (use this for existing zones)
        #[arg(long, conflicts_with = "domain")]
        id: Option<i64>,
        /// Domain name (resolved to a zone id via the zone list)
        #[arg(long, conflicts_with = "id")]
        domain: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DnsRecordAction {
    /// List records in a DNS zone
    List {
        /// DNS zone ID
        #[arg(long = "id", alias = "zone-id", value_name = "ID")]
        zone_id: i64,
    },
    /// Add a DNS record.
    ///
    /// EXAMPLES:
    ///   hoppy dns record add --id 50001 --type A    --value 192.0.2.1
    ///   hoppy dns record add --id 50001 --type CNAME --name www --value example.com
    ///   hoppy dns record add --id 50001 --type MX    --value mail.example.com --priority 10
    ///   hoppy dns record add --id 50001 --type CAA   --value letsencrypt.org --tag issue --flags 0
    ///
    /// Tip: for Magic-Container-backed Pull Zones, use a `CNAME` record
    /// pointing at the `b-cdn.net` hostname instead of `--type PullZone`
    /// (the latter only accepts standard, non-managed Pull Zone IDs and
    /// will return "pull zone ID is not valid" otherwise).
    Add {
        /// DNS zone ID
        #[arg(long = "id", alias = "zone-id", value_name = "ID")]
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
        /// DNS zone ID
        #[arg(long)]
        zone_id: i64,
        /// DNS record ID
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
        /// DNS zone ID
        #[arg(long)]
        zone_id: i64,
        /// DNS record ID
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
    ///
    /// The table/text output omits `ApiKey` and `ReadOnlyApiKey`. JSON
    /// output redacts them by default; pass the global `--reveal` flag
    /// to print the raw values in JSON.
    List {
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Page number (1-based)
        #[arg(long, conflicts_with = "all")]
        page: Option<u32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        per_page: Option<u32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a specific video library
    ///
    /// `ApiKey` and `ReadOnlyApiKey` are redacted by default in every
    /// output format. Pass the global `--reveal` flag to print the raw
    /// values.
    Get {
        /// Stream library ID
        #[arg(long)]
        id: i64,
    },
    /// Create a video library
    ///
    /// The response includes `ApiKey` and `ReadOnlyApiKey`, which are
    /// redacted by default. Pass the global `--reveal` flag to capture
    /// the new credentials in scripts.
    Create {
        /// Display name for the new video library.
        #[arg(long)]
        name: String,
    },
    /// Update a video library
    Update {
        /// Stream library ID
        #[arg(long)]
        id: i64,
        /// New display name for the library.
        #[arg(long)]
        name: Option<String>,
        /// Allow viewers to directly download or play the original video.
        #[arg(long)]
        allow_direct_play: Option<bool>,
        /// Generate MP4 fallback renditions for compatibility with older players.
        #[arg(long)]
        enable_mp4_fallback: Option<bool>,
        /// Overlay a watermark on encoded video renditions.
        #[arg(long)]
        has_watermark: Option<bool>,
    },
    /// Delete a video library
    Delete {
        /// Stream library ID
        #[arg(long)]
        id: i64,
    },
    /// Get statistics for a video library
    Statistics {
        /// Stream library ID
        #[arg(long)]
        id: i64,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// End date (YYYY-MM-DD)
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
        /// Video library ID
        #[arg(long = "id", alias = "library-id", value_name = "ID")]
        library_id: i64,
        /// Page number (1-based)
        #[arg(long, conflicts_with = "all")]
        page: Option<u32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
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
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a specific video
    Get {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
    },
    /// Upload a video file (two-step: create + upload binary)
    Upload {
        /// Video library ID
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
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
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
        /// Video library ID
        #[arg(long = "id", alias = "library-id", value_name = "ID")]
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
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
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
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
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
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
    },
    /// Re-encode a video (optionally for a specific codec)
    Reencode {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
        /// Optional output codec (x264, vp9, hevc, av1)
        #[arg(long)]
        codec: Option<String>,
    },
    /// Repackage a video's HLS/DASH manifests
    Repackage {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
        /// Discard previous file versions (default: keep)
        #[arg(long)]
        discard_originals: bool,
    },
    /// Trigger smart-generate (AI title/description/chapters/moments)
    SmartGenerate {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
        /// Language code for AI generation (ISO 639-1, e.g. `en`)
        #[arg(long)]
        language: Option<String>,
        /// Auto-generate a title for the video
        #[arg(long)]
        generate_title: bool,
        /// Auto-generate a description for the video
        #[arg(long)]
        generate_description: bool,
        /// Auto-generate chapter markers for the video
        #[arg(long)]
        generate_chapters: bool,
        /// Auto-generate moment highlights for the video
        #[arg(long)]
        generate_moments: bool,
    },
    /// Set the thumbnail for a video from a URL
    SetThumbnail {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
        /// Public URL of the image to use as the video thumbnail
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
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
    },
}

#[derive(Subcommand)]
pub enum StreamResolutionsAction {
    /// List configured/available resolutions for a video
    List {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
        #[arg(long)]
        video_id: String,
    },
    /// Cleanup video resolutions/files (destructive — confirmation required unless --yes or --dry-run)
    Cleanup {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
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
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
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
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Video GUID
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
        /// Video library ID
        #[arg(long = "id", alias = "library-id", value_name = "ID")]
        library_id: i64,
        /// Page number (1-based)
        #[arg(long, conflicts_with = "all")]
        page: Option<u32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        items_per_page: Option<u32>,
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Sort order
        #[arg(long)]
        order_by: Option<String>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a specific collection
    Get {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Collection GUID
        #[arg(long)]
        collection_id: String,
    },
    /// Create a new collection
    Create {
        /// Video library ID
        #[arg(long = "id", alias = "library-id", value_name = "ID")]
        library_id: i64,
        /// Collection name
        #[arg(long)]
        name: String,
    },
    /// Update a collection
    Update {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Collection GUID
        #[arg(long)]
        collection_id: String,
        /// New name for the collection
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a collection
    Delete {
        /// Video library ID
        #[arg(long)]
        library_id: i64,
        /// Collection GUID
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
    /// Manage API Guardian (OpenAPI schema enforcement)
    ApiGuardian {
        #[command(subcommand)]
        action: ShieldApiGuardianAction,
    },
    /// Manage upload scanning configuration
    UploadScanning {
        #[command(subcommand)]
        action: ShieldUploadScanningAction,
    },
    /// Retrieve event logs for a Shield Zone
    EventLogs {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
        /// Date of logs: ISO 8601 (YYYY-MM-DD) or legacy US format (MM-dd-yyyy)
        #[arg(long)]
        date: String,
        /// Continuation token for the next page (omit for the first page)
        #[arg(long)]
        continuation_token: Option<String>,
        /// Automatically paginate through all available pages
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// Get the mapping of Shield Zones to Pull Zones
    PullzoneMapping,
}

#[derive(Subcommand)]
pub enum ShieldMetricsAction {
    /// Get metrics overview for a Shield Zone
    Overview {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get detailed metrics for a Shield Zone (time-series breakdown)
    Detailed {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get rate limit metrics for all rules in a Shield Zone
    RateLimits {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get rate limit metrics for a single rule
    RateLimit {
        /// Rate limit rule ID
        #[arg(long)]
        id: i64,
    },
    /// Get WAF rule metrics for a specific rule in a Shield Zone
    WafRule {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
        /// WAF rule ID
        #[arg(long)]
        rule_id: i64,
    },
    /// Get bot detection metrics for a Shield Zone
    BotDetection {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get upload scanning metrics for a Shield Zone
    UploadScanning {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
}

#[derive(Subcommand)]
pub enum ShieldZoneAction {
    /// List all Shield Zones
    List,
    /// Get a Shield Zone by ID
    Get {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get a Shield Zone by Pull Zone ID
    GetByPullzone {
        /// Pull zone ID to look up the associated Shield Zone for
        #[arg(long)]
        pull_zone_id: i64,
    },
    /// Create a Shield Zone for a Pull Zone
    Create {
        /// Pull zone ID to attach the new Shield Zone to
        #[arg(long)]
        pull_zone_id: i64,
    },
    /// Update a Shield Zone's configuration
    Update {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
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
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get a custom WAF rule by ID
    GetRule {
        /// WAF rule ID
        #[arg(long)]
        id: i64,
    },
    /// Add a custom WAF rule
    AddRule {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
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
        /// WAF rule ID
        #[arg(long)]
        id: i64,
        /// Rule name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a custom WAF rule
    DeleteRule {
        /// WAF rule ID
        #[arg(long)]
        id: i64,
    },
    /// List triggered WAF rules for a Shield Zone
    TriggeredRules {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Review/update action for a triggered WAF rule
    ReviewTriggeredRule {
        /// Shield zone ID
        #[arg(long)]
        shield_zone_id: i64,
        /// Rule ID to review
        #[arg(long)]
        rule_id: String,
        /// Review action (0 = Pending, 1 = Approve, 2 = Reject)
        #[arg(long)]
        action: u8,
    },
    /// Get an AI recommendation for a triggered WAF rule
    RecommendTriggeredRule {
        /// Shield zone ID
        #[arg(long)]
        shield_zone_id: i64,
        /// Rule ID
        #[arg(long)]
        rule_id: String,
    },
    /// Get WAF rules segmented by plan tier
    PlanSegmentation,
    /// Get WAF engine configuration variables
    EngineConfig,
}

#[derive(Subcommand)]
pub enum ShieldApiGuardianAction {
    /// Get the API Guardian configuration for a Shield Zone
    Get {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Upload a new OpenAPI specification to API Guardian
    Upload {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
        /// Path to the OpenAPI specification file
        #[arg(long)]
        spec_file: std::path::PathBuf,
        /// Enforce authorization validation for all endpoints
        #[arg(long)]
        enforce_authorization: Option<bool>,
    },
    /// Update the API Guardian configuration with an updated OpenAPI spec
    Update {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
        /// Path to the OpenAPI specification file
        #[arg(long)]
        spec_file: std::path::PathBuf,
        /// Enforce authorization validation for all endpoints
        #[arg(long)]
        enforce_authorization: Option<bool>,
    },
    /// Update an individual API Guardian endpoint configuration
    UpdateEndpoint {
        /// Shield zone ID
        #[arg(long)]
        shield_zone_id: i64,
        /// Endpoint ID
        #[arg(long)]
        endpoint_id: i64,
        /// Enable or disable this endpoint
        #[arg(long)]
        enabled: Option<bool>,
        /// Validate the request body schema
        #[arg(long)]
        validate_request_body_schema: Option<bool>,
        /// Validate the response body schema
        #[arg(long)]
        validate_response_body_schema: Option<bool>,
        /// Validate authorization
        #[arg(long)]
        validate_authorization: Option<bool>,
    },
}

#[derive(Subcommand)]
pub enum ShieldUploadScanningAction {
    /// Get the upload scanning configuration for a Shield Zone
    Get {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Update the upload scanning configuration for a Shield Zone
    Update {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
        /// Enable or disable upload scanning
        #[arg(long)]
        enabled: Option<bool>,
        /// Antivirus scanning mode (0 = Disabled, 1 = LogOnly, 2 = Block)
        #[arg(long)]
        antivirus_mode: Option<u8>,
        /// CSAM scanning mode (0 = Disabled, 1 = LogOnly, 2 = Block)
        #[arg(long)]
        csam_mode: Option<u8>,
    },
}

#[derive(Subcommand)]
pub enum ShieldRateLimitAction {
    /// List rate limit rules for a Shield Zone
    List {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get a rate limit rule by ID
    Get {
        /// Rate limit rule ID
        #[arg(long)]
        id: i64,
    },
    /// Create a rate limit rule
    Create {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
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
        /// Rate limit rule ID
        #[arg(long)]
        id: i64,
        /// Rule name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a rate limit rule
    Delete {
        /// Rate limit rule ID
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ShieldAccessListAction {
    /// Get all access lists (managed + custom) for a Shield Zone
    List {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Get a custom access list by ID
    Get {
        /// Shield zone ID
        #[arg(long)]
        shield_zone_id: i64,
        /// Access list ID
        #[arg(long)]
        id: i64,
    },
    /// Create a custom access list
    Create {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
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
        /// Shield zone ID
        #[arg(long)]
        shield_zone_id: i64,
        /// Access list ID
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
        /// Shield zone ID
        #[arg(long)]
        shield_zone_id: i64,
        /// Access list ID
        #[arg(long)]
        id: i64,
    },
    /// Update access list configuration (enabled/action)
    UpdateConfig {
        /// Shield zone ID
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
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
        shield_zone_id: i64,
    },
    /// Update bot detection configuration
    Update {
        /// Shield zone ID
        #[arg(long = "id", alias = "shield-zone-id", value_name = "ID")]
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
        #[arg(long, conflicts_with = "all")]
        page: Option<i32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        per_page: Option<i32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get an edge script by ID
    Get {
        /// Script ID
        #[arg(long)]
        id: i64,
    },
    /// Create a new edge script
    Create {
        /// Display name for the edge script.
        #[arg(long)]
        name: String,
        /// Script type
        #[arg(long, value_enum)]
        script_type: ScriptTypeArg,
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
        /// Script ID
        #[arg(long)]
        id: i64,
        /// New name for the script
        #[arg(long)]
        name: Option<String>,
        /// Script type
        #[arg(long, value_enum)]
        script_type: Option<ScriptTypeArg>,
    },
    /// Delete an edge script
    Delete {
        /// Script ID
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
        /// Script ID
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
        /// Script ID
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
        /// Script ID
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ScriptCodeAction {
    /// Get the current draft source code
    Get {
        /// Script ID
        #[arg(long)]
        id: i64,
    },
    /// Update the draft source code
    Update {
        /// Script ID
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
        /// Script ID
        #[arg(long)]
        id: i64,
        /// Page number (1-based)
        #[arg(long, conflicts_with = "all")]
        page: Option<i32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        per_page: Option<i32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get the active (live) release for a script
    GetActive {
        /// Script ID
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ScriptVariableAction {
    /// List environment variables for a script
    List {
        /// Script ID
        #[arg(long)]
        id: i64,
    },
    /// Add an environment variable to a script
    Add {
        /// Script ID
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
        /// Script ID
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
        /// Script ID
        #[arg(long)]
        id: i64,
        /// Variable ID
        #[arg(long)]
        variable_id: i64,
    },
    /// Upsert (create or update by name) an environment variable
    Upsert {
        /// Script ID
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
        /// Script ID
        #[arg(long)]
        id: i64,
    },
    /// Add a secret to a script
    Add {
        /// Script ID
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
        /// Script ID
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
        /// Script ID
        #[arg(long)]
        id: i64,
        /// Secret ID
        #[arg(long)]
        secret_id: i64,
    },
    /// Upsert (create or update by name) a secret
    Upsert {
        /// Script ID
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

    /// Stream live syslog output from a Magic Containers application.
    ///
    /// NOTE: As of 2026-05-15 this command may fail at the log-forwarding-create
    /// step with an empty-body 400 from the bunny.net API. Tracking in
    /// backlog/log-forwarding-create-empty-400.md.
    ///
    /// How it works
    /// ============
    /// hoppy binds a local TCP syslog listener, exposes it via a tunnel so
    /// that Bunny's log-forwarding service can reach it, creates a temporary
    /// log-forwarding configuration for your application, then streams the
    /// incoming syslog messages to your terminal. On Ctrl-C (or any error)
    /// the forwarding configuration is deleted and the tunnel is torn down.
    ///
    /// Bunny delivers logs with a 10–30 s delay after the forwarding
    /// configuration is active — wait a moment before concluding that nothing
    /// is arriving.
    ///
    /// Tunnel options
    /// ==============
    /// • bore (default)   — shells out to the `bore` binary, which opens a
    ///   reverse tunnel through bore.pub.  Install: `cargo install bore-cli`
    ///   or `brew install bore-cli`.
    ///   WARNING: bore.pub is a third-party relay run by the bore project.
    ///   For sensitive logs prefer --tunnel-host or --tunnel none.
    ///
    /// • --tunnel none    — no tunnel is created; hoppy prints the local
    ///   address so you can configure your own ingress (VPN, public IP, etc.).
    ///
    /// • --tunnel-host <host:port>
    ///   You have already opened a tunnel (e.g. `ssh -R 5514:localhost:5514
    ///   user@vps.example.com`) and want hoppy to register that public
    ///   address with Bunny.  Pass the public endpoint as host:port.
    ///
    /// Examples
    /// ========
    ///   # Default — bore tunnel through bore.pub
    ///   hoppy container logs --id my-app-id
    ///
    ///   # No tunnel — use your own ingress
    ///   hoppy container logs --id my-app-id --tunnel none
    ///
    ///   # Pre-established SSH tunnel
    ///   ssh -R 5514:localhost:5514 user@vps.example.com &
    ///   hoppy container logs --id my-app-id --tunnel-host vps.example.com:5514
    Logs {
        /// Container app ID to stream logs from.
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
        app_id: String,
        /// Tunnel provider.  Use "bore" (default) to expose the local
        /// listener via bore.pub, or "none" if you have your own ingress.
        /// Conflicts with --tunnel-host.
        #[arg(long, default_value = "bore", value_parser = ["bore", "none"], conflicts_with = "tunnel_host")]
        tunnel: String,
        /// Static public endpoint for a pre-established tunnel (host:port).
        /// When set, hoppy registers this address with Bunny and listens
        /// locally.  Conflicts with --tunnel.
        /// Example: --tunnel-host vps.example.com:5514
        #[arg(long, conflicts_with = "tunnel")]
        tunnel_host: Option<String>,
        /// Local TCP port for the syslog listener.  0 = kernel-assigned
        /// (default).
        #[arg(long, default_value_t = 0)]
        local_port: u16,
        /// Overwrite an existing log-forwarding configuration and restore it
        /// on clean exit.
        #[arg(long)]
        replace_existing: bool,
        /// Accept the --follow flag for forward-compatibility (no-op; the
        /// command always follows).
        #[arg(long)]
        follow: bool,
        /// Bore relay server host.  Defaults to bore.pub.  Use this to
        /// point at a self-hosted bore server.
        #[arg(long)]
        bore_server: Option<String>,
    },

    /// Shortcut for `container app list` — mirrors `pull-zone list` etc.
    /// `app` is the canonical subcommand; this alias is provided for symmetry.
    List {
        /// Cursor for the next page
        #[arg(long, conflicts_with = "all")]
        cursor: Option<String>,
        /// Maximum number of results
        #[arg(long, conflicts_with = "all")]
        limit: Option<i32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Shortcut for `container app get`. `app` is the canonical subcommand.
    Get {
        /// Container app ID
        #[arg(long)]
        id: String,
    },
    /// Shortcut for `container app delete`. `app` is the canonical subcommand.
    Delete {
        /// Container app ID
        #[arg(long)]
        id: String,
        /// Also delete every auto-managed Pull Zone owned by this app.
        #[arg(long, conflicts_with = "no_cascade")]
        cascade: bool,
        /// Delete only the app; print orphan Pull Zone IDs for manual cleanup.
        #[arg(long)]
        no_cascade: bool,
    },
}

#[derive(Subcommand)]
pub enum ContainerAppAction {
    /// List all applications
    List {
        /// Cursor for the next page
        #[arg(long, conflicts_with = "all")]
        cursor: Option<String>,
        /// Maximum number of results
        #[arg(long, conflicts_with = "all")]
        limit: Option<i32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a specific application
    Get {
        /// Container app ID
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
        /// Container app ID
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
        /// Container app ID
        #[arg(long)]
        id: String,
    },
    /// Undeploy (suspend) an application
    Undeploy {
        /// Container app ID
        #[arg(long)]
        id: String,
    },
    /// Restart all pods in an application
    Restart {
        /// Container app ID
        #[arg(long)]
        id: String,
    },
    /// Delete an application.
    ///
    /// Refuses by default if the app has auto-managed Pull Zones (created
    /// for CDN endpoints) — pass `--cascade` to also delete them, or
    /// `--no-cascade` to delete only the app and print the orphan IDs.
    Delete {
        /// Container app ID
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
        /// Container app ID
        #[arg(long)]
        id: String,
    },
    /// Show statistics for an application
    Statistics {
        /// Container app ID
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
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
        app_id: String,
    },
    /// Update autoscaling settings for an application
    AutoscalingUpdate {
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
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
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
        app_id: String,
    },
    /// Update region settings for an application
    RegionSettingsUpdate {
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
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
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Container template ID
        #[arg(long)]
        container_id: String,
    },
    /// Add a container template to an application
    Add {
        /// Container app ID
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
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Container template ID
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
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Container template ID
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
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Container template ID
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
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
        app_id: String,
    },
    /// Add an endpoint to an application
    Add {
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Container template ID
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
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Endpoint ID
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
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Endpoint ID
        #[arg(long)]
        endpoint_id: String,
    },
}

#[derive(Subcommand)]
pub enum ContainerVolumeAction {
    /// List volumes for an application
    List {
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
        app_id: String,
    },
    /// Update a volume
    Update {
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Volume ID
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
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Volume ID
        #[arg(long)]
        volume_id: String,
    },
    /// Delete all instances of a volume
    Delete {
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Volume ID
        #[arg(long)]
        volume_id: String,
    },
    /// Delete a single volume instance
    DeleteInstance {
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Volume ID
        #[arg(long)]
        volume_id: String,
        /// Instance ID
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
        /// Registry ID
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
        /// Registry ID
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
        /// Registry ID
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
        #[arg(long, conflicts_with = "all")]
        cursor: Option<String>,
        /// Maximum number of results
        #[arg(long, conflicts_with = "all")]
        limit: Option<i32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get the optimal base region
    Optimal,
}

#[derive(Subcommand)]
pub enum ContainerNodeAction {
    /// List available nodes
    List {
        /// Cursor for the next page
        #[arg(long, conflicts_with = "all")]
        cursor: Option<String>,
        /// Maximum number of results
        #[arg(long, conflicts_with = "all")]
        limit: Option<i32>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
pub enum ContainerPodAction {
    /// Recreate a pod
    Recreate {
        /// Container app ID
        #[arg(long)]
        app_id: String,
        /// Pod ID
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
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
        app_id: String,
    },
    /// Create a log forwarding configuration
    Create {
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
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
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
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
        /// Container app ID
        #[arg(long = "id", alias = "app-id", value_name = "ID")]
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
        /// Database ID
        #[arg(long)]
        id: String,
        /// Start of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        from: String,
        /// End of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        to: String,
    },
    /// Get aggregated usage for a database (v2)
    Usage {
        /// Database ID
        #[arg(long)]
        id: String,
        /// Start of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        from: String,
        /// End of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
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
        /// Page number (1-based)
        #[arg(long, conflicts_with = "all")]
        page: Option<u32>,
        /// Items per page
        #[arg(long, conflicts_with = "all")]
        per_page: Option<u32>,
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
        /// Automatically paginate through all available pages
        #[arg(long)]
        all: bool,
    },
    /// Get a database (v2)
    Get {
        /// Database ID
        #[arg(long)]
        id: String,
    },
    /// Create a database (v2). NOTE: returns 500 upstream as of 2026-05-05.
    ///
    /// long_help: storage-region uses flat regions (eu-west-1, us-east-1).
    /// primary-region/replicas-region use compute codes (DE, FR, AMS, …).
    Create {
        /// Display name for the database.
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
        /// Database ID
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum DbGroupAction {
    /// List database groups
    List {
        /// Filter by name
        #[arg(long)]
        search: Option<String>,
    },
    /// Get a database group
    Get {
        /// Database group ID
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
        /// Database group ID
        #[arg(long)]
        id: String,
    },
    /// Get statistics for a database group
    Stats {
        /// Database group ID
        #[arg(long)]
        id: String,
        /// Start of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        from: String,
        /// End of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        to: String,
    },
    /// Get aggregated usage for a database group
    Usage {
        /// Database group ID
        #[arg(long)]
        id: String,
        /// Start of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        from: String,
        /// End of the time window (YYYY-MM-DD or YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        to: String,
    },
    /// Get live metrics for one or more groups
    Live {
        /// Database group IDs to fetch live metrics for (repeatable)
        #[arg(long = "id", value_name = "GROUP_ID")]
        ids: Vec<String>,
    },
    /// Generate a new auth token for a whole group
    GenerateKeys {
        /// Database group ID
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
        /// Database group ID
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
        /// Database ID (db_<ulid>)
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
        /// Database ID (db_<ulid>)
        #[arg(long)]
        db_id: String,
    },
    /// Mint a JWT for a database (v2)
    GenerateV2 {
        /// Database ID (db_<ulid>)
        #[arg(long)]
        db_id: String,
        /// Token scope: full-access or read-only
        #[arg(long, value_enum, default_value_t = TokenAuthorization::FullAccess)]
        authorization: TokenAuthorization,
        /// Optional expiry timestamp (RFC 3339)
        #[arg(long)]
        expires_at: Option<String>,
    },
    /// Revoke every auth token for a database (v2)
    RevokeV2 {
        /// Database ID (db_<ulid>)
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
    /// Get the optimal single-region recommendation (broken upstream — hidden)
    #[command(hide = true)]
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

impl From<TokenAuthorization> for bunny_net_api::database::types::Authorization {
    fn from(t: TokenAuthorization) -> Self {
        match t {
            TokenAuthorization::FullAccess => Self::FullAccess,
            TokenAuthorization::ReadOnly => Self::ReadOnly,
        }
    }
}

#[cfg(test)]
mod cli_parse_tests {
    use super::Cli;
    use clap::Parser;

    /// Regression test: `container logs` must parse without panicking.
    /// Previously crashed with "Mismatch between definition and access of 'format'"
    /// because the subcommand defined its own `--format` conflicting with the
    /// global flag.
    #[test]
    fn container_logs_parses_without_panic() {
        let result = Cli::try_parse_from(["hoppy", "container", "logs", "--app-id", "test-app-id"]);
        assert!(result.is_ok(), "CLI parse failed: {:?}", result.err());
    }

    #[test]
    fn version_string_full() {
        assert_eq!(
            super::format_version("0.3.0", "abc123def456", "2026-05-26"),
            "0.3.0 (abc123def456 2026-05-26)"
        );
    }

    #[test]
    fn version_string_bare_when_no_sha() {
        // Mirrors `CARGO_HOPPY_FORCE_NO_GIT=1`: build script emits empty
        // values, so `-V` falls back to the bare CARGO_PKG_VERSION.
        assert_eq!(super::format_version("0.3.0", "", ""), "0.3.0");
        assert_eq!(super::format_version("0.3.0", "", "2026-05-26"), "0.3.0");
    }

    #[test]
    fn version_string_dirty_suffix_preserved() {
        assert_eq!(
            super::format_version("0.3.0", "abc123def456+dirty", "2026-05-26"),
            "0.3.0 (abc123def456+dirty 2026-05-26)"
        );
    }

    #[test]
    fn no_hints_flag_parses() {
        let result = Cli::try_parse_from(["hoppy", "--no-hints", "pull-zone", "list"]);
        assert!(result.is_ok(), "CLI parse failed: {:?}", result.err());
        assert!(result.unwrap().no_hints);
    }

    #[test]
    fn looks_like_negative_number_basic() {
        assert!(super::looks_like_negative_number("-1"));
        assert!(super::looks_like_negative_number("-12"));
        assert!(super::looks_like_negative_number("-1.5"));
        assert!(super::looks_like_negative_number("-.5"));
        assert!(super::looks_like_negative_number("-1e3"));
        assert!(!super::looks_like_negative_number("-"));
        assert!(!super::looks_like_negative_number("-x"));
        assert!(!super::looks_like_negative_number("--min"));
        assert!(!super::looks_like_negative_number("1"));
        assert!(!super::looks_like_negative_number(""));
    }

    #[test]
    fn preceding_long_flag_finds_min() {
        let argv = vec![
            "hoppy".to_string(),
            "container".to_string(),
            "app".to_string(),
            "create".to_string(),
            "--min".to_string(),
            "-1".to_string(),
        ];
        assert_eq!(
            super::preceding_long_flag(&argv, "-1"),
            Some("min".to_string())
        );
    }

    #[test]
    fn preceding_long_flag_skips_short_flag() {
        let argv = vec!["hoppy".to_string(), "-x".to_string(), "-1".to_string()];
        assert_eq!(super::preceding_long_flag(&argv, "-1"), None);
    }

    #[test]
    fn rewrite_negative_value_error_targets_unknown_argument() {
        let result = Cli::try_parse_from([
            "hoppy",
            "container",
            "app",
            "create",
            "--name",
            "x",
            "--runtime-type",
            "shared",
            "--min",
            "-1",
            "--max",
            "1",
        ]);
        let err = match result {
            Ok(_) => panic!("expected parse error for --min -1"),
            Err(e) => e,
        };
        let argv: Vec<String> = vec![
            "hoppy".into(),
            "container".into(),
            "app".into(),
            "create".into(),
            "--name".into(),
            "x".into(),
            "--runtime-type".into(),
            "shared".into(),
            "--min".into(),
            "-1".into(),
            "--max".into(),
            "1".into(),
        ];
        let rendered = super::rewrite_negative_value_error(&err, &argv)
            .expect("expected a friendlier hint to be produced");
        assert!(rendered.contains("'-1'"), "rendered: {rendered}");
        assert!(rendered.contains("--min"), "rendered: {rendered}");
        assert!(rendered.contains("--min=-1"), "rendered: {rendered}");
    }

    #[test]
    fn rewrite_negative_value_error_eq_form_is_accepted() {
        // The `=` form sidesteps clap's short-flag parsing, so parse succeeds
        // and there is no error to rewrite. This locks the documented
        // workaround into the test suite.
        let result = Cli::try_parse_from([
            "hoppy",
            "container",
            "app",
            "create",
            "--name",
            "x",
            "--runtime-type",
            "shared",
            "--min=-1",
            "--max",
            "1",
        ]);
        // Parse succeeds even though -1 is out of the domain (we deliberately
        // do not validate range here — handler-level validation is out of
        // scope for this iteration).
        assert!(result.is_ok(), "parse should succeed for --min=-1");
    }

    #[test]
    fn container_logs_with_global_format_json() {
        let result = Cli::try_parse_from([
            "hoppy",
            "--format",
            "json",
            "container",
            "logs",
            "--app-id",
            "test-app-id",
        ]);
        assert!(result.is_ok(), "CLI parse failed: {:?}", result.err());
    }
}
