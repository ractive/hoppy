//! Progress bar helpers for upload and download operations.
//!
//! Progress bars are only shown when stderr is a TTY and `quiet` is false.
//! When either condition is not met all operations are no-ops, so call sites
//! do not need to special-case anything.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Returns `true` when progress bars should be displayed.
///
/// Conditions: stderr must be a TTY and the caller must not have requested
/// quiet mode.
fn should_show(quiet: bool) -> bool {
    !quiet && console::user_attended_stderr()
}

/// Create a determinate progress bar for a file transfer of known size.
///
/// The bar tracks bytes transferred and shows a human-readable rate and ETA.
/// Returns `None` when quiet or not a TTY.
pub fn file_progress(file_size: u64, quiet: bool) -> Option<ProgressBar> {
    if !should_show(quiet) {
        return None;
    }

    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    Some(pb)
}

/// Create an indeterminate spinner for operations where the total size is
/// unknown (e.g. a download before the `Content-Length` header is read).
///
/// Returns `None` when quiet or not a TTY.
pub fn spinner(message: impl Into<String>, quiet: bool) -> Option<ProgressBar> {
    if !should_show(quiet) {
        return None;
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.into());
    pb.enable_steady_tick(Duration::from_millis(120));
    Some(pb)
}

/// Finish a progress bar with a success message, or do nothing if `None`.
pub fn finish_with_message(pb: Option<&ProgressBar>, message: impl Into<String>) {
    if let Some(pb) = pb {
        pb.finish_with_message(message.into());
    }
}
