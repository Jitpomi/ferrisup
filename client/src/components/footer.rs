use dioxus::core_macro::{component, rsx};
use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::fa_brands_icons::FaRust};

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            class: "bg-gradient-to-b from-gray-900 to-black py-16 border-t border-amber-900/20",
            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                
                // Main footer content
                div {
                    class: "grid grid-cols-1 md:grid-cols-4 gap-10 mb-12",
                    
                    // Logo and description
                    div {
                        class: "md:col-span-2",
                        div {
                            class: "flex items-center mb-4",
                            Icon {
                                class: "w-8 h-8 text-amber-500 mr-2",
                                icon: FaRust,
                            }
                            h3 {
                                class: "text-xl font-bold text-white",
                                "FerrisUp"
                            }
                        }
                        p {
                            class: "text-gray-400 mb-6",
                            "A versatile Rust project bootstrapping tool that enables developers to create, transform, and scale Rust projects with ease."
                        }
                        
                        // Social links
                        div {
                            class: "flex space-x-4",
                            
                            // GitHub
                            a {
                                href: "https://github.com/Jitpomi/ferrisup",
                                target: "_blank",
                                class: "w-10 h-10 rounded-full bg-gray-800 flex items-center justify-center text-gray-400 hover:bg-amber-600 hover:text-white transition-colors duration-300",
                                aria_label: "GitHub",
                                
                                // GitHub icon
                                svg {
                                    class: "w-5 h-5",
                                    xmlns: "http://www.w3.org/2000/svg",
                                    view_box: "0 0 24 24",
                                    fill: "currentColor",
                                    path {
                                        d: "M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"
                                    }
                                }
                            }
                            
                            // Twitter/X
                            a {
                                href: "#",
                                class: "w-10 h-10 rounded-full bg-gray-800 flex items-center justify-center text-gray-400 hover:bg-amber-600 hover:text-white transition-colors duration-300",
                                aria_label: "Twitter",
                                
                                // Twitter/X icon
                                svg {
                                    class: "w-5 h-5",
                                    xmlns: "http://www.w3.org/2000/svg",
                                    view_box: "0 0 24 24",
                                    fill: "currentColor",
                                    path {
                                        d: "M24 4.557c-.883.392-1.832.656-2.828.775 1.017-.609 1.798-1.574 2.165-2.724-.951.564-2.005.974-3.127 1.195-.897-.957-2.178-1.555-3.594-1.555-3.179 0-5.515 2.966-4.797 6.045-4.091-.205-7.719-2.165-10.148-5.144-1.29 2.213-.669 5.108 1.523 6.574-.806-.026-1.566-.247-2.229-.616-.054 2.281 1.581 4.415 3.949 4.89-.693.188-1.452.232-2.224.084.626 1.956 2.444 3.379 4.6 3.419-2.07 1.623-4.678 2.348-7.29 2.04 2.179 1.397 4.768 2.212 7.548 2.212 9.142 0 14.307-7.721 13.995-14.646.962-.695 1.797-1.562 2.457-2.549z"
                                    }
                                }
                            }
                            
                            // Discord
                            a {
                                href: "#",
                                class: "w-10 h-10 rounded-full bg-gray-800 flex items-center justify-center text-gray-400 hover:bg-amber-600 hover:text-white transition-colors duration-300",
                                aria_label: "Discord",
                                
                                // Discord icon
                                svg {
                                    class: "w-5 h-5",
                                    xmlns: "http://www.w3.org/2000/svg",
                                    view_box: "0 0 24 24",
                                    fill: "currentColor",
                                    path {
                                        d: "M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.608 1.2495-1.8447-.2762-3.6677-.2762-5.4878 0-.1634-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"
                                    }
                                }
                            }
                        }
                    }
                    
                    // Quick links
                    div {
                        class: "md:col-span-1",
                        h4 {
                            class: "text-lg font-semibold text-white mb-4",
                            "Quick Links"
                        }
                        ul {
                            class: "space-y-2",
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Documentation"
                                }
                            }
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Getting Started"
                                }
                            }
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Examples"
                                }
                            }
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Templates"
                                }
                            }
                        }
                    }
                    
                    // Resources
                    div {
                        class: "md:col-span-1",
                        h4 {
                            class: "text-lg font-semibold text-white mb-4",
                            "Resources"
                        }
                        ul {
                            class: "space-y-2",
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Community"
                                }
                            }
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Blog"
                                }
                            }
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Changelog"
                                }
                            }
                            li {
                                a {
                                    href: "#",
                                    class: "text-gray-400 hover:text-amber-400 transition-colors duration-300",
                                    "Report a Bug"
                                }
                            }
                        }
                    }
                }
                
                // Divider
                div {
                    class: "h-px bg-gradient-to-r from-transparent via-amber-800/30 to-transparent mb-8"
                }
                
                // Copyright and credits
                div {
                    class: "flex flex-col md:flex-row justify-between items-center",
                    p {
                        class: "text-gray-500 mb-4 md:mb-0",
                        "© 2025 FerrisUp. Built with Rust and Dioxus."
                    }
                    
                    // Version badge
                    div {
                        class: "flex items-center space-x-4",
                        span {
                            class: "px-2 py-1 text-xs font-medium bg-amber-900/20 text-amber-400 rounded-md border border-amber-700/30",
                            "v0.1.23"
                        }
                        
                        a {
                            href: "https://crates.io/crates/ferrisup",
                            target: "_blank",
                            class: "text-gray-400 hover:text-amber-400 transition-colors duration-300 text-sm",
                            "View on crates.io"
                        }
                    }
                }
            }
        }
    }
}
