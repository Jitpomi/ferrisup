use dioxus::prelude::*;

// Import the LinkType from shared
use shared::LinkType;


#[component]
pub fn Test() -> Element {
    rsx! {
        // Main container with Tailwind classes
        div {
            id:"test-container",
            class: "p-8 flex flex-col items-center gap-4 bg-gray-100 min-h-screen",
            // Test div 1 - Basic dimensions
            div {
                id:"test-1",
                class: "h-[40px] w-[460px] flex items-center justify-center bg-white rounded-md shadow",
                "Dimensions (h-[40px] w-[460px])"
            }
            
            // Test div 2 - Background color
            div {
                id:"test-2",
                class: "h-[40px] w-[460px] bg-blue-500 flex items-center justify-center text-white font-medium rounded-md",
                "Background (bg-blue-500)"
            }
            
            // Test div 3 - Border
            div {
                id:"test-3",
                class: "h-[40px] w-[460px] border-2 border-red-500 flex items-center justify-center rounded-md",
                "Border (border-2 border-red-500)"
            }
            
            // Test div 4 - Border radius
            div {
                id:"test-4",
                class: "h-[40px] w-[460px] bg-gray-200 rounded-xl flex items-center justify-center",
                "Border radius (rounded-xl)"
            }
            
            // Test div 5 - Shadow
            div {
                id:"test-5",
                class: "h-[40px] w-[460px] bg-white shadow-lg flex items-center justify-center rounded-md",
                "Shadow (shadow-lg)"
            }
            
            // Test div 6 - Hover and transition effects
            div {
                id:"test-6",
                class: "h-[40px] w-[460px] bg-green-500 hover:bg-green-600 transition-colors duration-300 flex items-center justify-center text-white rounded-md",
                "Hover effect (hover:bg-green-600)"
            }
            
            // Test div 7 - Gradient
            div {
                id:"test-7",
                class: "h-[40px] w-[460px] bg-gradient-to-r from-purple-500 to-pink-500 flex items-center justify-center text-white rounded-md",
                "Gradient (from-purple-500 to-pink-500)"
            }
        }
    }
}

