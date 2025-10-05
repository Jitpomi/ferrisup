use dioxus::prelude::*;

#[component]
pub fn Community() -> Element {
    rsx! {
        section {
            class: "py-20 bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900",
            id: "community",
            
            div {
                class: "container mx-auto px-6",
                
                // Section header
                div {
                    class: "text-center mb-16",
                    
                    h2 {
                        class: "text-4xl md:text-5xl font-bold text-white mb-6",
                        "Join Our Community"
                    }
                    
                    p {
                        class: "text-xl text-gray-300 max-w-3xl mx-auto leading-relaxed",
                        "Connect with fellow Rust developers, share your projects, and stay updated with the latest FerrisUp developments."
                    }
                }
                
                // Community content grid
                div {
                    class: "grid lg:grid-cols-2 gap-12 items-start",
                    
                    // LinkedIn embed section
                    div {
                        class: "bg-gray-800/50 backdrop-blur-sm rounded-2xl p-8 border border-gray-700/50",
                        
                        h3 {
                            class: "text-2xl font-bold text-white mb-6 text-center",
                            "Latest Updates"
                        }
                        
                        div {
                            class: "flex justify-center",
                            
                            // LinkedIn embed iframe
                            iframe {
                                src: "https://www.linkedin.com/embed/feed/update/urn:li:share:7376174519444516864?collapsed=1",
                                height: "603",
                                width: "504",
                                frame_border: "0",
                                allowfullscreen: "true",
                                title: "FerrisUp LinkedIn Update",
                                class: "rounded-lg shadow-2xl max-w-full"
                            }
                        }
                    }
                    
                    // Community links and info
                    div {
                        class: "space-y-8",
                        
                        // GitHub section
                        div {
                            class: "bg-gray-800/50 backdrop-blur-sm rounded-2xl p-8 border border-gray-700/50",
                            
                            div {
                                class: "flex items-center mb-4",
                                
                                svg {
                                    class: "w-8 h-8 text-white mr-4",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"
                                    }
                                }
                                
                                h4 {
                                    class: "text-xl font-bold text-white",
                                    "Contribute on GitHub"
                                }
                            }
                            
                            p {
                                class: "text-gray-300 mb-6",
                                "Help improve FerrisUp by contributing code, reporting issues, or suggesting new features."
                            }
                            
                            a {
                                href: "https://github.com/Jitpomi/ferrisup",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "inline-flex items-center px-6 py-3 bg-gray-700 hover:bg-gray-600 text-white font-semibold rounded-lg transition-all duration-300 transform hover:scale-105",
                                
                                span { "View on GitHub" }
                                
                                svg {
                                    class: "w-4 h-4 ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                                    }
                                }
                            }
                        }
                        
                        // Crates.io section
                        div {
                            class: "bg-gray-800/50 backdrop-blur-sm rounded-2xl p-8 border border-gray-700/50",
                            
                            div {
                                class: "flex items-center mb-4",
                                
                                div {
                                    class: "w-8 h-8 bg-orange-500 rounded-lg flex items-center justify-center mr-4",
                                    
                                    svg {
                                        class: "w-5 h-5 text-white",
                                        fill: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"
                                        }
                                    }
                                }
                                
                                h4 {
                                    class: "text-xl font-bold text-white",
                                    "Install from Crates.io"
                                }
                            }
                            
                            p {
                                class: "text-gray-300 mb-6",
                                "Get the latest stable release of FerrisUp directly from the official Rust package registry."
                            }
                            
                            a {
                                href: "https://crates.io/crates/ferrisup",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "inline-flex items-center px-6 py-3 bg-orange-600 hover:bg-orange-500 text-white font-semibold rounded-lg transition-all duration-300 transform hover:scale-105",
                                
                                span { "View on Crates.io" }
                                
                                svg {
                                    class: "w-4 h-4 ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                                    }
                                }
                            }
                        }
                        
                        // Discord community
                        div {
                            class: "bg-gray-800/50 backdrop-blur-sm rounded-2xl p-8 border border-gray-700/50",
                            
                            div {
                                class: "flex items-center mb-4",
                                
                                div {
                                    class: "w-8 h-8 bg-indigo-500 rounded-lg flex items-center justify-center mr-4",
                                    
                                    svg {
                                        class: "w-5 h-5 text-white",
                                        fill: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419-.0002 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1568 2.4189Z"
                                        }
                                    }
                                }
                                
                                h4 {
                                    class: "text-xl font-bold text-white",
                                    "Join Our Discord"
                                }
                            }
                            
                            p {
                                class: "text-gray-300 mb-6",
                                "Connect with fellow Rust developers, get help with FerrisUp, share your projects, and participate in community discussions."
                            }
                            
                            a {
                                href: "https://discord.gg/P3h7bkUR",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "inline-flex items-center px-6 py-3 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-lg transition-all duration-300 transform hover:scale-105",
                                
                                svg {
                                    class: "w-5 h-5 mr-3",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419-.0002 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1568 2.4189Z"
                                    }
                                }
                                
                                span { "Join Discord Community" }
                                
                                svg {
                                    class: "w-4 h-4 ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
