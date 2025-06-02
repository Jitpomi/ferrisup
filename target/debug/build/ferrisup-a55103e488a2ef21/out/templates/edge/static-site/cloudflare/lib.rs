use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlElement, Window};
use wasm_bindgen::JsCast;
use js_sys::Date;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// A macro to provide `println!(..)` style syntax for `console.log` logging.
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Use console_error_panic_hook to get better error messages if we ever panic.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    // Initialize the app
    init_app()
}

#[wasm_bindgen]
pub fn init_app() -> Result<(), JsValue> {
    // Get the window object
    let window: Window = web_sys::window().expect("no global `window` exists");
    
    // Get the document from the window
    let document: Document = window.document().expect("should have a document on window");
    
    // Get the app div element
    let app_div = document.get_element_by_id("app").expect("should have app element");
    
    // Clear the app div
    app_div.set_inner_html("");
    
    // Create a heading element
    let title = document.create_element("h1")?;
    title.set_text_content(Some("🦀 Rust + WebAssembly + Cloudflare Pages"));
    app_div.append_child(&title)?;
    
    // Create a description paragraph
    let description = document.create_element("p")?;
    description.set_text_content(Some("This static site is powered by Rust compiled to WebAssembly, running on Cloudflare Pages."));
    app_div.append_child(&description)?;
    
    // Create a button to demonstrate interactivity
    let button = document.create_element("button")?;
    button.set_text_content(Some("Click me!"));
    button.set_attribute("class", "action-button")?;
    
    // Create a result div to show output
    let result_div = document.create_element("div")?;
    result_div.set_attribute("id", "result")?;
    result_div.set_attribute("class", "result-area")?;
    result_div.set_text_content(Some("Click the button to see the magic happen!"));
    
    // Add button click event listener
    let result_clone = result_div.clone();
    let click_handler = Closure::wrap(Box::new(move || {
        let now = Date::new_0();
        let result = result_clone.dyn_ref::<HtmlElement>().unwrap();
        result.set_inner_html(&format!(
            "Button clicked at: <strong>{}</strong><br>Powered by Rust + WebAssembly!",
            now.to_locale_string("en-US", &JsValue::UNDEFINED)
        ));
    }) as Box<dyn FnMut()>);
    
    button.dyn_ref::<HtmlElement>()
        .expect("button should be an `HtmlElement`")
        .set_onclick(Some(click_handler.as_ref().unchecked_ref()));
    
    // Forget the closure to keep it alive (avoid dropping)
    click_handler.forget();
    
    // Append elements to the app div
    app_div.append_child(&button)?;
    app_div.append_child(&result_div)?;
    
    // Add a section for custom content
    let content_section = document.create_element("section")?;
    content_section.set_attribute("class", "content-section")?;
    
    let section_title = document.create_element("h2")?;
    section_title.set_text_content(Some("Features"));
    content_section.append_child(&section_title)?;
    
    // Create a features list
    let features_list = document.create_element("ul")?;
    
    let features = [
        "🔥 Fast - Rust compiled to WebAssembly for native speed",
        "⚡ Optimized - Small bundle size for quick loading",
        "🌍 Global - Deployed on Cloudflare's global edge network",
        "🔧 Customizable - Easy to extend and modify",
        "🦀 Rusty - Leverage the safety and performance of Rust"
    ];
    
    for feature in features.iter() {
        let li = document.create_element("li")?;
        li.set_text_content(Some(feature));
        features_list.append_child(&li)?;
    }
    
    content_section.append_child(&features_list)?;
    app_div.append_child(&content_section)?;
    
    console_log!("Application initialized successfully!");
    Ok(())
}
