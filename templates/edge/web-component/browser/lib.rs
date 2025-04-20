use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement, Window, Document};
use wasm_bindgen::JsCast;
use js_sys::Function;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Custom component properties
#[wasm_bindgen]
#[derive(Default)]
pub struct ComponentProperties {
    title: String,
    description: String,
    theme: String,
}

#[wasm_bindgen]
impl ComponentProperties {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    #[wasm_bindgen(getter)]
    pub fn theme(&self) -> String {
        self.theme.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_theme(&mut self, theme: String) {
        self.theme = theme;
    }
}

// Main WebComponent class
#[wasm_bindgen]
pub struct RustComponent {
    element: HtmlElement,
    shadow_root: web_sys::ShadowRoot,
    properties: ComponentProperties,
    click_handler: Option<Function>,
}

#[wasm_bindgen]
impl RustComponent {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RustComponent, JsValue> {
        // Use console_error_panic_hook to get better error messages
        console_error_panic_hook::set_once();
        
        // Get the window and document
        let window = web_sys::window().expect("should have a window");
        let document = window.document().expect("should have a document");
        
        // Create a custom element
        let element = document.create_element("div")?
            .dyn_into::<HtmlElement>()?;
        
        // Attach a shadow DOM
        let shadow_root = element.attach_shadow(&web_sys::ShadowRootInit::new())?;
        
        // Create default properties
        let properties = ComponentProperties {
            title: "Rust WebComponent".to_string(),
            description: "A web component built with Rust and WebAssembly".to_string(),
            theme: "light".to_string(),
        };
        
        // Create and return the component
        let component = RustComponent {
            element,
            shadow_root,
            properties,
            click_handler: None,
        };
        
        // Render the initial component
        component.render()?;
        
        Ok(component)
    }
    
    // Get the DOM element
    #[wasm_bindgen]
    pub fn element(&self) -> HtmlElement {
        self.element.clone()
    }
    
    // Update component properties and re-render
    #[wasm_bindgen]
    pub fn update(&mut self, properties: ComponentProperties) -> Result<(), JsValue> {
        self.properties = properties;
        self.render()
    }
    
    // Set a click handler function
    #[wasm_bindgen]
    pub fn set_click_handler(&mut self, handler: Function) -> Result<(), JsValue> {
        self.click_handler = Some(handler);
        self.render()
    }
    
    // Internal render method
    fn render(&self) -> Result<(), JsValue> {
        // Create the CSS for the component
        let style = self.create_style()?;
        
        // Create the HTML structure
        let content = self.create_content()?;
        
        // Clear the shadow root
        while let Some(child) = self.shadow_root.first_child() {
            self.shadow_root.remove_child(&child)?;
        }
        
        // Add the style and content to the shadow root
        self.shadow_root.append_child(&style)?;
        self.shadow_root.append_child(&content)?;
        
        Ok(())
    }
    
    // Create the component's CSS
    fn create_style(&self) -> Result<Element, JsValue> {
        let document = self.shadow_root.owner_document().expect("should have document");
        let style = document.create_element("style")?;
        
        let theme_vars = if self.properties.theme == "dark" {
            "--bg-color: #222; --text-color: #eee; --accent-color: #ff9500;"
        } else {
            "--bg-color: #fff; --text-color: #333; --accent-color: #0075c9;"
        };
        
        style.set_text_content(Some(&format!(
            r#"
            :host \{{
                display: block;
                {}
            \}}
            .component \{{
                background-color: var(--bg-color);
                color: var(--text-color);
                border-radius: 8px;
                padding: 20px;
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                font-family: sans-serif;
                transition: all 0.3s ease;
            \}}
            .title \{{
                color: var(--accent-color);
                margin-top: 0;
                margin-bottom: 10px;
            \}}
            .description \{{
                margin-bottom: 20px;
            \}}
            button \{{
                background-color: var(--accent-color);
                color: white;
                border: none;
                padding: 8px 16px;
                border-radius: 4px;
                cursor: pointer;
                font-size: 14px;
                transition: opacity 0.3s;
            \}}
            button:hover \{{
                opacity: 0.9;
            \}}
            "#,
            theme_vars
        )));
        
        Ok(style)
    }
    
    // Create the component's HTML content
    fn create_content(&self) -> Result<Element, JsValue> {
        let document = self.shadow_root.owner_document().expect("should have document");
        
        // Create the main container
        let container = document.create_element("div")?;
        container.set_class_name("component");
        
        // Create the title
        let title = document.create_element("h2")?;
        title.set_class_name("title");
        title.set_text_content(Some(&self.properties.title));
        container.append_child(&title)?;
        
        // Create the description
        let description = document.create_element("p")?;
        description.set_class_name("description");
        description.set_text_content(Some(&self.properties.description));
        container.append_child(&description)?;
        
        // Create a button if we have a click handler
        if let Some(handler) = &self.click_handler {
            let button = document.create_element("button")?;
            button.set_text_content(Some("Click Me"));
            
            // Clone the handler for the closure
            let handler_clone = handler.clone();
            
            // Create a Rust closure that will call the JS function
            let click_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                let _ = handler_clone.call0(&JsValue::NULL);
            }) as Box<dyn FnMut(_)>);
            
            // Set the click event
            let button_element = button.dyn_into::<HtmlElement>()?;
            button_element.set_onclick(Some(click_callback.as_ref().unchecked_ref()));
            
            // Forget the closure to keep it alive
            click_callback.forget();
            
            container.append_child(&button_element)?;
        }
        
        Ok(container)
    }
}

// Helper function to register the component
#[wasm_bindgen]
pub fn register_rust_component(name: &str) -> Result<(), JsValue> {
    let window: Window = web_sys::window().expect("no global `window` exists");
    let document: Document = window.document().expect("should have a document on window");
    
    // Create a template for usage instructions
    let template = document.create_element("template")?;
    template.set_id(&format!("{}-template", name));
    template.set_inner_html(&format!(
        r#"
        <div>
            <p>To use this component, import the WebAssembly module and call:</p>
            <pre>import init, \{{ register_rust_component \}} from './pkg/\{{\{{project_name\}}\}}.js';
await init();
register_rust_component('{}');</pre>
            <p>Then use it in your HTML:</p>
            <pre>&lt;{}&gt;&lt;/{}&gt;</pre>
        </div>
        "#,
        name, name, name
    ));
    
    // Append the template to the document body
    document.body().expect("document should have a body").append_child(&template)?;
    
    log(&format!("Registered Rust component: <{}>", name));
    Ok(())
}
