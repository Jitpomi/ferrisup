use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};
use wasm_bindgen::JsCast;

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

// Initialize the application
#[wasm_bindgen(start)]
pub fn init_app() -> Result<(), JsValue> {
    // Set panic hook for better debugging
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("No global window exists");
    let document = window.document().expect("Should have a document on window");
    
    // Create the main elements of the page
    create_header(&document)?;
    create_hero(&document)?;
    create_features(&document)?;
    create_demo(&document)?;
    create_footer(&document)?;

    Ok(())
}

// Create the site header
fn create_header(document: &Document) -> Result<(), JsValue> {
    let header = document.get_element_by_id("header")
        .expect("Should have a header element");
    
    let logo = document.create_element("div")?;
    logo.set_class_name("logo");
    logo.set_text_content(Some("{{project_name}}"));
    
    let nav = document.create_element("nav")?;
    nav.set_class_name("nav-links");
    
    let nav_items = ["Home", "Features", "Demo", "Docs", "Contact"];
    for item in nav_items.iter() {
        let link = document.create_element("a")?;
        link.set_class_name("nav-link");
        link.set_text_content(Some(item));
        link.set_attribute("href", &format!("#{}", item.to_lowercase()))?;
        nav.append_child(&link)?;
    }
    
    // Create mobile menu toggle
    let menu_toggle = document.create_element("button")?
        .dyn_into::<HtmlElement>()?;
    menu_toggle.set_class_name("mobile-menu-toggle");
    menu_toggle.set_inner_html("☰");
    
    // Add click handler to toggle mobile menu
    let document_clone = document.clone();
    let click_callback = Closure::wrap(Box::new(move || {
        if let Some(nav_element) = document_clone.query_selector(".nav-links").ok().flatten() {
            let nav_html = nav_element.dyn_into::<HtmlElement>().unwrap();
            // Get the className
            let class_name = nav_html.class_name();
            if class_name.contains("active") {
                // Remove active class
                nav_html.set_class_name(&class_name.replace("active", "").trim());
            } else {
                // Add active class
                nav_html.set_class_name(&format!("{} active", class_name));
            }
        }
    }) as Box<dyn FnMut()>);
    
    menu_toggle.set_onclick(Some(click_callback.as_ref().unchecked_ref()));
    click_callback.forget();
    
    header.append_child(&logo)?;
    header.append_child(&nav)?;
    header.append_child(&menu_toggle)?;
    
    Ok(())
}

// Create the hero section
fn create_hero(document: &Document) -> Result<(), JsValue> {
    let hero = document.get_element_by_id("hero")
        .expect("Should have a hero element");
    
    let content = document.create_element("div")?;
    content.set_class_name("hero-content");
    
    let title = document.create_element("h1")?;
    title.set_class_name("hero-title");
    title.set_text_content(Some("Rust + WebAssembly on Netlify"));
    
    let subtitle = document.create_element("p")?;
    subtitle.set_class_name("hero-subtitle");
    subtitle.set_text_content(Some("Build blazing fast static sites with native-like performance using Rust compiled to WebAssembly"));
    
    let cta = document.create_element("div")?;
    cta.set_class_name("hero-cta");
    
    let primary_btn = document.create_element("a")?;
    primary_btn.set_class_name("btn btn-primary");
    primary_btn.set_text_content(Some("Get Started"));
    primary_btn.set_attribute("href", "#demo")?;
    
    let secondary_btn = document.create_element("a")?;
    secondary_btn.set_class_name("btn btn-secondary");
    secondary_btn.set_text_content(Some("Learn More"));
    secondary_btn.set_attribute("href", "#features")?;
    
    cta.append_child(&primary_btn)?;
    cta.append_child(&secondary_btn)?;
    
    content.append_child(&title)?;
    content.append_child(&subtitle)?;
    content.append_child(&cta)?;
    
    hero.append_child(&content)?;
    
    Ok(())
}

