pub mod cmd;
pub mod server;

/// Skip the current test when running in live mode.
/// Mock-based tests call this at the top to avoid hitting real APIs.
macro_rules! skip_in_live_mode {
    () => {
        if $crate::e2e_support::cmd::is_live_mode() {
            eprintln!("Skipping mock-based test in live mode");
            return;
        }
    };
}
pub(crate) use skip_in_live_mode;
