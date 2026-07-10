use crate::auth;
use crate::cli::{
    OutputFormat, StreamAction, StreamCaptionAction, StreamCollectionAction, StreamLibraryAction,
    StreamResolutionsAction, StreamVideoAction,
};
use crate::date;
use crate::output::{self, PaginatedListJson};
use crate::progress;
use crate::redact::{RedactConfig, placeholder, redact_secrets_in_json};
use anyhow::{Context as _, Result, bail};
use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::{CreateVideoLibrary, UpdateVideoLibrary, VideoLibrary};
use bunny_net_api::stream::types::{Collection, Video};
use bunny_net_api::stream::{
    CreateCollection, CreateVideo, EncoderOutputCodec, FetchVideo, SmartGenerateSettings,
    StreamCleanupResolutions, StreamClient, TranscribeSettings, UpdateCollection, UpdateVideo,
};
use std::io::{self, BufRead, Write};
use tokio_util::io::ReaderStream;

// ---------------------------------------------------------------------------
// Display structs — Video Libraries
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct VideoLibraryRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Videos")]
    video_count: i64,
    #[tabled(rename = "Storage")]
    storage_usage: i64,
    #[tabled(rename = "Created")]
    date_created: String,
}

impl From<&VideoLibrary> for VideoLibraryRow {
    fn from(l: &VideoLibrary) -> Self {
        Self {
            id: l.id,
            name: l.name.clone(),
            video_count: l.video_count,
            storage_usage: l.storage_usage,
            date_created: l.date_created.clone(),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct VideoLibraryDetail {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Videos")]
    video_count: i64,
    #[tabled(rename = "Traffic")]
    traffic_usage: i64,
    #[tabled(rename = "Storage")]
    storage_usage: i64,
    #[tabled(rename = "Pull Zone ID")]
    pull_zone_id: i64,
    #[tabled(rename = "Storage Zone ID")]
    storage_zone_id: i64,
    #[tabled(rename = "Resolutions")]
    enabled_resolutions: String,
    #[tabled(rename = "Watermark")]
    has_watermark: bool,
    #[tabled(rename = "MP4 Fallback")]
    enable_mp4_fallback: bool,
    #[tabled(rename = "Created")]
    date_created: String,
    /// Sensitive — redacted unless `--reveal` is set.
    #[serde(rename = "ApiKey")]
    #[tabled(rename = "API Key")]
    api_key: String,
    /// Sensitive — redacted unless `--reveal` is set.
    #[serde(rename = "ReadOnlyApiKey")]
    #[tabled(rename = "Read-Only API Key")]
    read_only_api_key: String,
}

impl VideoLibraryDetail {
    fn from_library(l: &VideoLibrary, redact_cfg: &RedactConfig) -> Self {
        let render_secret = |raw: &str| {
            if redact_cfg.reveal_field() {
                raw.to_owned()
            } else {
                placeholder(raw)
            }
        };
        Self {
            id: l.id,
            name: l.name.clone(),
            video_count: l.video_count,
            traffic_usage: l.traffic_usage,
            storage_usage: l.storage_usage,
            pull_zone_id: l.pull_zone_id,
            storage_zone_id: l.storage_zone_id,
            enabled_resolutions: l.enabled_resolutions.as_deref().unwrap_or("-").to_owned(),
            has_watermark: l.has_watermark,
            enable_mp4_fallback: l.enable_mp4_fallback,
            date_created: l.date_created.clone(),
            api_key: render_secret(&l.api_key),
            read_only_api_key: render_secret(&l.read_only_api_key),
        }
    }
}

// ---------------------------------------------------------------------------
// Display structs — Videos
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct VideoRow {
    #[tabled(rename = "GUID")]
    guid: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Length")]
    length: i32,
    #[tabled(rename = "Views")]
    views: i64,
    #[tabled(rename = "Storage")]
    storage_size: i64,
    #[tabled(rename = "Uploaded")]
    date_uploaded: String,
}

impl From<&Video> for VideoRow {
    fn from(v: &Video) -> Self {
        Self {
            guid: v.guid.clone(),
            title: v.title.clone(),
            status: v.status.to_string(),
            length: v.length,
            views: v.views,
            storage_size: v.storage_size,
            date_uploaded: v.date_uploaded.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display structs — Collections
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct CollectionRow {
    #[tabled(rename = "GUID")]
    guid: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Videos")]
    video_count: i64,
    #[tabled(rename = "Size")]
    total_size: i64,
}

impl From<&Collection> for CollectionRow {
    fn from(c: &Collection) -> Self {
        Self {
            guid: c.guid.as_deref().unwrap_or("-").to_owned(),
            name: c.name.as_deref().unwrap_or("-").to_owned(),
            video_count: c.video_count,
            total_size: c.total_size,
        }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &StreamAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    quiet: bool,
    record: Option<&str>,
    redact_cfg: &RedactConfig,
) -> Result<()> {
    match action {
        StreamAction::Library { action } => {
            handle_library(action, format, debug, yes, record, redact_cfg).await
        }
        StreamAction::Video { action } => {
            handle_video(action, format, debug, yes, quiet, record).await
        }
        StreamAction::Collection { action } => {
            handle_collection(action, format, debug, yes, record).await
        }
    }
}

async fn handle_library(
    action: &StreamLibraryAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    redact_cfg: &RedactConfig,
) -> Result<()> {
    let core = auth::core_client(debug, record)?;

    match action {
        StreamLibraryAction::List {
            search,
            page,
            per_page,
            all,
        } => {
            if *all {
                const AUTO_PER_PAGE: u32 = 1000;
                let mut current_page: u32 = 1;
                let mut accumulated: Vec<VideoLibrary> = Vec::new();
                loop {
                    let result = core
                        .list_video_libraries(
                            Some(current_page),
                            Some(AUTO_PER_PAGE),
                            search.as_deref(),
                        )
                        .await?;
                    let has_more = result.has_more_items;
                    if let OutputFormat::Json = format {
                        accumulated.extend(result.items);
                    } else {
                        let rows: Vec<VideoLibraryRow> =
                            result.items.iter().map(VideoLibraryRow::from).collect();
                        output::print_data(&rows, format);
                    }
                    if !has_more {
                        break;
                    }
                    current_page += 1;
                }
                if let OutputFormat::Json = format {
                    let total = accumulated.len() as i64;
                    let envelope = PaginatedListJson {
                        items: &accumulated,
                        current_page: current_page as i64,
                        total_items: total,
                        has_more_items: false,
                    };
                    let mut value = serde_json::to_value(&envelope)
                        .context("failed to serialize video library list to JSON")?;
                    redact_secrets_in_json(&mut value, redact_cfg);
                    let json = serde_json::to_string_pretty(&value)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                }
            } else {
                let result = core
                    .list_video_libraries(*page, *per_page, search.as_deref())
                    .await?;
                if let OutputFormat::Json = format {
                    let envelope = PaginatedListJson {
                        items: &result.items,
                        current_page: result.current_page,
                        total_items: result.total_items,
                        has_more_items: result.has_more_items,
                    };
                    let mut value = serde_json::to_value(&envelope)
                        .context("failed to serialize video library list to JSON")?;
                    redact_secrets_in_json(&mut value, redact_cfg);
                    let json = serde_json::to_string_pretty(&value)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                } else {
                    let rows: Vec<VideoLibraryRow> =
                        result.items.iter().map(VideoLibraryRow::from).collect();
                    output::print_data(&rows, format);
                    if let Some(first) = result.items.first() {
                        output::hints::tip(&format!(
                            "hoppy stream video list --library-id {}",
                            first.id
                        ));
                    }
                }
            }
        }
        StreamLibraryAction::Get { id } => {
            let lib = core.get_video_library(*id).await?;
            print_library(&lib, format, redact_cfg);
        }
        StreamLibraryAction::Create { name } => {
            let body = CreateVideoLibrary::new(name);
            let lib = core.create_video_library(&body).await?;
            print_library(&lib, format, redact_cfg);
        }
        StreamLibraryAction::Update {
            id,
            name,
            allow_direct_play,
            enable_mp4_fallback,
            has_watermark,
        } => {
            if name.is_none()
                && allow_direct_play.is_none()
                && enable_mp4_fallback.is_none()
                && has_watermark.is_none()
            {
                bail!(
                    "at least one update flag is required (--name, --allow-direct-play, --enable-mp4-fallback, or --has-watermark)"
                );
            }
            let mut body = UpdateVideoLibrary::new();
            if let Some(v) = name {
                body = body.name(v);
            }
            if let Some(v) = allow_direct_play {
                body = body.allow_direct_play(*v);
            }
            if let Some(v) = enable_mp4_fallback {
                body = body.enable_mp4_fallback(*v);
            }
            if let Some(v) = has_watermark {
                body = body.has_watermark(*v);
            }
            let lib = core.update_video_library(*id, &body).await?;
            print_library(&lib, format, redact_cfg);
        }
        StreamLibraryAction::Delete { id } => {
            if !yes {
                eprint!("Delete video library {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            core.delete_video_library(*id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "stream-library",
                serde_json::json!({}),
                &format!("Deleted video library {id}"),
            );
        }
        StreamLibraryAction::ResetApiKey { id } => {
            reset_library_key(&core, format, yes, redact_cfg, *id, false).await?;
        }
        StreamLibraryAction::ResetReadOnlyApiKey { id } => {
            reset_library_key(&core, format, yes, redact_cfg, *id, true).await?;
        }
        StreamLibraryAction::Statistics {
            id,
            date_from,
            date_to,
            hourly,
            video_guid,
        } => {
            let date_from = date::normalise_datetime_opt(date_from.as_deref())?;
            let date_to = date::normalise_datetime_opt(date_to.as_deref())?;
            let stream = resolve_stream_client(*id, debug, record).await?;
            let stats = stream
                .get_library_statistics(
                    *id,
                    date_from.as_deref(),
                    date_to.as_deref(),
                    *hourly,
                    video_guid.as_deref(),
                )
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&stats).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                #[derive(serde::Serialize, tabled::Tabled)]
                struct Row {
                    #[tabled(rename = "Metric")]
                    metric: String,
                    #[tabled(rename = "Value")]
                    value: String,
                }
                // API returns -1 as a sentinel meaning "no data available".
                let engagement_display = if stats.engagement_score == -1 {
                    "N/A".to_owned()
                } else {
                    stats.engagement_score.to_string()
                };
                let rows = vec![Row {
                    metric: "Engagement Score".to_owned(),
                    value: engagement_display,
                }];
                output::print_data(&rows, format);
            }
        }
    }
    Ok(())
}

async fn resolve_stream_client(
    library_id: i64,
    debug: bool,
    record: Option<&str>,
) -> Result<StreamClient> {
    if let Some(key) = auth::get_stream_key() {
        let mut client = StreamClient::new(key);
        client = if let Some(url) = auth::get_stream_url() {
            client.with_base_url(url)
        } else {
            client
        };
        client = client.with_debug(debug);
        if let Some(dir) = auth::get_record_dir(record) {
            client = client.with_record(dir);
        }
        return Ok(client);
    }
    let core = auth::core_client(debug, record)?;
    let lib = core.get_video_library(library_id).await?;
    if lib.api_key.is_empty() {
        bail!("could not determine stream API key for library {library_id}; set BUNNY_STREAM_KEY");
    }
    let mut client = StreamClient::new(&lib.api_key);
    client = if let Some(url) = auth::get_stream_url() {
        client.with_base_url(url)
    } else {
        client
    };
    client = client.with_debug(debug);
    if let Some(dir) = auth::get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

async fn handle_video(
    action: &StreamVideoAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    quiet: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        StreamVideoAction::List {
            library_id,
            page,
            items_per_page,
            search,
            collection,
            order_by,
            all,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            if *all {
                const AUTO_PER_PAGE: u32 = 1000;
                let mut current_page: u32 = 1;
                let mut accumulated: Vec<Video> = Vec::new();
                loop {
                    let result = stream
                        .list_videos(
                            *library_id,
                            Some(current_page),
                            Some(AUTO_PER_PAGE),
                            search.as_deref(),
                            collection.as_deref(),
                            order_by.as_deref(),
                        )
                        .await?;
                    let more = has_more_items(&result);
                    if let OutputFormat::Json = format {
                        accumulated.extend(result.items);
                    } else {
                        let rows: Vec<VideoRow> = result.items.iter().map(VideoRow::from).collect();
                        output::print_data(&rows, format);
                    }
                    if !more {
                        break;
                    }
                    current_page += 1;
                }
                if let OutputFormat::Json = format {
                    let total = accumulated.len() as i64;
                    let envelope = PaginatedListJson {
                        items: &accumulated,
                        current_page: current_page as i64,
                        total_items: total,
                        has_more_items: false,
                    };
                    let json = serde_json::to_string_pretty(&envelope)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                }
            } else {
                let result = stream
                    .list_videos(
                        *library_id,
                        *page,
                        *items_per_page,
                        search.as_deref(),
                        collection.as_deref(),
                        order_by.as_deref(),
                    )
                    .await?;
                if let OutputFormat::Json = format {
                    let envelope = PaginatedListJson {
                        items: &result.items,
                        current_page: result.current_page,
                        total_items: result.total_items,
                        has_more_items: has_more_items(&result),
                    };
                    let json = serde_json::to_string_pretty(&envelope)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                } else {
                    let rows: Vec<VideoRow> = result.items.iter().map(VideoRow::from).collect();
                    output::print_data(&rows, format);
                }
            }
        }
        StreamVideoAction::Get {
            library_id,
            video_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let video = stream.get_video(*library_id, video_id).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&video).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let row = VideoRow::from(&video);
                output::print_single(&row, format);
            }
        }
        StreamVideoAction::Upload {
            library_id,
            file,
            title,
            collection_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let video_title = title.as_deref().unwrap_or_else(|| {
                std::path::Path::new(file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file.as_str())
            });
            let mut create_body = CreateVideo::new(video_title);
            if let Some(cid) = collection_id {
                create_body = create_body.collection_id(cid);
            }
            let video = stream.create_video(*library_id, &create_body).await?;

            // Open the file and get its size for the progress bar.
            let fh = tokio::fs::File::open(file)
                .await
                .with_context(|| format!("opening file: {file}"))?;
            let file_size = fh
                .metadata()
                .await
                .with_context(|| format!("reading metadata for: {file}"))?
                .len();

            let pb = progress::file_progress(file_size, quiet);

            let body: reqwest::Body = if let Some(bar) = &pb {
                reqwest::Body::wrap_stream(ReaderStream::new(bar.wrap_async_read(fh)))
            } else {
                reqwest::Body::wrap_stream(ReaderStream::new(fh))
            };

            stream.upload_video(*library_id, &video.guid, body).await?;

            progress::finish_with_message(
                pb.as_ref(),
                format!("Uploaded {} ({})", video.guid, video.title),
            );
            if pb.is_none() && !quiet {
                output::print_mutation_result(
                    format,
                    "upload",
                    "stream-video",
                    serde_json::json!({ "Guid": video.guid }),
                    &format!("Uploaded video {} ({})", video.guid, video.title),
                );
            }

            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&video).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let row = VideoRow::from(&video);
                output::print_single(&row, format);
            }
        }
        StreamVideoAction::Update {
            library_id,
            video_id,
            title,
            collection_id,
        } => {
            if title.is_none() && collection_id.is_none() {
                bail!("at least one update flag is required (--title or --collection-id)");
            }
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let mut body = UpdateVideo::new();
            if let Some(t) = title {
                body = body.title(t);
            }
            if let Some(cid) = collection_id {
                body = body.collection_id(cid);
            }
            stream.update_video(*library_id, video_id, &body).await?;
            let video = stream.get_video(*library_id, video_id).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&video).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let row = VideoRow::from(&video);
                output::print_single(&row, format);
            }
        }
        StreamVideoAction::Fetch {
            library_id,
            url,
            title,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let mut body = FetchVideo::new(url);
            if let Some(t) = title {
                body = body.title(t);
            }
            let status = stream.fetch_video(*library_id, &body).await?;
            if status.success {
                // Redact query string from URL to avoid leaking signed tokens or credentials.
                let safe_url = url.split('?').next().unwrap_or(url);
                eprintln!("Fetch initiated from {safe_url}");
                eprintln!("The video will appear in the library once processing completes.");
                eprintln!("Check status with: hoppy stream video list --library-id {library_id}");
            } else {
                bail!(
                    "fetch failed: {}",
                    status.message.as_deref().unwrap_or("unknown error")
                );
            }
        }
        StreamVideoAction::Delete {
            library_id,
            video_id,
        } => {
            if !yes {
                eprint!("Delete video {video_id} from library {library_id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            stream.delete_video(*library_id, video_id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "stream-video",
                serde_json::json!({}),
                &format!("Deleted video {video_id} from library {library_id}"),
            );
        }
        StreamVideoAction::Caption { action } => {
            handle_caption(action, format, debug, record).await?;
        }
        StreamVideoAction::Transcribe {
            library_id,
            video_id,
            force,
            language,
            target_languages,
            generate_title,
            generate_description,
            generate_chapters,
            generate_moments,
        } => {
            let mut any = false;
            let mut settings = TranscribeSettings::new();
            if let Some(lang) = language {
                settings = settings.source_language(lang);
                any = true;
            }
            if !target_languages.is_empty() {
                settings = settings.target_languages(target_languages.iter().map(|s| s.as_str()));
                any = true;
            }
            if *generate_title {
                settings = settings.generate_title(true);
                any = true;
            }
            if *generate_description {
                settings = settings.generate_description(true);
                any = true;
            }
            if *generate_chapters {
                settings = settings.generate_chapters(true);
                any = true;
            }
            if *generate_moments {
                settings = settings.generate_moments(true);
                any = true;
            }
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let status = stream
                .transcribe_video(
                    *library_id,
                    video_id,
                    *force,
                    if any { Some(&settings) } else { None },
                )
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&status).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                eprintln!(
                    "Triggered transcription for {video_id}: {}",
                    status.message.as_deref().unwrap_or("queued")
                );
            }
        }
        StreamVideoAction::Heatmap {
            library_id,
            video_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let heatmap = stream.get_video_heatmap(*library_id, video_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&heatmap)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                match &heatmap.heatmap {
                    None => eprintln!("No heatmap data"),
                    Some(map) if map.is_empty() => eprintln!("No heatmap data"),
                    Some(map) => {
                        let mut segments: Vec<_> = map.iter().collect();
                        segments.sort_by_key(|(k, _)| k.parse::<i64>().unwrap_or(i64::MAX));
                        #[derive(serde::Serialize, tabled::Tabled)]
                        struct HeatmapRow {
                            #[tabled(rename = "Segment")]
                            segment: String,
                            #[tabled(rename = "Intensity")]
                            intensity: i32,
                        }
                        let rows: Vec<HeatmapRow> = segments
                            .into_iter()
                            .map(|(k, v)| HeatmapRow {
                                segment: k.clone(),
                                intensity: *v,
                            })
                            .collect();
                        output::print_data(&rows, format);
                    }
                }
            }
        }
        StreamVideoAction::Reencode {
            library_id,
            video_id,
            codec,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let video = if let Some(codec_str) = codec {
                let c = EncoderOutputCodec::parse(codec_str).ok_or_else(|| {
                    anyhow::anyhow!(
                        "invalid codec {:?}; expected one of: x264, vp9, hevc, av1",
                        codec_str
                    )
                })?;
                stream
                    .reencode_video_using_codec(*library_id, video_id, c)
                    .await?
            } else {
                stream.reencode_video(*library_id, video_id).await?
            };
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&video).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let row = VideoRow::from(&video);
                output::print_single(&row, format);
            }
        }
        StreamVideoAction::Repackage {
            library_id,
            video_id,
            discard_originals,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let video = stream
                .repackage_video(*library_id, video_id, !discard_originals)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&video).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let row = VideoRow::from(&video);
                output::print_single(&row, format);
            }
        }
        StreamVideoAction::SmartGenerate {
            library_id,
            video_id,
            language,
            generate_title,
            generate_description,
            generate_chapters,
            generate_moments,
        } => {
            let mut settings = SmartGenerateSettings::new();
            if let Some(lang) = language {
                settings = settings.source_language(lang);
            }
            if *generate_title {
                settings = settings.generate_title(true);
            }
            if *generate_description {
                settings = settings.generate_description(true);
            }
            if *generate_chapters {
                settings = settings.generate_chapters(true);
            }
            if *generate_moments {
                settings = settings.generate_moments(true);
            }
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let status = stream
                .smart_generate(*library_id, video_id, &settings)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&status).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                eprintln!(
                    "Triggered smart-generate for {video_id}: {}",
                    status.message.as_deref().unwrap_or("queued")
                );
            }
        }
        StreamVideoAction::SetThumbnail {
            library_id,
            video_id,
            thumbnail_url,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let status = stream
                .set_video_thumbnail(*library_id, video_id, Some(thumbnail_url))
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&status).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                eprintln!(
                    "Set thumbnail for {video_id}: {}",
                    status.message.as_deref().unwrap_or("ok")
                );
            }
        }
        StreamVideoAction::Resolutions { action } => {
            handle_resolutions(action, format, debug, yes, record).await?;
        }
        StreamVideoAction::Storage {
            library_id,
            video_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let envelope = stream.get_video_storage_size(*library_id, video_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&envelope)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                #[derive(serde::Serialize, tabled::Tabled)]
                struct StorageRow {
                    #[tabled(rename = "Category")]
                    category: String,
                    #[tabled(rename = "Bytes")]
                    bytes: i64,
                }
                if let Some(data) = &envelope.data {
                    let mut rows = vec![
                        StorageRow {
                            category: "Originals".to_owned(),
                            bytes: data.originals,
                        },
                        StorageRow {
                            category: "Thumbnails".to_owned(),
                            bytes: data.thumbnails,
                        },
                        StorageRow {
                            category: "Previews".to_owned(),
                            bytes: data.previews,
                        },
                        StorageRow {
                            category: "MP4 Fallback".to_owned(),
                            bytes: data.mp4_fallback,
                        },
                        StorageRow {
                            category: "Miscellaneous".to_owned(),
                            bytes: data.miscellaneous,
                        },
                    ];
                    if let Some(encoded) = &data.encoded {
                        let mut renditions: Vec<_> = encoded.iter().collect();
                        renditions.sort_by_key(|(k, _)| k.as_str());
                        for (key, r) in renditions {
                            rows.push(StorageRow {
                                category: format!("Encoded/{key}"),
                                bytes: r.size,
                            });
                        }
                    }
                    output::print_data(&rows, format);
                } else {
                    eprintln!("No storage data available");
                }
            }
        }
    }
    Ok(())
}