// Create the features section
fn create_features(document: &Document) -> Result<(), JsValue> {
    let features = document.get_element_by_id("features")
        .expect("Should have a features element");
    
    let section_header = document.create_element("div")?;
    section_header.set_class_name("section-header");
    
    let section_title = document.create_element("h2")?;
    section_title.set_text_content(Some("Why Rust + WebAssembly?"));
    
    let section_desc = document.create_element("p")?;
    section_desc.set_text_content(Some("Combine the power of Rust with the reach of the web"));
    
    section_header.append_child(&section_title)?;
    section_header.append_child(&section_desc)?;
    
    let feature_cards = document.create_element("div")?;
    feature_cards.set_class_name("feature-cards");
    
    let feature_info = [
        ("Native-Like Performance", "WebAssembly runs at near-native speed, making your web applications lightning fast"),
        ("Type Safety", "Rust's strong type system helps prevent common bugs at compile time"),
        ("Memory Safety", "Rust guarantees memory safety without a garbage collector"),
        ("Seamless Deployment", "Deploy to Netlify's global edge network with a single command"),
        ("Optimized Binaries", "Smaller bundle sizes mean faster loading times for your users"),
        ("Modern Development", "Use modern tooling for a productive development experience")
    ];
    
    for (title, description) in feature_info.iter() {
        let card = document.create_element("div")?;
        card.set_class_name("feature-card");
        
        let card_title = document.create_element("h3")?;
        card_title.set_text_content(Some(title));
        
        let card_desc = document.create_element("p")?;
        card_desc.set_text_content(Some(description));
        
        card.append_child(&card_title)?;
        card.append_child(&card_desc)?;
        
        feature_cards.append_child(&card)?;
    }
    
    features.append_child(&section_header)?;
    features.append_child(&feature_cards)?;
    
    Ok(())
}

