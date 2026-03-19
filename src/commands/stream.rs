use crate::auth;
use crate::cli::{
    OutputFormat, StreamAction, StreamCollectionAction, StreamLibraryAction, StreamVideoAction,
};
use crate::output::{self, PaginatedListJson};
use crate::progress;
use anyhow::{Context as _, Result, bail};
use bunny_api_core::CoreClient;
use bunny_api_core::types::{CreateVideoLibrary, UpdateVideoLibrary, VideoLibrary};
use bunny_api_stream::types::{Collection, Video};
use bunny_api_stream::{
    CreateCollection, CreateVideo, FetchVideo, StreamClient, UpdateCollection, UpdateVideo,
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
}

impl From<&VideoLibrary> for VideoLibraryDetail {
    fn from(l: &VideoLibrary) -> Self {
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
) -> Result<()> {
    match action {
        StreamAction::Library { action } => handle_library(action, format, debug, yes).await,
        StreamAction::Video { action } => handle_video(action, format, debug, yes, quiet).await,
        StreamAction::Collection { action } => handle_collection(action, format, debug, yes).await,
    }
}

async fn handle_library(
    action: &StreamLibraryAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
) -> Result<()> {
    let core = CoreClient::new(auth::get_api_key()?).with_debug(debug);

    match action {
        StreamLibraryAction::List {
            search,
            page,
            per_page,
        } => {
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
                let json =
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<VideoLibraryRow> =
                    result.items.iter().map(VideoLibraryRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        StreamLibraryAction::Get { id } => {
            let lib = core.get_video_library(*id).await?;
            print_library(&lib, format);
        }
        StreamLibraryAction::Create { name } => {
            let body = CreateVideoLibrary::new(name);
            let lib = core.create_video_library(&body).await?;
            print_library(&lib, format);
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
            print_library(&lib, format);
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
            eprintln!("Deleted video library {id}");
        }
    }
    Ok(())
}

async fn resolve_stream_client(library_id: i64, debug: bool) -> Result<StreamClient> {
    if let Some(key) = auth::get_stream_key() {
        return Ok(StreamClient::new(key).with_debug(debug));
    }
    let core = CoreClient::new(auth::get_api_key()?).with_debug(debug);
    let lib = core.get_video_library(library_id).await?;
    if lib.api_key.is_empty() {
        bail!("could not determine stream API key for library {library_id}; set BUNNY_STREAM_KEY");
    }
    Ok(StreamClient::new(&lib.api_key).with_debug(debug))
}

async fn handle_video(
    action: &StreamVideoAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    quiet: bool,
) -> Result<()> {
    match action {
        StreamVideoAction::List {
            library_id,
            page,
            items_per_page,
            search,
            collection,
            order_by,
        } => {
            let stream = resolve_stream_client(*library_id, debug).await?;
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
                    has_more_items: (result.current_page * result.items_per_page as i64)
                        < result.total_items,
                };
                let json =
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<VideoRow> = result.items.iter().map(VideoRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        StreamVideoAction::Get {
            library_id,
            video_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug).await?;
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
            let stream = resolve_stream_client(*library_id, debug).await?;
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
                eprintln!("Uploaded video {} ({})", video.guid, video.title);
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
            let stream = resolve_stream_client(*library_id, debug).await?;
            let mut body = UpdateVideo::new();
            if let Some(t) = title {
                body = body.title(t);
            }
            if let Some(cid) = collection_id {
                body = body.collection_id(cid);
            }
            let status = stream.update_video(*library_id, video_id, &body).await?;
            eprintln!(
                "Updated video {video_id}: {}",
                status.message.as_deref().unwrap_or("ok")
            );
        }
        StreamVideoAction::Fetch {
            library_id,
            url,
            title,
        } => {
            let stream = resolve_stream_client(*library_id, debug).await?;
            let mut body = FetchVideo::new(url);
            if let Some(t) = title {
                body = body.title(t);
            }
            eprintln!("Fetching video from {url}");
            let status = stream.fetch_video(*library_id, &body).await?;
            eprintln!("{}", status.message.as_deref().unwrap_or("ok"));
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
            let stream = resolve_stream_client(*library_id, debug).await?;
            stream.delete_video(*library_id, video_id).await?;
            eprintln!("Deleted video {video_id} from library {library_id}");
        }
    }
    Ok(())
}

async fn handle_collection(
    action: &StreamCollectionAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
) -> Result<()> {
    match action {
        StreamCollectionAction::List {
            library_id,
            page,
            items_per_page,
            search,
            order_by,
        } => {
            let stream = resolve_stream_client(*library_id, debug).await?;
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
                    has_more_items: (result.current_page * result.items_per_page as i64)
                        < result.total_items,
                };
                let json =
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<CollectionRow> =
                    result.items.iter().map(CollectionRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        StreamCollectionAction::Get {
            library_id,
            collection_id,
        } => {
            let stream = resolve_stream_client(*library_id, debug).await?;
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
            let stream = resolve_stream_client(*library_id, debug).await?;
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
            let stream = resolve_stream_client(*library_id, debug).await?;
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
            let stream = resolve_stream_client(*library_id, debug).await?;
            stream.delete_collection(*library_id, collection_id).await?;
            eprintln!("Deleted collection {collection_id} from library {library_id}");
        }
    }
    Ok(())
}

fn print_library(lib: &VideoLibrary, format: OutputFormat) {
    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(lib).expect("failed to serialize to JSON");
        println!("{json}");
    } else {
        let detail = VideoLibraryDetail::from(lib);
        output::print_single(&detail, format);
    }
}
