use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement};
use wasm_bindgen::JsCast;
use js_sys::{Date, Math};

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

// Data structure for UI cards
struct Card {
    title: String,
    description: String,
    icon: String,
    #[allow(dead_code)]
    link: String,
}

// Main entry point for our WebAssembly module
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Use console_error_panic_hook to get better error messages if we ever panic.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    // Initialize our app
    init_app()
}

// Main function to initialize the application
#[wasm_bindgen]
pub fn init_app() -> Result<(), JsValue> {
    // Get the window and document objects
    let window = web_sys::window().expect("No global `window` exists");
    let document = window.document().expect("Should have a document on window");
    
    // Get the main container element
    let app_element = document.get_element_by_id("app").expect("Should have an #app element");
    
    // Clear any existing content
    app_element.set_inner_html("");
    
    // Create and append the header section
    create_header(&document, &app_element)?;
    
    // Create and append the hero section
    create_hero(&document, &app_element)?;
    
    // Create and append the features section
    create_features(&document, &app_element)?;
    
    // Create and append the demo section
    create_demo(&document, &app_element)?;
    
    // Create and append the footer
    create_footer(&document, &app_element)?;
    
    // Log initialization success
    console_log!("Application initialized successfully!");
    Ok(())
}

// Create the site header
fn create_header(document: &Document, parent: &Element) -> Result<(), JsValue> {
    let header = document.create_element("header")?;
    header.set_class_name("site-header");
    
    let container = document.create_element("div")?;
    container.set_class_name("container");
    
    let logo = document.create_element("div")?;
    logo.set_class_name("logo");
    logo.set_text_content(Some("🦀 {{project_name}}"));
    
    let nav = document.create_element("nav")?;
    
    let nav_items = [
        ("Home", "#"),
        ("Features", "#features"),
        ("Demo", "#demo"),
        ("GitHub", "https://github.com/Jitpomi/ferrisup"),
    ];
    
    for (text, href) in nav_items.iter() {
        let link = document.create_element("a")?;
        link.set_text_content(Some(text));
        link.set_attribute("href", href)?;
        nav.append_child(&link)?;
    }
    
    container.append_child(&logo)?;
    container.append_child(&nav)?;
    header.append_child(&container)?;
    parent.append_child(&header)?;
    
    Ok(())
}