// Create an interactive demo section
fn create_demo(document: &Document) -> Result<(), JsValue> {
    let demo = document.get_element_by_id("demo")
        .expect("Should have a demo element");
    
    let section_header = document.create_element("div")?;
    section_header.set_class_name("section-header");
    
    let section_title = document.create_element("h2")?;
    section_title.set_text_content(Some("Interactive Demo"));
    
    let section_desc = document.create_element("p")?;
    section_desc.set_text_content(Some("Try out the WebAssembly-powered features"));
    
    section_header.append_child(&section_title)?;
    section_header.append_child(&section_desc)?;
    
    // Create counter demo
    let counter_demo = document.create_element("div")?;
    counter_demo.set_class_name("demo-widget");
    
    let counter_title = document.create_element("h3")?;
    counter_title.set_text_content(Some("Wasm Counter"));
    
    let counter_display = document.create_element("div")?;
    counter_display.set_class_name("counter-display");
    counter_display.set_text_content(Some("0"));
    counter_display.set_id("counter-value");
    
    let counter_controls = document.create_element("div")?;
    counter_controls.set_class_name("counter-controls");
    
    let decrement_btn = document.create_element("button")?
        .dyn_into::<HtmlElement>()?;
    decrement_btn.set_class_name("counter-btn");
    decrement_btn.set_text_content(Some("-"));
    
    let increment_btn = document.create_element("button")?
        .dyn_into::<HtmlElement>()?;
    increment_btn.set_class_name("counter-btn");
    increment_btn.set_text_content(Some("+"));
    
    // Add counter logic
    let document_clone = document.clone();
    let increment_callback = Closure::wrap(Box::new(move || {
        if let Some(counter_el) = document_clone.get_element_by_id("counter-value") {
            let current_value = counter_el.text_content()
                .unwrap_or_else(|| "0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            counter_el.set_text_content(Some(&(current_value + 1).to_string()));
        }
    }) as Box<dyn FnMut()>);
    
    let document_clone = document.clone();
    let decrement_callback = Closure::wrap(Box::new(move || {
        if let Some(counter_el) = document_clone.get_element_by_id("counter-value") {
            let current_value = counter_el.text_content()
                .unwrap_or_else(|| "0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            counter_el.set_text_content(Some(&(current_value - 1).to_string()));
        }
    }) as Box<dyn FnMut()>);
    
    increment_btn.set_onclick(Some(increment_callback.as_ref().unchecked_ref()));
    decrement_btn.set_onclick(Some(decrement_callback.as_ref().unchecked_ref()));
    
    increment_callback.forget();
    decrement_callback.forget();
    
    counter_controls.append_child(&decrement_btn)?;
    counter_controls.append_child(&increment_btn)?;
    
    counter_demo.append_child(&counter_title)?;
    counter_demo.append_child(&counter_display)?;
    counter_demo.append_child(&counter_controls)?;
    
    // Create fibonacci calculator
    let fib_demo = document.create_element("div")?;
    fib_demo.set_class_name("demo-widget");
    
    let fib_title = document.create_element("h3")?;
    fib_title.set_text_content(Some("Fibonacci Calculator"));
    
    let fib_desc = document.create_element("p")?;
    fib_desc.set_text_content(Some("Calculate Fibonacci numbers quickly with Wasm"));
    
    let fib_input_group = document.create_element("div")?;
    fib_input_group.set_class_name("input-group");
    
    let fib_input = document.create_element("input")?
        .dyn_into::<web_sys::HtmlInputElement>()?;
    fib_input.set_type("number");
    fib_input.set_min("1");
    fib_input.set_max("42");
    fib_input.set_value("10");
    fib_input.set_id("fib-input");
    
    let fib_btn = document.create_element("button")?
        .dyn_into::<HtmlElement>()?;
    fib_btn.set_class_name("btn btn-primary");
    fib_btn.set_text_content(Some("Calculate"));
    
    let fib_result = document.create_element("div")?;
    fib_result.set_class_name("result-display");
    fib_result.set_id("fib-result");
    fib_result.set_text_content(Some("Result: 55"));
    
    // Add fibonacci calculation logic
    let document_clone = document.clone();
    let fib_callback = Closure::wrap(Box::new(move || {
        if let Some(input_el) = document_clone.get_element_by_id("fib-input") {
            if let Some(result_el) = document_clone.get_element_by_id("fib-result") {
                if let Some(input) = input_el.dyn_ref::<web_sys::HtmlInputElement>() {
                    if let Ok(n) = input.value().parse::<u32>() {
                        // Cap at 42 to prevent potential overflow
                        let n = std::cmp::min(n, 42);
                        let result = fibonacci(n);
                        result_el.set_text_content(Some(&format!("Result: {}", result)));
                    }
                }
            }
        }
    }) as Box<dyn FnMut()>);
    
    fib_btn.set_onclick(Some(fib_callback.as_ref().unchecked_ref()));
    fib_callback.forget();
    
    fib_input_group.append_child(&fib_input)?;
    fib_input_group.append_child(&fib_btn)?;
    
    fib_demo.append_child(&fib_title)?;
    fib_demo.append_child(&fib_desc)?;
    fib_demo.append_child(&fib_input_group)?;
    fib_demo.append_child(&fib_result)?;
    
    let demo_container = document.create_element("div")?;
    demo_container.set_class_name("demo-container");
    demo_container.append_child(&counter_demo)?;
    demo_container.append_child(&fib_demo)?;
    
    demo.append_child(&section_header)?;
    demo.append_child(&demo_container)?;
    
    Ok(())
}

// Create the footer
fn create_footer(document: &Document) -> Result<(), JsValue> {
    let footer = document.get_element_by_id("footer")
        .expect("Should have a footer element");
    
    let footer_content = document.create_element("div")?;
    footer_content.set_class_name("footer-content");
    
    let copyright = document.create_element("div")?;
    copyright.set_class_name("copyright");
    copyright.set_text_content(Some(" 2023 {{project_name}} | Built with Rust, WebAssembly, and Netlify"));
    
    let links = document.create_element("div")?;
    links.set_class_name("footer-links");
    
    let link_data = [
        ("GitHub", "#"),
        ("Documentation", "#"),
        ("Privacy Policy", "#"),
        ("Terms of Service", "#")
    ];
    
    for (text, href) in link_data.iter() {
        let link = document.create_element("a")?;
        link.set_class_name("footer-link");
        link.set_text_content(Some(text));
        link.set_attribute("href", href)?;
        links.append_child(&link)?;
    }
    
    footer_content.append_child(&copyright)?;
    footer_content.append_child(&links)?;
    
    footer.append_child(&footer_content)?;
    
    Ok(())
}

// Fibonacci calculation function
fn fibonacci(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    
    let mut a = 0u64;
    let mut b = 1u64;
    
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    
    b
}
