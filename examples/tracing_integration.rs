//! Tracing integration showing how to pass additional Bugsnag parameters
//! through structured tracing spans and events.

extern crate bugsnag;
extern crate tracing;
extern crate tracing_subscriber;

use std::collections::HashMap;
use tracing::{Event, Level, Metadata, Subscriber};
use tracing::field::{Field, Visit};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

/// Visitor to extract field values from tracing events
struct FieldVisitor {
    fields: HashMap<String, String>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(field.name().to_string(), format!("{:?}", value));
    }
    
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }
    
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }
}

/// Tracing layer that sends events to Bugsnag
struct BugsnagLayer {
    api: bugsnag::Bugsnag,
    min_level: Level,
}

impl BugsnagLayer {
    pub fn new(api: bugsnag::Bugsnag, min_level: Level) -> Self {
        Self { api, min_level }
    }
    
    fn convert_level(level: &Level) -> bugsnag::Severity {
        match *level {
            Level::ERROR => bugsnag::Severity::Error,
            Level::WARN => bugsnag::Severity::Warning,
            Level::INFO => bugsnag::Severity::Info,
            _ => bugsnag::Severity::Info,
        }
    }
}

impl<S> Layer<S> for BugsnagLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Only process events at or above our minimum level
        let metadata = event.metadata();
        if *metadata.level() > self.min_level {
            return;
        }
        
        // Extract fields from the event
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);
        
        // Extract message (usually in "message" field)
        let message = visitor.fields
            .get("message")
            .cloned()
            .unwrap_or_else(|| "No message".to_string());
        
        // Extract Bugsnag-specific parameters
        let error_class = visitor.fields
            .get("error_class")
            .map(|s| s.as_str())
            .unwrap_or_else(|| metadata.level().as_str());
        
        let context = visitor.fields
            .get("context")
            .map(|s| s.as_str());
        
        let grouping_hash = visitor.fields
            .get("grouping_hash")
            .map(|s| s.as_str());
        
        let unhandled = visitor.fields
            .get("unhandled")
            .and_then(|s| s.parse::<bool>().ok());
        
        // Build and send the notification
        let mut builder = self.api.notify(error_class, &message)
            .severity(Self::convert_level(metadata.level()));
        
        if let Some(ctx) = context {
            builder = builder.context(ctx);
        }
        
        if let Some(hash) = grouping_hash {
            builder = builder.grouping_hash(hash);
        }
        
        if let Some(unh) = unhandled {
            builder = builder.unhandled(unh);
        }
        
        if let Err(e) = builder.send() {
            eprintln!("Failed to send Bugsnag notification: {:?}", e);
        }
    }
}

// Convenience macros for common patterns
#[macro_export]
macro_rules! bugsnag_error {
    ($error_class:expr, $msg:expr) => {
        tracing::error!(
            error_class = $error_class,
            message = $msg
        );
    };
    ($error_class:expr, $msg:expr, context = $context:expr) => {
        tracing::error!(
            error_class = $error_class,
            context = $context,
            message = $msg
        );
    };
    ($error_class:expr, $msg:expr, context = $context:expr, grouping_hash = $hash:expr) => {
        tracing::error!(
            error_class = $error_class,
            context = $context,
            grouping_hash = $hash,
            message = $msg
        );
    };
    ($error_class:expr, $msg:expr, context = $context:expr, grouping_hash = $hash:expr, unhandled = $unhandled:expr) => {
        tracing::error!(
            error_class = $error_class,
            context = $context,
            grouping_hash = $hash,
            unhandled = $unhandled,
            message = $msg
        );
    };
}

#[macro_export]
macro_rules! bugsnag_warn {
    ($msg:expr) => {
        tracing::warn!(message = $msg);
    };
    ($msg:expr, context = $context:expr) => {
        tracing::warn!(
            context = $context,
            message = $msg
        );
    };
}

fn main() {
    // Set up Bugsnag API
    let api_key = std::env::var("BUGSNAG_API_KEY").unwrap_or_else(|_| "api-key".to_string());
    let mut api = bugsnag::Bugsnag::new(&api_key, env!("CARGO_MANIFEST_DIR"));
    
    api.set_app_info(
        Some(env!("CARGO_PKG_VERSION")),
        Some("development"),
        Some("rust"),
    );
    
    api.set_user("user-123", Some("Test User"), Some("test@example.com"));
    
    // Set up tracing with Bugsnag layer
    let bugsnag_layer = BugsnagLayer::new(api, Level::INFO);
    
    tracing_subscriber::registry()
        .with(bugsnag_layer)
        .with(tracing_subscriber::fmt::layer()) // Also log to console
        .init();
    
    // Examples of using tracing with Bugsnag parameters
    
    // Basic error with custom class
    tracing::error!(
        error_class = "DatabaseError",
        message = "Failed to connect to database"
    );
    
    // Error with context
    tracing::error!(
        error_class = "ValidationError",
        context = "/api/users/create",
        message = "Invalid email format"
    );
    
    // Error with grouping hash for better error grouping
    tracing::error!(
        error_class = "PaymentError",
        context = "/checkout",
        grouping_hash = "stripe-timeout",
        unhandled = false,
        message = "Stripe API timeout after 30 seconds"
    );
    
    // Using convenience macros
    bugsnag_error!("NetworkError", "Connection refused to upstream service");
    
    bugsnag_error!(
        "AuthenticationError",
        "Invalid JWT token",
        context = "/api/protected"
    );
    
    bugsnag_error!(
        "RateLimitError",
        "Too many requests from IP 192.168.1.1",
        context = "/api/search",
        grouping_hash = "rate-limit-exceeded"
    );
    
    // Example with function instrumentation
    example_function();
    
    // Warning with context
    bugsnag_warn!("Cache miss for key: user_preferences", context = "/api/preferences");
    
    // Info level
    tracing::info!(
        context = "/startup",
        message = "Application started successfully"
    );
}

// Example of instrumented function that can capture errors with context
#[tracing::instrument(fields(context = "/api/data"))]
fn example_function() {
    // This error will inherit the context from the span
    tracing::error!(
        error_class = "DataProcessingError",
        grouping_hash = "invalid-data-format",
        message = "Failed to parse JSON data"
    );
    
    // You can also use spans to add context
    let span = tracing::span!(
        Level::INFO,
        "process_payment",
        context = "/payment/process",
        user_id = "user-456"
    );
    
    let _enter = span.enter();
    
    // Errors within this span will have the context
    tracing::error!(
        error_class = "PaymentValidationError",
        grouping_hash = "invalid-card",
        unhandled = false,
        message = "Credit card validation failed"
    );
}