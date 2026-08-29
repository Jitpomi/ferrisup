// Project workspace entry point
pub mod handlers;
pub mod templates;

// Re-export key components
pub use handlers::{ProjectHandler, find_handler, get_handlers};
pub use templates::{get_template_config, list_templates};
