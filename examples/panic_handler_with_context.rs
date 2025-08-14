//! Enhanced panic handler that can extract additional context from various sources
//! including thread-local storage, global state, and panic payloads.

extern crate bugsnag;
extern crate lazy_static;

use std::cell::RefCell;
use std::panic;
use std::sync::{Arc, Mutex};

// Thread-local storage for request context
thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<RequestContext>> = RefCell::new(None);
}

#[derive(Clone, Debug)]
struct RequestContext {
    path: String,
    user_id: Option<String>,
    request_id: String,
    grouping_hash: Option<String>,
}

// Global state for application-wide context
lazy_static::lazy_static! {
    static ref APP_CONTEXT: Arc<Mutex<AppContext>> = Arc::new(Mutex::new(AppContext::default()));
}

#[derive(Default)]
struct AppContext {
    environment: String,
    version: String,
    feature_flags: Vec<String>,
}

/// Enhanced panic handler that extracts context from multiple sources
fn handle_panic_with_context(
    api: &bugsnag::Bugsnag,
    info: &panic::PanicHookInfo,
    methods_to_ignore: Option<&[&str]>,
) -> Result<(), bugsnag::Error> {
    // Extract panic message
    let message = if let Some(data) = info.payload().downcast_ref::<String>() {
        data.to_owned()
    } else if let Some(data) = info.payload().downcast_ref::<&str>() {
        (*data).to_owned()
    } else {
        format!("Error: {:?}", info.payload())
    };
    
    // Start building notification with enhanced error class
    let error_class = determine_error_class(&message);
    let mut notify = api.notify(&error_class, &message)
        .severity(bugsnag::Severity::Error)
        .unhandled(true);
    
    // Add context from thread-local storage (e.g., current HTTP request)
    CURRENT_CONTEXT.with(|ctx| {
        if let Some(ref context) = *ctx.borrow() {
            notify = notify.context(&context.path);
            
            if let Some(ref hash) = context.grouping_hash {
                notify = notify.grouping_hash(hash);
            }
            
            // In a real implementation, you might also:
            // - Set user info: api.set_user(&context.user_id, ...)
            // - Add metadata about the request
        }
    });
    
    // Add panic location if available
    if let Some(location) = info.location() {
        let location_str = format!("{}:{}", location.file(), location.line());
        // Could add this as metadata or incorporate into grouping hash
        notify = notify.grouping_hash(&format!("panic-{}", location.file()));
    }
    
    // Apply methods to ignore
    if let Some(methods) = methods_to_ignore {
        notify = notify.methods_to_ignore(methods);
    }
    
    notify.send()
}

/// Determine a more specific error class based on the panic message
fn determine_error_class(message: &str) -> String {
    if message.contains("unwrap") || message.contains("Option::None") {
        "UnwrapPanic".to_string()
    } else if message.contains("index out of bounds") {
        "IndexOutOfBounds".to_string()
    } else if message.contains("assertion") || message.contains("assert") {
        "AssertionFailed".to_string()
    } else if message.contains("overflow") {
        "ArithmeticOverflow".to_string()
    } else {
        "Panic".to_string()
    }
}

/// Middleware/guard to set request context before handling
pub struct RequestContextGuard {
    _restore: Option<RequestContext>,
}

impl RequestContextGuard {
    pub fn new(context: RequestContext) -> Self {
        let restore = CURRENT_CONTEXT.with(|ctx| {
            ctx.borrow_mut().replace(context)
        });
        RequestContextGuard { _restore: restore }
    }
}

impl Drop for RequestContextGuard {
    fn drop(&mut self) {
        CURRENT_CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = self._restore.take();
        });
    }
}

/// Register the enhanced panic handler
fn register_enhanced_panic_handler(api: bugsnag::Bugsnag) {
    panic::set_hook(Box::new(move |info| {
        if handle_panic_with_context(&api, &info, Some(&["register_enhanced_panic_handler"])).is_err() {
            eprintln!("Failed to notify Bugsnag about panic!");
        }
    }));
}

/// Example: Web framework integration
fn handle_request(path: &str, user_id: Option<&str>) {
    // Set context for this request
    let _guard = RequestContextGuard::new(RequestContext {
        path: path.to_string(),
        user_id: user_id.map(String::from),
        request_id: uuid::Uuid::new_v4().to_string(),
        grouping_hash: Some(format!("route-{}", path)),
    });
    
    // Simulate request processing that might panic
    process_request();
}

fn process_request() {
    // This will panic with our context available
    let data: Vec<i32> = vec![1, 2, 3];
    let _ = data[10]; // Index out of bounds panic
}

/// Alternative: Custom panic payload with context
#[derive(Debug)]
struct ContextualPanic {
    message: String,
    error_class: String,
    context: String,
    grouping_hash: Option<String>,
}

impl std::fmt::Display for ContextualPanic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Panic with explicit context
fn panic_with_context(error_class: &str, message: &str, context: &str, grouping_hash: Option<&str>) {
    panic::panic_any(ContextualPanic {
        message: message.to_string(),
        error_class: error_class.to_string(),
        context: context.to_string(),
        grouping_hash: grouping_hash.map(String::from),
    });
}

/// Enhanced handler that checks for ContextualPanic
fn handle_contextual_panic(
    api: &bugsnag::Bugsnag,
    info: &panic::PanicHookInfo,
) -> Result<(), bugsnag::Error> {
    // Check if this is a ContextualPanic
    if let Some(contextual) = info.payload().downcast_ref::<ContextualPanic>() {
        let mut notify = api.notify(&contextual.error_class, &contextual.message)
            .severity(bugsnag::Severity::Error)
            .context(&contextual.context)
            .unhandled(true);
        
        if let Some(ref hash) = contextual.grouping_hash {
            notify = notify.grouping_hash(hash);
        }
        
        return notify.send();
    }
    
    // Fall back to regular panic handling
    handle_panic_with_context(api, info, None)
}

fn main() {
    let api_key = std::env::var("BUGSNAG_API_KEY").unwrap_or_else(|_| "api-key".to_string());
    let mut api = bugsnag::Bugsnag::new(&api_key, env!("CARGO_MANIFEST_DIR"));
    
    api.set_app_info(
        Some(env!("CARGO_PKG_VERSION")),
        Some("development"),
        Some("rust"),
    );
    
    // Register the enhanced panic handler
    register_enhanced_panic_handler(api);
    
    println!("=== PANIC HANDLER CONTEXT STRATEGIES ===\n");
    
    println!("1. Thread-Local Context (best for web servers):");
    println!("   - Set context at request boundary");
    println!("   - Automatically available in panic handler");
    println!("   - Cleaned up when request completes\n");
    
    println!("2. Custom Panic Payloads:");
    println!("   - Use panic_any() with structured data");
    println!("   - Handler extracts fields from payload");
    println!("   - Most explicit but requires code changes\n");
    
    println!("3. Smart Error Classification:");
    println!("   - Parse panic message for patterns");
    println!("   - Set error_class based on panic type");
    println!("   - Better grouping in Bugsnag UI\n");
    
    println!("4. Location-based Grouping:");
    println!("   - Use panic location for grouping_hash");
    println!("   - Groups panics by source location");
    println!("   - Helpful for tracking down issues\n");
    
    // Example: Simulate a request that panics
    println!("Simulating panic with context...");
    // Uncomment to test:
    // handle_request("/api/users/123", Some("user-456"));
    
    // Example: Explicit contextual panic
    // panic_with_context(
    //     "ValidationError",
    //     "Invalid user input",
    //     "/api/validate",
    //     Some("validation-failure")
    // );
}