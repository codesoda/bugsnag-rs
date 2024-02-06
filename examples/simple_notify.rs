//! An example showing the integration of the bugsnag api for a simple notification.
//!
//! This simple implementations consumes the api object. So, we can not change
//! any parameters after registering the panic handler.

extern crate bugsnag;

fn main() {
    let api_key = "api-key";
    let mut api =
        bugsnag::Bugsnag::new(api_key, concat!(env!("CARGO_MANIFEST_DIR"), "/examples"));
    api.set_app_info(
        Some(env!("CARGO_PKG_VERSION")),
        Some("development"),
        Some("rust"),
    );
    api.set_user(
        "19",
        Some("Chris Raethke"),
        Some("chris@example.com")
    );

    api.notify("ErrorClass", "This is a test with no handling info")
        .severity(bugsnag::Severity::Warning);

    api.notify("ErrorClass", "This is a handled test")
        .severity(bugsnag::Severity::Error)
        .unhandled(false);

    api.notify("ErrorClass", "This is an unhandled test")
        .severity(bugsnag::Severity::Error)
        .unhandled(true);
    
}