async fn handle_resolutions(
    action: &StreamResolutionsAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        StreamResolutionsAction::List {
            library_id,
            video_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let envelope = stream.get_video_resolutions(*library_id, video_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&envelope)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &envelope.data {
                #[derive(serde::Serialize, tabled::Tabled)]
                struct ResRow {
                    #[tabled(rename = "Type")]
                    res_type: String,
                    #[tabled(rename = "Resolutions")]
                    resolutions: String,
                }
                let rows = vec![
                    ResRow {
                        res_type: "Available".to_owned(),
                        resolutions: data.available_resolutions.join(", "),
                    },
                    ResRow {
                        res_type: "Configured".to_owned(),
                        resolutions: data.configured_resolutions.join(", "),
                    },
                    ResRow {
                        res_type: "Playlist".to_owned(),
                        resolutions: data
                            .playlist_resolutions
                            .iter()
                            .filter_map(|r| r.resolution.as_deref())
                            .collect::<Vec<_>>()
                            .join(", "),
                    },
                ];
                output::print_data(&rows, format);
            } else {
                eprintln!("No resolution data available");
            }
        }
        StreamResolutionsAction::Cleanup {
            library_id,
            video_id,
            resolutions,
            delete_non_configured,
            delete_original,
            delete_mp4_files,
            dry_run,
        } => {
            if resolutions.is_none()
                && !*delete_non_configured
                && !*delete_original
                && !*delete_mp4_files
            {
                bail!(
                    "specify what to delete: --resolutions, --delete-non-configured, --delete-original, or --delete-mp4-files (combine with --dry-run to preview)"
                );
            }
            if !*dry_run && !yes {
                eprint!(
                    "Cleanup resolutions for video {video_id}? This permanently deletes files. [y/N] "
                );
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let opts = StreamCleanupResolutions {
                resolutions_to_delete: resolutions.as_deref(),
                delete_non_configured_resolutions: *delete_non_configured,
                delete_original: *delete_original,
                delete_mp4_files: *delete_mp4_files,
                dry_run: *dry_run,
            };
            let status = stream
                .cleanup_video_resolutions(*library_id, video_id, &opts)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&status).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let prefix = if *dry_run { "[dry-run] " } else { "" };
                eprintln!(
                    "{prefix}Cleanup for {video_id}: {}",
                    status.message.as_deref().unwrap_or("done")
                );
            }
        }
    }
    Ok(())
}

