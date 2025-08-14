//! Example: a small error reporter that sets a better error class
//! and trims noisy runtime frames (tokio/backtrace) using
//! NotifyBuilder::methods_to_ignore and the crate's improved trimming.
//!
//! Run with:
//!   cargo run --example error_reporter

extern crate bugsnag;
#[macro_use]
extern crate lazy_static;

// Release stage defaults
#[cfg(debug_assertions)]
static BUGSNAG_RELEASE_STAGE: &str = "development";
#[cfg(not(debug_assertions))]
static BUGSNAG_RELEASE_STAGE: &str = "production";
static BUGSNAG_RELEASE_TYPE: &str = "example";

// Basic default ignore list to de-noise stack traces in the UI.
// Add/adjust items to taste in your application.
const IGNORED_METHODS: &[&str] = &[
    // Backtrace capture & our helpers
    "backtrace::",
    "bugsnag::stacktrace",
    "bugsnag::bugsnag_impl::",
    // Common runtime noise
    "tokio::",
    "std::sys::",
    "std::panicking",
    "core::ops::function::FnOnce::call_once",
    "__rust_begin_short_backtrace",
];

fn build_api() -> bugsnag::Bugsnag {
    // Provide your API key via env or replace here during testing
    let api_key = std::env::var("BUGSNAG_API_KEY").unwrap_or_else(|_| "api-key".to_string());

    // Use the workspace root or crate root to improve in-project detection
    let mut api = bugsnag::Bugsnag::new(&api_key, env!("CARGO_MANIFEST_DIR"));
    api.set_app_info(
        Some(env!("CARGO_PKG_VERSION")),
        Some(BUGSNAG_RELEASE_STAGE),
        Some(BUGSNAG_RELEASE_TYPE),
    );
    // Optionally set a user if you have one
    api.set_user("example-user-id", Some("Example User"), None);
    api
}

lazy_static! {
    static ref API: bugsnag::Bugsnag = build_api();
}

/// Send a Bugsnag notification with optional context, severity and unhandled flag.
fn notify(
    error_class: &str,
    message: &str,
    severity: bugsnag::Severity,
    context: Option<&str>,
    unhandled: Option<bool>,
) {
    let mut builder = API
        .notify(error_class, message)
        .severity(severity)
        .methods_to_ignore(IGNORED_METHODS);

    if let Some(ctx) = context {
        builder = builder.context(ctx);
    }
    if let Some(unh) = unhandled {
        builder = builder.unhandled(unh);
    }

    if let Err(e) = builder.send() {
        eprintln!("Failed to notify Bugsnag: {:?}", e);
    }
}

// Convenience wrappers with sensible defaults
fn info(message: &str, context: Option<&str>) {
    notify("Info", message, bugsnag::Severity::Info, context, Some(false));
}

fn warn(message: &str, context: Option<&str>) {
    notify("Warning", message, bugsnag::Severity::Warning, context, Some(false));
}

fn error_class(error_class: &str, message: &str, context: Option<&str>) {
    notify(error_class, message, bugsnag::Severity::Error, context, Some(false));
}

fn error(message: &str, context: Option<&str>) {
    error_class("Error", message, context);
}

fn main() {
    // Demonstration of improved usage
    info("Startup complete", Some("boot/phase1"));

    // More descriptive error class improves Bugsnag grouping and title
    error_class(
        "SlackApiTimeout",
        "Slack integration sync failed: Timeout while calling Slack API",
        Some("integration_sync/slack"),
    );

    // Plain error with default class
    error("Something went wrong without classification", None);

    // Warning example
    warn("Non-fatal issue encountered", Some("housekeeping"));
}
