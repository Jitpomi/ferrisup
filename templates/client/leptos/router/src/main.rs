use leptos::prelude::mount_to_body;
use {{project_name}}::App;

fn main() {
    // Initialize logging for debugging
    #[cfg(debug_assertions)]
    {
        _ = console_log::init_with_level(log::Level::Debug);
        console_error_panic_hook::set_once();
    }
    
    // Mount the application to the document body
    mount_to_body(|| App());
}