async fn handle_caption(
    action: &StreamCaptionAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        StreamCaptionAction::Add {
            library_id,
            video_id,
            srclang,
            file,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let content = std::fs::read_to_string(file)
                .with_context(|| format!("failed to read caption file: {file}"))?;
            let result = stream
                .add_caption(*library_id, video_id, srclang, &content)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                output::print_mutation_result(
                    format,
                    "add",
                    "stream-caption",
                    serde_json::json!({}),
                    &format!("Added {srclang} captions to video {video_id}"),
                );
            }
        }
        StreamCaptionAction::Delete {
            library_id,
            video_id,
            srclang,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            stream
                .delete_caption(*library_id, video_id, srclang)
                .await?;
            output::print_mutation_result(
                format,
                "delete",
                "stream-caption",
                serde_json::json!({}),
                &format!("Deleted {srclang} captions from video {video_id}"),
            );
        }
    }
    Ok(())
}

async fn handle_collection(
    action: &StreamCollectionAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        StreamCollectionAction::List {
            library_id,
            page,
            items_per_page,
            search,
            order_by,
            all,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            if *all {
                const AUTO_PER_PAGE: u32 = 1000;
                let mut current_page: u32 = 1;
                let mut accumulated: Vec<Collection> = Vec::new();
                loop {
                    let result = stream
                        .list_collections(
                            *library_id,
                            Some(current_page),
                            Some(AUTO_PER_PAGE),
                            search.as_deref(),
                            order_by.as_deref(),
                        )
                        .await?;
                    let more = has_more_items(&result);
                    if let OutputFormat::Json = format {
                        accumulated.extend(result.items);
                    } else {
                        let rows: Vec<CollectionRow> =
                            result.items.iter().map(CollectionRow::from).collect();
                        output::print_data(&rows, format);
                    }
                    if !more {
                        break;
                    }
                    current_page += 1;
                }
                if let OutputFormat::Json = format {
                    let total = accumulated.len() as i64;
                    let envelope = PaginatedListJson {
                        items: &accumulated,
                        current_page: current_page as i64,
                        total_items: total,
                        has_more_items: false,
                    };
                    let json = serde_json::to_string_pretty(&envelope)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                }
            } else {
                let result = stream
                    .list_collections(
                        *library_id,
                        *page,
                        *items_per_page,
                        search.as_deref(),
                        order_by.as_deref(),
                    )
                    .await?;
                if let OutputFormat::Json = format {
                    let envelope = PaginatedListJson {
                        items: &result.items,
                        current_page: result.current_page,
                        total_items: result.total_items,
                        has_more_items: has_more_items(&result),
                    };
                    let json = serde_json::to_string_pretty(&envelope)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                } else {
                    let rows: Vec<CollectionRow> =
                        result.items.iter().map(CollectionRow::from).collect();
                    output::print_data(&rows, format);
                }
            }
        }
        StreamCollectionAction::Get {
            library_id,
            collection_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let collection = stream.get_collection(*library_id, collection_id).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&collection).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let row = CollectionRow::from(&collection);
                output::print_single(&row, format);
            }
        }
        StreamCollectionAction::Create { library_id, name } => {
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let body = CreateCollection::new(name);
            let collection = stream.create_collection(*library_id, &body).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&collection).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let row = CollectionRow::from(&collection);
                output::print_single(&row, format);
            }
        }
        StreamCollectionAction::Update {
            library_id,
            collection_id,
            name,
        } => {
            if name.is_none() {
                bail!("at least one update flag is required (--name)");
            }
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            let mut body = UpdateCollection::new();
            if let Some(n) = name {
                body = body.name(n);
            }
            let collection = stream
                .update_collection(*library_id, collection_id, &body)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&collection).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let row = CollectionRow::from(&collection);
                output::print_single(&row, format);
            }
        }
        StreamCollectionAction::Delete {
            library_id,
            collection_id,
        } => {
            if !yes {
                eprint!("Delete collection {collection_id} from library {library_id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let stream = resolve_stream_client(*library_id, debug, record).await?;
            stream.delete_collection(*library_id, collection_id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "stream-collection",
                serde_json::json!({}),
                &format!("Deleted collection {collection_id} from library {library_id}"),
            );
        }
    }
    Ok(())
}

