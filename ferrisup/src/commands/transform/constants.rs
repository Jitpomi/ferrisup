// Constants for transform commands

// List of available component types with descriptions
lazy_static::lazy_static! {
    pub static ref COMPONENT_TYPES: Vec<(&'static str, &'static str)> = vec![
        ("client", "Frontend web application"),
        ("server", "Backend API server"),
        ("shared", "Shared code between client and server"),
        ("edge", "Edge computing applications (Cloudflare, Vercel, Fastly)"),
        ("serverless", "Serverless functions (AWS Lambda, Cloudflare Workers)"),
        ("data-science", "Data science and machine learning projects"),
        ("embedded", "Embedded systems firmware"),
        ("library", "Reusable library crate"),
        ("minimal", "Minimal Rust project"),
    ];
}

/// Get a list of component type names without descriptions
pub fn get_component_type_names() -> Vec<&'static str> {
    COMPONENT_TYPES.iter().map(|(name, _)| *name).collect()
}

/// Get a list of formatted component types with descriptions for display
pub fn get_formatted_component_types() -> Vec<String> {
    COMPONENT_TYPES.iter()
        .map(|(name, desc)| format!("{} - {}", name, desc))
        .collect()
}
