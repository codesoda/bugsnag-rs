//! Comparison of how env_logger vs custom bugsnag logger handle structured logging
//! 
//! This example shows what happens when you use key-value pairs with different loggers.

extern crate bugsnag;
#[macro_use]
extern crate log;

use log::{kv, Level, Metadata, Record};
use std::io::Write;

/// What env_logger does with structured logging
fn demonstrate_env_logger() {
    println!("\n=== ENV_LOGGER BEHAVIOR ===\n");
    
    // env_logger WITHOUT kv feature (default):
    // - Ignores all key-value pairs
    // - Only prints the message
    // Output: [ERROR] Failed to connect to database
    
    // env_logger WITH kv feature enabled:
    // - Prints key-value pairs inline with the message
    // Output: [ERROR] Failed to connect to database error_class="DatabaseError" context="/api/users" grouping_hash="db-connection-pool"
    
    println!("Without kv feature: [ERROR] Failed to connect to database");
    println!("With kv feature:    [ERROR] Failed to connect to database error_class=\"DatabaseError\" context=\"/api/users\" grouping_hash=\"db-connection-pool\"");
}

/// Basic Bugsnag logger (like your current example) - ignores KV pairs
struct BasicBugsnagLogger {
    api: bugsnag::Bugsnag,
}

impl log::Log for BasicBugsnagLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // This logger doesn't check key-values at all
            // It would just send: error_class="Error", message="Failed to connect to database"
            let message = record.args().to_string();
            println!("BasicBugsnagLogger would send:");
            println!("  error_class: \"{}\"", record.level());
            println!("  message: \"{}\"", message);
            println!("  (all KV pairs are IGNORED)");
        }
    }

    fn flush(&self) {}
}

/// Enhanced Bugsnag logger that extracts KV pairs
struct EnhancedBugsnagLogger {
    api: bugsnag::Bugsnag,
}

// Visitor to extract KV pairs (requires log crate with kv feature)
struct KvExtractor {
    error_class: Option<String>,
    context: Option<String>,
    grouping_hash: Option<String>,
}

impl KvExtractor {
    fn new() -> Self {
        Self {
            error_class: None,
            context: None,
            grouping_hash: None,
        }
    }
}

impl<'kvs> kv::VisitSource<'kvs> for KvExtractor {
    fn visit_pair(&mut self, key: kv::Key<'kvs>, value: kv::Value<'kvs>) -> Result<(), kv::Error> {
        match key.as_str() {
            "error_class" => self.error_class = Some(value.to_string()),
            "context" => self.context = Some(value.to_string()),
            "grouping_hash" => self.grouping_hash = Some(value.to_string()),
            _ => {} // Ignore other fields
        }
        Ok(())
    }
}

impl log::Log for EnhancedBugsnagLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = record.args().to_string();
            
            // Extract KV pairs if log crate has kv feature
            let mut extractor = KvExtractor::new();
            let _ = record.key_values().visit(&mut extractor);
            
            let error_class = extractor.error_class
                .as_deref()
                .unwrap_or(record.level().as_str());
            
            println!("\nEnhancedBugsnagLogger would send:");
            println!("  error_class: \"{}\"", error_class);
            println!("  message: \"{}\"", message);
            if let Some(ctx) = &extractor.context {
                println!("  context: \"{}\"", ctx);
            }
            if let Some(hash) = &extractor.grouping_hash {
                println!("  grouping_hash: \"{}\"", hash);
            }
            
            // In real implementation:
            // let mut builder = self.api.notify(error_class, &message);
            // if let Some(ctx) = extractor.context { builder = builder.context(&ctx); }
            // if let Some(hash) = extractor.grouping_hash { builder = builder.grouping_hash(&hash); }
            // builder.send();
        }
    }

    fn flush(&self) {}
}

/// Multi-logger that combines env_logger and Bugsnag
struct CombinedLogger {
    env_logger: env_logger::Logger,
    bugsnag_logger: Box<dyn log::Log>,
}

impl log::Log for CombinedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.env_logger.enabled(metadata) || self.bugsnag_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        // Both loggers process the same record
        self.env_logger.log(record);
        self.bugsnag_logger.log(record);
    }

    fn flush(&self) {
        self.env_logger.flush();
        self.bugsnag_logger.flush();
    }
}

fn main() {
    demonstrate_env_logger();
    
    println!("\n=== BASIC BUGSNAG LOGGER (current implementation) ===");
    
    // Simulate what happens with basic logger
    let api = bugsnag::Bugsnag::new("api-key", env!("CARGO_MANIFEST_DIR"));
    let basic_logger = BasicBugsnagLogger { api: api.clone() };
    
    // Create a fake record to demonstrate
    // In real usage this would come from: error!(target: "bugsnag", ...)
    println!("\nWhen you call:");
    println!("  error!(target: \"bugsnag\",");
    println!("         error_class = \"DatabaseError\",");
    println!("         context = \"/api/users\",");
    println!("         grouping_hash = \"db-connection-pool\",");
    println!("         \"Failed to connect to database\");");
    println!("");
    
    // Show what basic logger does (ignores KV pairs)
    // basic_logger.log(&record);
    
    println!("\n=== ENHANCED BUGSNAG LOGGER (with KV support) ===");
    
    let enhanced_logger = EnhancedBugsnagLogger { api };
    // enhanced_logger.log(&record);
    
    println!("\n=== KEY DIFFERENCES ===\n");
    println!("1. env_logger (default): Ignores KV pairs entirely, just logs message");
    println!("2. env_logger (with kv): Prints KV pairs as part of the log line");
    println!("3. Basic BugsnagLogger: Ignores KV pairs, uses log level as error_class");
    println!("4. Enhanced BugsnagLogger: Extracts KV pairs and uses them in Bugsnag API");
    
    println!("\n=== RECOMMENDATION ===\n");
    println!("To use structured logging with Bugsnag:");
    println!("1. Enable the 'kv' feature in Cargo.toml:");
    println!("   log = {{ version = \"0.4\", features = [\"kv\"] }}");
    println!("");
    println!("2. Use the Enhanced logger that extracts KV pairs");
    println!("");
    println!("3. Optionally combine with env_logger for console output:");
    println!("   - Use a CombinedLogger that delegates to both");
    println!("   - Or use target filtering (target: \"bugsnag\" goes to Bugsnag only)");
}