/// Whether a `PaginatedList` has more pages after the current one.
fn has_more_items<T>(list: &bunny_net_api::stream::PaginatedList<T>) -> bool {
    list.current_page.saturating_mul(list.items_per_page as i64) < list.total_items
}

/// Render a single video library.
///
/// `ApiKey` and `ReadOnlyApiKey` are sensitive — they grant full
/// (resp. read-only) access to the library. They are redacted by default
/// across every output format; `--reveal` opts in to the raw values.
fn print_library(lib: &VideoLibrary, format: OutputFormat, redact_cfg: &RedactConfig) {
    if let OutputFormat::Json = format {
        let mut value =
            serde_json::to_value(lib).expect("failed to serialize video library to JSON");
        redact_secrets_in_json(&mut value, redact_cfg);
        let json = serde_json::to_string_pretty(&value).expect("failed to serialize to JSON");
        println!("{json}");
    } else {
        let detail = VideoLibraryDetail::from_library(lib, redact_cfg);
        output::print_single(&detail, format);
    }
}

/// Confirm, rotate a video-library API key, then re-fetch and display the library.
///
/// The reset endpoints return `204 No Content` — the new key is never echoed by
/// the API, so we re-fetch the library to surface it. The key is redacted unless
/// the user passed the global `--reveal` flag.
async fn reset_library_key(
    core: &CoreClient,
    format: OutputFormat,
    yes: bool,
    redact_cfg: &RedactConfig,
    id: i64,
    read_only: bool,
) -> Result<()> {
    let label = if read_only {
        "read-only API key"
    } else {
        "API key"
    };
    if !yes {
        eprint!(
            "Rotate the {label} for video library {id}? This invalidates the current key. [y/N] "
        );
        io::stderr().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        let answer = line.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            eprintln!("Aborted.");
            return Ok(());
        }
    }
    if read_only {
        core.reset_video_library_read_only_api_key(id).await?;
    } else {
        core.reset_video_library_api_key(id).await?;
    }
    // Re-fetch so the freshly-generated key can be shown.
    let lib = core.get_video_library(id).await.with_context(|| {
        format!(
            "{label} for video library {id} was rotated but the credential re-fetch failed — \
             run `hoppy stream library get --id {id}` to retrieve it"
        )
    })?;
    if !matches!(format, OutputFormat::Json) {
        eprintln!("Rotated {label} for video library {id}.");
    }
    print_library(&lib, format, redact_cfg);
    Ok(())
}
