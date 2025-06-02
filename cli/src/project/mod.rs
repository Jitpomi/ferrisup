// Project workspace entry point
pub mod handlers;
pub mod templates;

// Re-export key components
pub use handlers::{find_handler, get_handlers, ProjectHandler};
pub use templates::{list_templates, get_template_config};
