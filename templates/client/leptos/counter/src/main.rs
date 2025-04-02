use {{project_name}}::App;
use leptos::prelude::mount_to_body;

pub fn main() {
    // Initialize logging for debugging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    
    mount_to_body(App)
}
