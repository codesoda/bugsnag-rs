//! Comparison of how different tracing subscribers handle structured fields
//! when you use tracing macros with Bugsnag-specific fields.

use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;

fn main() {
    println!("=== TRACING BEHAVIOR WITH DIFFERENT SUBSCRIBERS ===\n");
    
    // Example call we'll be making:
    println!("When you call:");
    println!("  tracing::error!(");
    println!("      error_class = \"DatabaseError\",");
    println!("      context = \"/api/users\",");
    println!("      grouping_hash = \"db-connection-pool\",");
    println!("      \"Failed to connect to database\"");
    println!("  );\n");
    
    demonstrate_fmt_subscriber();
    demonstrate_env_filter();
    demonstrate_json_subscriber();
    demonstrate_custom_bugsnag_layer();
    show_recommendations();
}

fn demonstrate_fmt_subscriber() {
    println!("=== 1. tracing_subscriber::fmt (default formatter) ===");
    println!("Output: ERROR example: Failed to connect to database error_class=\"DatabaseError\" context=\"/api/users\" grouping_hash=\"db-connection-pool\"");
    println!("→ ALL fields are printed inline with the message\n");
}

fn demonstrate_env_filter() {
    println!("=== 2. tracing_subscriber::fmt with EnvFilter ===");
    println!("Same as above, but you can filter by level/target:");
    println!("  RUST_LOG=error → shows this event");
    println!("  RUST_LOG=bugsnag=error → only if target=\"bugsnag\"");
    println!("→ Fields are still just printed, not processed specially\n");
}

fn demonstrate_json_subscriber() {
    println!("=== 3. tracing_subscriber::fmt::json ===");
    println!("Output (formatted):");
    println!("{{");
    println!("  \"timestamp\": \"2024-01-15T10:30:00Z\",");
    println!("  \"level\": \"ERROR\",");
    println!("  \"message\": \"Failed to connect to database\",");
    println!("  \"error_class\": \"DatabaseError\",");
    println!("  \"context\": \"/api/users\",");
    println!("  \"grouping_hash\": \"db-connection-pool\"");
    println!("}}");
    println!("→ Fields become JSON properties - good for log aggregation\n");
}

fn demonstrate_custom_bugsnag_layer() {
    println!("=== 4. Custom BugsnagLayer (from previous example) ===");
    println!("The layer extracts fields and calls Bugsnag API:");
    println!("  api.notify(\"DatabaseError\", \"Failed to connect to database\")");
    println!("     .context(\"/api/users\")");
    println!("     .grouping_hash(\"db-connection-pool\")");
    println!("     .send()");
    println!("→ Fields are EXTRACTED and used as Bugsnag parameters\n");
}

fn show_recommendations() {
    println!("=== KEY DIFFERENCES FROM log CRATE ===\n");
    println!("1. **Tracing ALWAYS supports structured fields** - no feature flag needed");
    println!("2. **Fields are first-class** - every subscriber sees them");
    println!("3. **Type-safe** - fields are compile-time checked\n");
    
    println!("=== TYPICAL SETUP: MULTIPLE LAYERS ===\n");
    println!("```rust");
    println!("tracing_subscriber::registry()");
    println!("    // Console output with all fields visible");
    println!("    .with(tracing_subscriber::fmt::layer())");
    println!("    ");
    println!("    // Bugsnag layer that extracts specific fields");
    println!("    .with(BugsnagLayer::new(bugsnag_api))");
    println!("    ");
    println!("    // Optional: JSON logs to file");
    println!("    .with(");
    println!("        tracing_subscriber::fmt::layer()");
    println!("            .json()");
    println!("            .with_writer(std::fs::File::create(\"app.log\"))");
    println!("    )");
    println!("    .init();");
    println!("```\n");
    
    println!("=== WHAT EACH LAYER SEES ===\n");
    println!("When you call:");
    println!("  error!(error_class = \"DatabaseError\", \"Failed\");");
    println!("");
    println!("• fmt layer → prints: ERROR Failed error_class=\"DatabaseError\"");
    println!("• BugsnagLayer → sends to Bugsnag with error_class extracted");
    println!("• JSON layer → writes: {{\"level\":\"ERROR\",\"message\":\"Failed\",\"error_class\":\"DatabaseError\"}}");
    println!("");
    println!("ALL layers see ALL fields - they just handle them differently!");
}