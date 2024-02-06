//! An example showing the integration of the bugsnag api with the logging framework of rust.
//!
//! This simple implementations consumes the api object. So, we can not change
//! any parameters after registering the logger.

extern crate bugsnag;
#[macro_use]
extern crate log;

use log::{Level, Metadata, Record, SetLoggerError};

/// Our logger for bugsnag
struct BugsnagLogger {
    max_loglevel: Level,
    api: bugsnag::Bugsnag,
}

impl BugsnagLogger {
    pub fn init(api: bugsnag::Bugsnag, max_loglevel: Level) -> Result<(), SetLoggerError> {
        let logger: Box<dyn log::Log + 'static> = Box::new(BugsnagLogger {
            api: api,
            max_loglevel: max_loglevel,
        });
        match log::set_logger(Box::leak(logger)) {
            Ok(()) => {
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }
}

fn convert_log_level(level: Level) -> bugsnag::Severity {
    match level {
        Level::Info => bugsnag::Severity::Info,
        Level::Warn => bugsnag::Severity::Warning,
        Level::Error => bugsnag::Severity::Error,
        _ => bugsnag::Severity::Info,
    }
}

impl log::Log for BugsnagLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_loglevel
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = convert_log_level(record.metadata().level());
            let level_str = record.metadata().level().to_string();
            let message = record.args().to_string();

            self.api.notify(&level_str, &message).severity(level);
        }
    }

    fn flush(&self) {}
}

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

    // initialize the logger
    BugsnagLogger::init(api, Level::Warn).unwrap();

    // the following two messages should not show up in bugsnag, because
    // we set the maximum log level to errors
    debug!("Hello this is a debug message!");
    info!("Hello this is a info message!");

    // the following two should be send to bugsnag
    warn!("Hello this is a warn message!");
    error!("Hello this is a error message!");
}