// Create the hero section
fn create_hero(document: &Document, parent: &Element) -> Result<(), JsValue> {
    let hero = document.create_element("section")?;
    hero.set_class_name("hero");
    hero.set_id("hero");
    
    let container = document.create_element("div")?;
    container.set_class_name("container");
    
    let hero_content = document.create_element("div")?;
    hero_content.set_class_name("hero-content");
    
    let title = document.create_element("h1")?;
    title.set_text_content(Some("Rust + WebAssembly on Vercel"));
    
    let subtitle = document.create_element("p")?;
    subtitle.set_class_name("subtitle");
    subtitle.set_text_content(Some("A high-performance web application powered by Rust and deployed on Vercel's edge network"));
    
    let cta_button = document.create_element("a")?;
    cta_button.set_class_name("button primary");
    cta_button.set_text_content(Some("Get Started"));
    cta_button.set_attribute("href", "#features")?;
    
    let secondary_button = document.create_element("a")?;
    secondary_button.set_class_name("button secondary");
    secondary_button.set_text_content(Some("View Demo"));
    secondary_button.set_attribute("href", "#demo")?;
    
    let button_container = document.create_element("div")?;
    button_container.set_class_name("button-container");
    button_container.append_child(&cta_button)?;
    button_container.append_child(&secondary_button)?;
    
    hero_content.append_child(&title)?;
    hero_content.append_child(&subtitle)?;
    hero_content.append_child(&button_container)?;
    
    let hero_image = document.create_element("div")?;
    hero_image.set_class_name("hero-image");
    hero_image.set_inner_html(r##"
        <svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg" style="width: 300px; height: 300px;">
            <path fill="#FF4A00" d="M47.7,-57.2C59.9,-45.8,67,-28.5,70.1,-10.3C73.2,8,72.3,27.1,62.2,38.5C52.1,49.9,32.9,53.5,15.1,58.7C-2.7,63.9,-19.1,70.6,-33.9,66.7C-48.7,62.8,-61.9,48.3,-68.9,30.9C-75.9,13.5,-76.7,-6.8,-69.7,-22.8C-62.7,-38.8,-47.9,-50.4,-33,-58.5C-18.1,-66.5,-3.1,-70.9,9.9,-68.9C22.8,-66.9,35.5,-68.5,47.7,-57.2Z" transform="translate(100 100)" />
            <text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" fill="white" font-size="42px" font-family="Arial, sans-serif">Wasm</text>
        </svg>
    "##);
    
    container.append_child(&hero_content)?;
    container.append_child(&hero_image)?;
    hero.append_child(&container)?;
    parent.append_child(&hero)?;
    
    Ok(())
}

// Create the features section
fn create_features(document: &Document, parent: &Element) -> Result<(), JsValue> {
    let section = document.create_element("section")?;
    section.set_class_name("features");
    section.set_id("features");
    
    let container = document.create_element("div")?;
    container.set_class_name("container");
    
    let section_header = document.create_element("div")?;
    section_header.set_class_name("section-header");
    
    let title = document.create_element("h2")?;
    title.set_text_content(Some("Features"));
    
    let description = document.create_element("p")?;
    description.set_text_content(Some("Take advantage of Rust's performance and safety with WebAssembly on Vercel's edge network"));
    
    section_header.append_child(&title)?;
    section_header.append_child(&description)?;
    container.append_child(&section_header)?;
    
    // Create feature cards
    let cards = [
        Card {
            title: "Speed".to_string(),
            description: "Experience native-like performance with Rust compiled to WebAssembly".to_string(),
            icon: "⚡".to_string(),
            link: "#".to_string(),
        },
        Card {
            title: "Safety".to_string(),
            description: "Leverage Rust's memory safety and concurrency guarantees".to_string(),
            icon: "🔒".to_string(),
            link: "#".to_string(),
        },
        Card {
            title: "Global Edge".to_string(),
            description: "Deploy on Vercel's global edge network for low-latency access worldwide".to_string(),
            icon: "🌎".to_string(),
            link: "#".to_string(),
        },
        Card {
            title: "Seamless Updates".to_string(),
            description: "Continuous deployment with Git integration and preview deployments".to_string(),
            icon: "🔄".to_string(),
            link: "#".to_string(),
        },
    ];
    
    let cards_container = document.create_element("div")?;
    cards_container.set_class_name("cards");
    
    for card in cards.iter() {
        let card_element = document.create_element("div")?;
        card_element.set_class_name("card");
        
        let icon = document.create_element("div")?;
        icon.set_class_name("card-icon");
        icon.set_text_content(Some(&card.icon));
        
        let card_content = document.create_element("div")?;
        card_content.set_class_name("card-content");
        
        let card_title = document.create_element("h3")?;
        card_title.set_text_content(Some(&card.title));
        
        let card_description = document.create_element("p")?;
        card_description.set_text_content(Some(&card.description));
        
        card_content.append_child(&card_title)?;
        card_content.append_child(&card_description)?;
        
        card_element.append_child(&icon)?;
        card_element.append_child(&card_content)?;
        cards_container.append_child(&card_element)?;
    }
    
    container.append_child(&cards_container)?;
    section.append_child(&container)?;
    parent.append_child(&section)?;
    
    Ok(())
}

// Create the demo section
fn create_demo(document: &Document, parent: &Element) -> Result<(), JsValue> {
    let section = document.create_element("section")?;
    section.set_class_name("demo");
    section.set_id("demo");
    
    let container = document.create_element("div")?;
    container.set_class_name("container");
    
    let section_header = document.create_element("div")?;
    section_header.set_class_name("section-header");
    
    let title = document.create_element("h2")?;
    title.set_text_content(Some("Interactive Demo"));
    
    let description = document.create_element("p")?;
    description.set_text_content(Some("See WebAssembly in action with this interactive demo"));
    
    section_header.append_child(&title)?;
    section_header.append_child(&description)?;
    container.append_child(&section_header)?;
    
    // Create the demo area
    let demo_area = document.create_element("div")?;
    demo_area.set_class_name("demo-area");
    
    let demo_controls = document.create_element("div")?;
    demo_controls.set_class_name("demo-controls");
    
    let counter_display = document.create_element("div")?;
    counter_display.set_class_name("counter-display");
    counter_display.set_id("counter-display");
    counter_display.set_text_content(Some("0"));
    
    let button_container = document.create_element("div")?;
    button_container.set_class_name("button-container");
    
    // Create increment button
    let increment_button = document.create_element("button")?;
    increment_button.set_class_name("button primary");
    increment_button.set_text_content(Some("Increment"));
    let increment_handler = Closure::wrap(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(counter) = document.get_element_by_id("counter-display") {
                    if let Ok(current) = counter.text_content().unwrap_or_default().parse::<i32>() {
                        counter.set_text_content(Some(&(current + 1).to_string()));
                    }
                }
            }
        }
    }) as Box<dyn FnMut()>);
    increment_button.dyn_ref::<HtmlElement>().unwrap()
        .set_onclick(Some(increment_handler.as_ref().unchecked_ref()));
    increment_handler.forget();
    
    // Create decrement button
    let decrement_button = document.create_element("button")?;
    decrement_button.set_class_name("button secondary");
    decrement_button.set_text_content(Some("Decrement"));
    let decrement_handler = Closure::wrap(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(counter) = document.get_element_by_id("counter-display") {
                    if let Ok(current) = counter.text_content().unwrap_or_default().parse::<i32>() {
                        counter.set_text_content(Some(&(current - 1).to_string()));
                    }
                }
            }
        }
    }) as Box<dyn FnMut()>);
    decrement_button.dyn_ref::<HtmlElement>().unwrap()
        .set_onclick(Some(decrement_handler.as_ref().unchecked_ref()));
    decrement_handler.forget();
    
    // Create random button
    let random_button = document.create_element("button")?;
    random_button.set_class_name("button accent");
    random_button.set_text_content(Some("Random"));
    let random_handler = Closure::wrap(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(counter) = document.get_element_by_id("counter-display") {
                    let random = (Math::random() * 100.0) as i32;
                    counter.set_text_content(Some(&random.to_string()));
                }
            }
        }
    }) as Box<dyn FnMut()>);
    random_button.dyn_ref::<HtmlElement>().unwrap()
        .set_onclick(Some(random_handler.as_ref().unchecked_ref()));
    random_handler.forget();
    
    button_container.append_child(&increment_button)?;
    button_container.append_child(&decrement_button)?;
    button_container.append_child(&random_button)?;
    
    demo_controls.append_child(&counter_display)?;
    demo_controls.append_child(&button_container)?;
    
    // Create the demo output
    let demo_output = document.create_element("div")?;
    demo_output.set_class_name("demo-output");
    demo_output.set_inner_html(r##"
        <div class="output-header">
            <h3>WebAssembly Output</h3>
            <span class="badge">Live</span>
        </div>
        <pre id="output-log">// WebAssembly module initialized
// Ready to process events...</pre>
    "##);
    
    // Create a log update function
    let log_button = document.create_element("button")?;
    log_button.set_class_name("button secondary small");
    log_button.set_text_content(Some("Log Current State"));
    let log_handler = Closure::wrap(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(counter) = document.get_element_by_id("counter-display") {
                    if let Some(log) = document.get_element_by_id("output-log") {
                        let current = counter.text_content().unwrap_or_default();
                        let timestamp = Date::new_0().to_locale_time_string("en-US");
                        let log_text = format!("{}\n// Counter value: {} at {}", 
                                              log.text_content().unwrap_or_default(),
                                              current,
                                              timestamp);
                        log.set_text_content(Some(&log_text));
                    }
                }
            }
        }
    }) as Box<dyn FnMut()>);
    log_button.dyn_ref::<HtmlElement>().unwrap()
        .set_onclick(Some(log_handler.as_ref().unchecked_ref()));
    log_handler.forget();
    
    demo_output.append_child(&log_button)?;
    
    demo_area.append_child(&demo_controls)?;
    demo_area.append_child(&demo_output)?;
    
    container.append_child(&demo_area)?;
    section.append_child(&container)?;
    parent.append_child(&section)?;
    
    Ok(())
}

// Create the footer
fn create_footer(document: &Document, parent: &Element) -> Result<(), JsValue> {
    let footer = document.create_element("footer")?;
    footer.set_class_name("site-footer");
    
    let container = document.create_element("div")?;
    container.set_class_name("container");
    
    let copyright = document.create_element("p")?;
    copyright.set_class_name("copyright");
    copyright.set_inner_html("&copy; 2025 {{project_name}} - Built with <a href=\"https://github.com/Jitpomi/ferrisup\">FerrisUp</a>");
    
    let links = document.create_element("div")?;
    links.set_class_name("footer-links");
    
    for (text, url) in [("GitHub", "https://github.com"), 
                        ("Vercel", "https://vercel.com"), 
                        ("Rust", "https://rust-lang.org"),
                        ("WebAssembly", "https://webassembly.org")].iter() {
        let link = document.create_element("a")?;
        link.set_text_content(Some(text));
        link.set_attribute("href", url)?;
        link.set_attribute("target", "_blank")?;
        links.append_child(&link)?;
    }
    
    container.append_child(&copyright)?;
    container.append_child(&links)?;
    footer.append_child(&container)?;
    parent.append_child(&footer)?;
    
    Ok(())
}
