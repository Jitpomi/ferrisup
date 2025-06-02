use leptos::prelude::mount_to_body;
use {{project_name}}::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
