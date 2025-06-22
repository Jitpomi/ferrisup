#!/bin/bash

set -e

# Default configuration
CONFIG_FILE="config.json"
TEMPLATE="full-stack"
SKIP_GIT=false
PROJECT_NAME=""
MINIMAL_PROJECT=false
SCALE_PROJECT=false

# Error handling
error_exit() {
    echo "Error: $1" >&2
    exit 1
}

# Log with color
log_info() {
    echo -e "\033[0;34m[INFO]\033[0m $1"
}

log_success() {
    echo -e "\033[0;32m[SUCCESS]\033[0m $1"
}

log_warning() {
    echo -e "\033[0;33m[WARNING]\033[0m $1"
}

log_error() {
    echo -e "\033[0;31m[ERROR]\033[0m $1"
}

# Show help
show_help() {
    echo "FerrisUp - Flexible Rust workspace initializer"
    echo ""
    echo "Usage: ./ferrisup.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help                Show this help message"
    echo "  -c, --config FILE         Use config file (default: config.json)"
    echo "  -t, --template NAME       Select template (default: from config)"
    echo "  -n, --name NAME           Set project name (default: from config)"
    echo "  --skip-git                Skip Git initialization"
    echo "  --list-templates          List available component types"
    echo "  --minimal                 Create a minimal project (hello world)"
    echo "  --transform=TEMPLATE      Transform existing project to new template"
    echo "  --project=PATH            Specify project path for transformation"
    echo ""
    echo "Examples:"
    echo "  ./ferrisup.sh                       # Use default config.json"
    echo "  ./ferrisup.sh -t backend-only       # Create backend-only project"
    echo "  ./ferrisup.sh -n myproject          # Custom project name"
    echo "  ./ferrisup.sh --minimal             # Create minimal hello world project"
    echo ""
    exit 0
}

# Parse command-line arguments
parse_args() {
    CONFIG_FILE="config.json"
    MINIMAL_PROJECT=false
    SCALE_PROJECT=false
    TRANSFORM_TARGET=""
    TRANSFORM_PROJECT=""
    
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                show_help ;;
            --list-templates)
                echo "Available templates:"
                echo "  full-stack    - Complete client, server and libraries"
                echo "  backend-only  - Server and libraries only"
                echo "  frontend-only - Client and libraries only"
                echo "  library       - Only library components"
                echo "  minimal       - Minimal hello world project"
                echo "  ai            - AI components"
                echo "  edge          - Edge computing components"
                echo "  embedded      - Embedded systems components"
                exit 0 ;;
            --minimal) MINIMAL_PROJECT=true; shift ;;
            --scale) SCALE_PROJECT=true; shift ;;
            --transform=*)
                TRANSFORM_TARGET="${1#*=}"
                shift ;;
            --project=*)
                TRANSFORM_PROJECT="${1#*=}"
                shift ;;
            -c|--config)
                CONFIG_FILE="$2"; shift 2 ;;
            -n|--name)
                PROJECT_NAME="$2"; shift 2 ;;
            -t|--template)
                TEMPLATE="$2"; shift 2 ;;
            *)
                if [ -z "$PROJECT_NAME" ]; then
                    PROJECT_NAME="$1"
                    shift
                else
                    log_error "Unknown option: $1"
                    exit 1
                fi ;;
        esac
    done
    
    # Check if transformation requested
    if [ -n "$TRANSFORM_TARGET" ]; then
        if [ -z "$TRANSFORM_PROJECT" ]; then
            log_error "Project path must be specified with --project option when using --transform"
            exit 1
        fi
        
        # Check if template exists
        if ! jq -e --arg template "$TRANSFORM_TARGET" '.templates[$template]' "$CONFIG_FILE" > /dev/null; then
            log_error "Template '$TRANSFORM_TARGET' does not exist in config"
            exit 1
        fi
        
        # Perform transformation
        transform_project "$TRANSFORM_PROJECT" "$TRANSFORM_TARGET"
        exit 0
    fi
}

# Process configuration
process_config() {
    # Read basic project configuration
    PROJECT_NAME=$(jq -r '.project_name // "rust_workspace"' "$CONFIG_FILE")
    TEMPLATE=$(jq -r '.template // "full-stack"' "$CONFIG_FILE")
    
    if [ -n "$PROJECT_NAME_OVERRIDE" ]; then
        PROJECT_NAME="$PROJECT_NAME_OVERRIDE"
    fi
    
    if [ -n "$TEMPLATE_OVERRIDE" ]; then
        TEMPLATE="$TEMPLATE_OVERRIDE"
    fi

    if [ "$MINIMAL_PROJECT" = true ]; then
        TEMPLATE="minimal"
    fi
    
    log_info "Project name: $PROJECT_NAME"
    log_info "Template: $TEMPLATE"
    
    # Determine active components based on template
    ACTIVE_COMPONENTS=($(jq -r ".templates[\"$TEMPLATE\"] | .[]" "$CONFIG_FILE"))
    log_info "Active components: ${ACTIVE_COMPONENTS[*]}"
    
    # Process component configurations
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " client " ]]; then
        CLIENT_APPS=($(jq -r '.components.client.apps | .[]' "$CONFIG_FILE"))
        CLIENT_FRAMEWORKS=($(jq -r '.components.client.frameworks | .[]' "$CONFIG_FILE"))
        
        # Validate number of frameworks matches number of apps
        if [ ${#CLIENT_APPS[@]} -ne ${#CLIENT_FRAMEWORKS[@]} ]; then
            log_error "Number of client apps (${#CLIENT_APPS[@]}) does not match number of frameworks (${#CLIENT_FRAMEWORKS[@]})"
            exit 1
        fi
    fi
    
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " server " ]]; then
        SERVER_SERVICES=($(jq -r '.components.server.services | .[]' "$CONFIG_FILE"))
        SERVER_FRAMEWORKS=($(jq -r '.components.server.frameworks | .[]' "$CONFIG_FILE"))
        
        # Validate number of services matches number of frameworks
        if [ ${#SERVER_SERVICES[@]} -ne ${#SERVER_FRAMEWORKS[@]} ]; then
            log_error "Number of server services (${#SERVER_SERVICES[@]}) does not match number of frameworks (${#SERVER_FRAMEWORKS[@]})"
            exit 1
        fi
    fi
    
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " database " ]]; then
        DATABASE_ENABLED=$(jq -r '.components.database.enabled // false' "$CONFIG_FILE")
        DATABASE_ENGINES=($(jq -r '.components.database.engines | .[]' "$CONFIG_FILE"))
        DATABASE_MIGRATION_TOOL=$(jq -r '.components.database.migration_tool // "sqlx"' "$CONFIG_FILE")
    fi
    
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " libs " ]]; then
        LIBS_MODULES=($(jq -r '.components.libs.modules | .[]' "$CONFIG_FILE"))
    fi
    
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " binaries " ]]; then
        BINARY_APPS=($(jq -r '.components.binaries.apps | .[] // "hello"' "$CONFIG_FILE"))
    fi
    
    if [ "$SCALE_PROJECT" = true ]; then
        DOCKER_ENABLED=$(jq -r '.scaling_options.docker // false' "$CONFIG_FILE")
        K8S_ENABLED=$(jq -r '.scaling_options.kubernetes // false' "$CONFIG_FILE")
        CI_CD_ENABLED=$(jq -r '.scaling_options.ci_cd // false' "$CONFIG_FILE")
        DEPLOYMENT_TARGETS=($(jq -r '.scaling_options.deployment | keys | .[]' "$CONFIG_FILE"))
    fi
    
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " ai " ]]; then
        process_ai_config
    fi
    
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " edge " ]]; then
        process_edge_config
    fi
    
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " embedded " ]]; then
        process_embedded_config
    fi
}

# Process AI configuration
process_ai_config() {
    AI_MODELS=($(jq -r '.components.ai.models | .[]' "$CONFIG_FILE"))
    AI_BACKENDS=($(jq -r '.components.ai.backends | .[]' "$CONFIG_FILE"))
    AI_FEATURES=($(jq -r '.components.ai.features | .[]' "$CONFIG_FILE"))
}

# Process edge computing configuration
process_edge_config() {
    EDGE_TARGETS=($(jq -r '.components.edge.targets | .[]' "$CONFIG_FILE"))
    EDGE_FEATURES=($(jq -r '.components.edge.features | .[]' "$CONFIG_FILE"))
}

# Process embedded systems configuration
process_embedded_config() {
    EMBEDDED_TARGETS=($(jq -r '.components.embedded.targets | .[]' "$CONFIG_FILE"))
    EMBEDDED_FEATURES=($(jq -r '.components.embedded.features | .[]' "$CONFIG_FILE"))
}

# Create AI components
create_ai_components() {
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " ai " ]]; then
        log_info "Generating AI components..."
        
        # Create AI directory structure
        mkdir -p "$PROJECT_NAME/ai/models"
        mkdir -p "$PROJECT_NAME/ai/pipelines"
        mkdir -p "$PROJECT_NAME/ai/examples"
        
        # Create AI module Cargo.toml
        cat > "$PROJECT_NAME/ai/Cargo.toml" <<EOL
[package]
name = "ai"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
EOL
        
        # Add selected AI backends
        if [[ " ${AI_BACKENDS[@]} " =~ " candle " ]]; then
            echo "candle-core = { version = \"0.3\", features = [\"metal\"] }" >> "$PROJECT_NAME/ai/Cargo.toml"
            echo "candle-nn = { version = \"0.3\" }" >> "$PROJECT_NAME/ai/Cargo.toml"
        fi
        
        if [[ " ${AI_BACKENDS[@]} " =~ " ort " ]]; then
            echo "ort = { version = \"1.16\", features = [\"download-binaries\"] }" >> "$PROJECT_NAME/ai/Cargo.toml"
        fi
        
        if [[ " ${AI_BACKENDS[@]} " =~ " tch " ]]; then
            echo "tch = { version = \"0.13\" }" >> "$PROJECT_NAME/ai/Cargo.toml"
        fi
        
        # Add selected AI models
        if [[ " ${AI_MODELS[@]} " =~ " llama " ]]; then
            echo "llm = { version = \"0.1\", features = [\"cublas\"] }" >> "$PROJECT_NAME/ai/Cargo.toml"
        fi
        
        if [[ " ${AI_MODELS[@]} " =~ " whisper " ]]; then
            echo "whisper-rs = { version = \"0.8\" }" >> "$PROJECT_NAME/ai/Cargo.toml"
        fi
        
        if [[ " ${AI_MODELS[@]} " =~ " stable-diffusion " ]]; then
            echo "diffusers-rs = { version = \"0.3\" }" >> "$PROJECT_NAME/ai/Cargo.toml"
        fi
        
        # Create AI component source files
        cat > "$PROJECT_NAME/ai/src/lib.rs" <<EOL
//! AI module for $PROJECT_NAME
//! 
//! This module provides AI capabilities including:
//! - Text generation
//! - Speech recognition
//! - Image generation
//! - Embeddings

pub mod model;
pub mod pipeline;
pub mod utils;

pub use model::Model;
pub use pipeline::Pipeline;
EOL

        mkdir -p "$PROJECT_NAME/ai/src/model"
        cat > "$PROJECT_NAME/ai/src/model/mod.rs" <<EOL
//! AI models implementation

use std::path::Path;

/// Generic AI model trait
pub trait Model {
    type Input;
    type Output;
    type Error;
    
    /// Load a model from a path
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error> where Self: Sized;
    
    /// Run inference with the model
    fn infer(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
EOL

        mkdir -p "$PROJECT_NAME/ai/src/pipeline"
        cat > "$PROJECT_NAME/ai/src/pipeline/mod.rs" <<EOL
//! AI pipelines implementation

/// Generic pipeline trait
pub trait Pipeline {
    type Input;
    type Output;
    type Error;
    
    /// Process input through the pipeline
    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
EOL

        mkdir -p "$PROJECT_NAME/ai/src/utils"
        cat > "$PROJECT_NAME/ai/src/utils/mod.rs" <<EOL
//! Utility functions for AI components

/// Load a tokenizer for text processing
pub fn load_tokenizer(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation would depend on the tokenizer library
    Ok(())
}

/// Download a model from a hub
pub fn download_model(model_id: &str, target_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading model {} to {}", model_id, target_path);
    // Implementation would use reqwest or similar to download
    Ok(())
}
EOL

        # Create example for text generation if selected
        if [[ " ${AI_FEATURES[@]} " =~ " text-generation " ]]; then
            mkdir -p "$PROJECT_NAME/ai/examples/text-generation"
            cat > "$PROJECT_NAME/ai/examples/text-generation/main.rs" <<EOL
//! Text generation example

use ai::model::Model;
use ai::pipeline::Pipeline;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading text generation model...");
    
    // Example code to load and run a text generation model
    let prompt = "Once upon a time";
    println!("Generating text from prompt: {}", prompt);
    
    // In a real implementation, this would use the actual model
    println!("Generated: {} in a land far away, there lived a programmer who loved Rust...", prompt);
    
    Ok(())
}
EOL
        fi

        # Create example for speech recognition if selected
        if [[ " ${AI_FEATURES[@]} " =~ " speech-to-text " ]]; then
            mkdir -p "$PROJECT_NAME/ai/examples/speech-to-text"
            cat > "$PROJECT_NAME/ai/examples/speech-to-text/main.rs" <<EOL
//! Speech to text example

use ai::model::Model;
use ai::pipeline::Pipeline;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading speech recognition model...");
    
    // Example code to load and run a speech recognition model
    let audio_path = "sample.wav";
    println!("Transcribing audio from: {}", audio_path);
    
    // In a real implementation, this would use the actual model
    println!("Transcription: Hello, world from Ferris Up!");
    
    Ok(())
}
EOL
        fi

        # Create .env file for AI configuration
        cat > "$PROJECT_NAME/ai/.env.example" <<EOL
# AI Configuration
MODEL_PATH=./models
DEVICE=cpu  # or cuda, metal
TOKENIZER_PATH=./tokenizers
EOL
    fi
}

# Create edge computing components
create_edge_components() {
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " edge " ]]; then
        log_info "Generating edge computing components..."
        
        # Create edge directory structure
        mkdir -p "$PROJECT_NAME/edge/functions"
        mkdir -p "$PROJECT_NAME/edge/workers"
        
        # Create edge module Cargo.toml
        cat > "$PROJECT_NAME/edge/Cargo.toml" <<EOL
[package]
name = "edge"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOL
        
        # Add selected edge targets
        if [[ " ${EDGE_TARGETS[@]} " =~ " wasm " ]]; then
            echo "wasm-bindgen = { version = \"0.2\" }" >> "$PROJECT_NAME/edge/Cargo.toml"
        fi
        
        if [[ " ${EDGE_TARGETS[@]} " =~ " cloudflare-workers " ]]; then
            echo "worker = { version = \"0.0.15\" }" >> "$PROJECT_NAME/edge/Cargo.toml"
        fi
        
        # Create hello world function for edge
        cat > "$PROJECT_NAME/edge/functions/hello.js" <<EOL
// Example edge function
export default function handler(request, context) {
  return new Response("Hello from FerrisUp Edge Functions!", {
    headers: { "content-type": "text/plain" }
  });
}
EOL

        # Create Cloudflare worker if selected
        if [[ " ${EDGE_TARGETS[@]} " =~ " cloudflare-workers " ]]; then
            cat > "$PROJECT_NAME/edge/workers/worker.js" <<EOL
// Cloudflare Worker example
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
  return new Response("Hello from FerrisUp Cloudflare Worker!", {
    headers: { "content-type": "text/plain" }
  });
}
EOL
        fi
    fi
}

# Create embedded systems components
create_embedded_components() {
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " embedded " ]]; then
        log_info "Generating embedded systems components..."
        
        # Create embedded directory structure
        mkdir -p "$PROJECT_NAME/embedded/src"
        mkdir -p "$PROJECT_NAME/embedded/memory.x"
        
        # Create embedded module Cargo.toml
        cat > "$PROJECT_NAME/embedded/Cargo.toml" <<EOL
[package]
name = "embedded"
version = "0.1.0"
edition = "2021"

[dependencies]
embedded-hal = "1.0.0"
panic-halt = "0.2.0"

# Disable standard library for embedded targets
[features]
default = []
std = []
EOL
        
        # Add selected embedded targets
        if [[ " ${EMBEDDED_TARGETS[@]} " =~ " rp2040 " ]]; then
            echo "rp2040-hal = { version = \"0.9\" }" >> "$PROJECT_NAME/embedded/Cargo.toml"
        fi
        
        if [[ " ${EMBEDDED_TARGETS[@]} " =~ " esp32 " ]]; then
            echo "esp32-hal = { version = \"0.14\" }" >> "$PROJECT_NAME/embedded/Cargo.toml"
        fi
        
        # Create basic embedded example
        cat > "$PROJECT_NAME/embedded/src/main.rs" <<EOL
//! Embedded system example for $PROJECT_NAME
//! 
//! This is a simple "blink LED" example that works with various targets

#![no_std]
#![no_main]

// Import panic handler
use panic_halt as _;

// Platform-specific code will be linked based on target
#[entry]
fn main() -> ! {
    // Initialize the hardware
    let peripherals = setup_peripherals();
    
    // Configure LED pin as output
    let mut led = setup_led(peripherals);
    
    // Main loop
    loop {
        // Turn LED on
        led_on(&mut led);
        delay(500);
        
        // Turn LED off
        led_off(&mut led);
        delay(500);
    }
}

// Platform-specific functions would be implemented in separate modules
// based on the chosen target architecture
EOL
    fi
}

# Transform existing project to a different template
transform_project() {
    local PROJECT_DIR="$1"
    local TARGET_TEMPLATE="$2"
    
    # Validate inputs
    if [ ! -d "$PROJECT_DIR" ]; then
        log_error "Project directory $PROJECT_DIR does not exist"
        exit 1
    fi
    
    # Check if this appears to be a FerrisUp project
    if [ ! -f "$PROJECT_DIR/Cargo.toml" ]; then
        log_error "Project directory $PROJECT_DIR does not appear to be a Rust workspace"
        exit 1
    fi
    
    log_info "Analyzing existing project structure..."
    
    # Detect existing components
    local HAS_CLIENT=false
    local HAS_SERVER=false
    local HAS_DATABASE=false
    local HAS_LIBS=false
    local HAS_BINARIES=false
    local HAS_AI=false
    local HAS_EDGE=false
    local HAS_EMBEDDED=false
    
    # Check for existing components
    if [ -d "$PROJECT_DIR/client" ]; then HAS_CLIENT=true; fi
    if [ -d "$PROJECT_DIR/server" ]; then HAS_SERVER=true; fi
    if [ -d "$PROJECT_DIR/database" ]; then HAS_DATABASE=true; fi
    if [ -d "$PROJECT_DIR/libs" ]; then HAS_LIBS=true; fi
    if [ -d "$PROJECT_DIR/binaries" ]; then HAS_BINARIES=true; fi
    if [ -d "$PROJECT_DIR/ai" ]; then HAS_AI=true; fi
    if [ -d "$PROJECT_DIR/edge" ]; then HAS_EDGE=true; fi
    if [ -d "$PROJECT_DIR/embedded" ]; then HAS_EMBEDDED=true; fi
    
    # Get target template components
    local TARGET_COMPONENTS=($(jq -r --arg template "$TARGET_TEMPLATE" '.templates[$template] | .[]' "$CONFIG_FILE"))
    
    log_info "Transforming project to template: $TARGET_TEMPLATE"
    log_info "Current components: ${ACTIVE_COMPONENTS[*]}"
    log_info "Target components: ${TARGET_COMPONENTS[*]}"
    
    # Determine components to add
    local COMPONENTS_TO_ADD=()
    for component in "${TARGET_COMPONENTS[@]}"; do
        case "$component" in
            "client")
                if [ "$HAS_CLIENT" = false ]; then
                    COMPONENTS_TO_ADD+=("client")
                fi
                ;;
            "server")
                if [ "$HAS_SERVER" = false ]; then
                    COMPONENTS_TO_ADD+=("server")
                fi
                ;;
            "database")
                if [ "$HAS_DATABASE" = false ]; then
                    COMPONENTS_TO_ADD+=("database")
                fi
                ;;
            "libs")
                if [ "$HAS_LIBS" = false ]; then
                    COMPONENTS_TO_ADD+=("libs")
                fi
                ;;
            "binaries")
                if [ "$HAS_BINARIES" = false ]; then
                    COMPONENTS_TO_ADD+=("binaries")
                fi
                ;;
            "ai")
                if [ "$HAS_AI" = false ]; then
                    COMPONENTS_TO_ADD+=("ai")
                fi
                ;;
            "edge")
                if [ "$HAS_EDGE" = false ]; then
                    COMPONENTS_TO_ADD+=("edge")
                fi
                ;;
            "embedded")
                if [ "$HAS_EMBEDDED" = false ]; then
                    COMPONENTS_TO_ADD+=("embedded")
                fi
                ;;
        esac
    done
    
    log_info "Components to add: ${COMPONENTS_TO_ADD[*]}"
    
    # Special handling for minimal to library conversion
    if [ "$HAS_BINARIES" = true ] && [[ " ${COMPONENTS_TO_ADD[@]} " =~ " libs " ]]; then
        log_info "Converting binary to library..."
        
        # Check if binary has main.rs
        if [ -f "$PROJECT_DIR/binaries/cli/src/main.rs" ]; then
            # Create libs directory
            mkdir -p "$PROJECT_DIR/libs/core/src"
            
            # Copy content from binary to lib, transforming appropriately
            awk '
            BEGIN { skip_next = 0; fn_main_found = 0; } 
            /fn main\(\)/ { 
                fn_main_found = 1; 
                print "//! Core library functionality converted from binary"; 
                print ""; 
                print "/// Runs the main application logic"; 
                print "pub fn run() {"; 
                skip_next = 1; 
                next; 
            } 
            /^}$/ { 
                if (fn_main_found) { 
                    print "}"; 
                    fn_main_found = 0; 
                } else { 
                    print $0; 
                } 
                next; 
            } 
            { 
                if (skip_next) { 
                    skip_next = 0; 
                } else { 
                    print $0; 
                } 
            }' "$PROJECT_DIR/binaries/cli/src/main.rs" > "$PROJECT_DIR/libs/core/src/lib.rs"
            
            # Create a new main.rs that uses the library
            cat > "$PROJECT_DIR/binaries/cli/src/main.rs" <<EOL
//! Command-line interface that uses the core library

fn main() {
    // Initialize the library
    core::run();
}
EOL
            
            # Create Cargo.toml for the library
            cat > "$PROJECT_DIR/libs/core/Cargo.toml" <<EOL
[package]
name = "core"
version = "0.1.0"
edition = "2021"

[dependencies]
EOL
            
            # Update the binary's Cargo.toml to depend on the library
            awk '
            /^\[dependencies\]/ { 
                print $0; 
                print "core = { path = \"../../libs/core\" }"; 
                next; 
            } 
            { print $0; }' "$PROJECT_DIR/binaries/cli/Cargo.toml" > "$PROJECT_DIR/binaries/cli/Cargo.toml.new"
            mv "$PROJECT_DIR/binaries/cli/Cargo.toml.new" "$PROJECT_DIR/binaries/cli/Cargo.toml"
        fi
    fi
    
    # Add each missing component
    for component in "${COMPONENTS_TO_ADD[@]}"; do
        log_info "Adding component: $component"
        
        case "$component" in
            "client")
                # Set up client component
                mkdir -p "$PROJECT_DIR/client"
                local CLIENT_APPS=($(jq -r '.components.client.apps | .[]' "$CONFIG_FILE"))
                local CLIENT_FRAMEWORKS=($(jq -r '.components.client.frameworks | .[]' "$CONFIG_FILE"))
                
                # Create client apps
                for ((i=0; i<${#CLIENT_APPS[@]}; i++)); do
                    local app="${CLIENT_APPS[$i]}"
                    local framework="${CLIENT_FRAMEWORKS[$i]}"
                    mkdir -p "$PROJECT_DIR/client/$app"
                    # Create app-specific configuration (similar to create_client_apps function)
                    # ... (code from create_client_apps)
                done
                ;;
                
            "server")
                # Set up server component
                mkdir -p "$PROJECT_DIR/server"
                local SERVER_SERVICES=($(jq -r '.components.server.services | .[]' "$CONFIG_FILE"))
                local SERVER_FRAMEWORKS=($(jq -r '.components.server.frameworks | .[]' "$CONFIG_FILE"))
                
                # Create server services
                for ((i=0; i<${#SERVER_SERVICES[@]}; i++)); do
                    local service="${SERVER_SERVICES[$i]}"
                    local framework="${SERVER_FRAMEWORKS[$i]}"
                    mkdir -p "$PROJECT_DIR/server/$service"
                    # Create service-specific configuration (similar to create_server_services function)
                    # ... (code from create_server_services)
                done
                ;;
                
            "database")
                # Set up database component
                mkdir -p "$PROJECT_DIR/database"
                # Create database configuration (similar to create_database function)
                # ... (code from create_database)
                ;;
                
            "libs")
                # Set up libs component
                mkdir -p "$PROJECT_DIR/libs"
                local LIB_MODULES=($(jq -r '.components.libs.modules | .[]' "$CONFIG_FILE"))
                
                # Create lib modules
                for module in "${LIB_MODULES[@]}"; do
                    mkdir -p "$PROJECT_DIR/libs/$module/src"
                    # Create module-specific configuration (similar to create_libs function)
                    # ... (code from create_libs)
                done
                ;;
                
            "binaries")
                # Set up binaries component
                mkdir -p "$PROJECT_DIR/binaries"
                local BINARY_APPS=($(jq -r '.components.binaries.apps | .[]' "$CONFIG_FILE"))
                
                # Create binary apps
                for app in "${BINARY_APPS[@]}"; do
                    mkdir -p "$PROJECT_DIR/binaries/$app/src"
                    # Create app-specific configuration (similar to create_binary_apps function)
                    # ... (code from create_binary_apps)
                done
                ;;
                
            "ai")
                # Set up AI component
                mkdir -p "$PROJECT_DIR/ai"
                # Create AI configuration (similar to create_ai_components function)
                # ... (code from create_ai_components)
                ;;
                
            "edge")
                # Set up edge component
                mkdir -p "$PROJECT_DIR/edge"
                # Create edge configuration (similar to create_edge_components function)
                # ... (code from create_edge_components)
                ;;
                
            "embedded")
                # Set up embedded component
                mkdir -p "$PROJECT_DIR/embedded"
                # Create embedded configuration (similar to create_embedded_components function)
                # ... (code from create_embedded_components)
                ;;
        esac
    done
    
    # Update workspace Cargo.toml to include new members
    local WORKSPACE_TOML="$PROJECT_DIR/Cargo.toml"
    local TMP_TOML="$PROJECT_DIR/Cargo.toml.new"
    
    # Extract the current workspace members
    local CURRENT_MEMBERS=$(grep -A 100 '\[workspace\]' "$WORKSPACE_TOML" | grep -A 100 'members' | grep -v '^\[' | grep -v 'members' | sed -e 's/,$//' -e 's/^[ \t]*//' -e 's/"//g' | grep -v '^$')
    
    # Determine new members to add
    local NEW_MEMBERS=()
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " client " ]]; then
        for app in "${CLIENT_APPS[@]}"; do
            NEW_MEMBERS+=("\"client/$app\"")
        done
    fi
    
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " server " ]]; then
        for service in "${SERVER_SERVICES[@]}"; do
            NEW_MEMBERS+=("\"server/$service\"")
        done
    fi
    
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " database " ]]; then
        NEW_MEMBERS+=("\"database\"")
    fi
    
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " libs " ]]; then
        for module in "${LIB_MODULES[@]}"; do
            NEW_MEMBERS+=("\"libs/$module\"")
        done
    fi
    
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " binaries " ]]; then
        for app in "${BINARY_APPS[@]}"; do
            NEW_MEMBERS+=("\"binaries/$app\"")
        done
    fi
    
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " ai " ]]; then
        NEW_MEMBERS+=("\"ai\"")
    fi
    
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " edge " ]]; then
        NEW_MEMBERS+=("\"edge\"")
    fi
    
    if [[ " ${COMPONENTS_TO_ADD[@]} " =~ " embedded " ]]; then
        NEW_MEMBERS+=("\"embedded\"")
    fi
    
    # Create updated Cargo.toml with new members
    awk -v new_members="$(printf ",%s" "${NEW_MEMBERS[@]}")" '
    BEGIN { in_members = 0; members_updated = 0; }
    /members = \[/ { 
        in_members = 1; 
        if (!members_updated) {
            line = $0;
            sub(/\]$/, new_members "]", line);
            print line;
            members_updated = 1;
            next;
        }
    }
    /^]$/ {
        if (in_members) {
            in_members = 0;
            if (members_updated) next;
        }
    }
    { if (!members_updated || !in_members) print $0; }' "$WORKSPACE_TOML" > "$TMP_TOML"
    
    mv "$TMP_TOML" "$WORKSPACE_TOML"
    
    log_success "Successfully transformed project to $TARGET_TEMPLATE template"
}

# Load environment variables from .env file if present
[ -f .env ] && export $(grep -v '^#' .env | xargs) 2>/dev/null || true

# Ensure jq is available
if ! command -v jq &> /dev/null; then
    log_warning "jq not found. Using fallback mode for JSON parsing."
    log_warning "For best results, please install jq."
fi

# Create default config if it doesn't exist
if [ ! -f "$CONFIG_FILE" ]; then
    log_info "Creating default configuration..."
    cat > "$CONFIG_FILE" <<EOL
{
  "project_name": "rust_workspace",
  "template": "full-stack",
  "components": {
    "client": {
      "apps": ["app1", "app2"],
      "frameworks": ["dioxus", "dioxus"]
    },
    "server": {
      "services": ["service1", "service2"],
      "frameworks": ["poem", "actix-web"]
    },
    "database": {
      "enabled": true,
      "engines": ["postgres", "mysql"],
      "migration_tool": "sqlx"
    },
    "libs": {
      "modules": ["core", "models", "auth"]
    },
    "ai": {
      "models": ["llama", "whisper"],
      "backends": ["candle", "ort"],
      "features": ["text-generation", "speech-to-text"]
    },
    "edge": {
      "targets": ["wasm", "cloudflare-workers"],
      "features": ["edge-functions"]
    },
    "embedded": {
      "targets": ["rp2040", "esp32"],
      "features": ["blink-led"]
    }
  },
  "dependencies": {
    "dioxus": { "version": "0.4", "features": ["web"] },
    "tauri": { "version": "1.0", "features": ["all-api"] },
    "poem": { "version": "1.3", "features": [] },
    "actix-web": { "version": "4.0", "features": [] },
    "serde": { "version": "1.0", "features": ["derive"] }
  },
  "templates": {
    "full-stack": ["client", "server", "database", "libs"],
    "backend-only": ["server", "database", "libs"],
    "frontend-only": ["client", "libs"],
    "library": ["libs"],
    "minimal": ["binaries"],
    "ai": ["ai"],
    "edge": ["edge"],
    "embedded": ["embedded"]
  }
}
EOL
fi

# Process command-line arguments
parse_args "$@"

# Process configuration
process_config

# Check for Rust/Cargo installation
if ! command -v cargo &> /dev/null; then
    log_warning "Cargo not found. FerrisUp creates a Rust workspace that requires Cargo to build."
    read -p "Do you want to continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Exiting. Please install Rust and Cargo before continuing."
        exit 0
    fi
fi

# Validate project name
if [ -z "$PROJECT_NAME" ]; then
    PROJECT_NAME="rust_workspace"
    log_warning "No project name specified. Using default: $PROJECT_NAME"
fi

# Check if target directory already exists
if [ -d "$PROJECT_NAME" ]; then
    log_error "Directory '$PROJECT_NAME' already exists!"
    read -p "Do you want to overwrite it? This will delete all existing content. (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Aborted. Please choose a different project name."
        exit 0
    else
        log_warning "Removing existing directory '$PROJECT_NAME'..."
        rm -rf "$PROJECT_NAME"
    fi
fi

# Create project structure
log_info "Creating workspace structure for template: $TEMPLATE"
mkdir -p "$PROJECT_NAME"

# Create component directories based on active template
for component in "${ACTIVE_COMPONENTS[@]}"; do
    case "$component" in
        "client")
            log_info "Setting up client components..."
            mkdir -p "$PROJECT_NAME/client"
            for app in "${CLIENT_APPS[@]}"; do mkdir -p "$PROJECT_NAME/client/$app"; done
            mkdir -p "$PROJECT_NAME/client/common"
            ;;
        "server")
            log_info "Setting up server components..."
            mkdir -p "$PROJECT_NAME/server"
            for service in "${SERVER_SERVICES[@]}"; do mkdir -p "$PROJECT_NAME/server/$service"; done
            mkdir -p "$PROJECT_NAME/server/common"
            ;;
        "database")
            log_info "Setting up database components..."
            mkdir -p "$PROJECT_NAME/database"
            mkdir -p "$PROJECT_NAME/database/migrations"
            mkdir -p "$PROJECT_NAME/database/schema"
            mkdir -p "$PROJECT_NAME/database/seeds"
            mkdir -p "$PROJECT_NAME/database/utils"
            ;;
        "libs")
            log_info "Setting up library components..."
            mkdir -p "$PROJECT_NAME/libs"
            for lib in "${LIBS_MODULES[@]}"; do mkdir -p "$PROJECT_NAME/libs/$lib"; done
            ;;
        "binaries")
            log_info "Setting up binary components..."
            mkdir -p "$PROJECT_NAME/binaries"
            for app in "${BINARY_APPS[@]}"; do mkdir -p "$PROJECT_NAME/binaries/$app"; done
            ;;
        "ai")
            log_info "Setting up AI components..."
            mkdir -p "$PROJECT_NAME/ai"
            mkdir -p "$PROJECT_NAME/ai/models"
            mkdir -p "$PROJECT_NAME/ai/pipelines"
            mkdir -p "$PROJECT_NAME/ai/examples"
            ;;
        "edge")
            log_info "Setting up edge computing components..."
            mkdir -p "$PROJECT_NAME/edge"
            mkdir -p "$PROJECT_NAME/edge/functions"
            mkdir -p "$PROJECT_NAME/edge/workers"
            ;;
        "embedded")
            log_info "Setting up embedded systems components..."
            mkdir -p "$PROJECT_NAME/embedded"
            mkdir -p "$PROJECT_NAME/embedded/src"
            mkdir -p "$PROJECT_NAME/embedded/memory.x"
            ;;
    esac
done

# Create Cargo workspace configuration
log_info "Creating Cargo workspace configuration..."
cat > "$PROJECT_NAME/Cargo.toml" <<EOL
[workspace]
members = [
EOL

# Add workspace members based on active components
if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " client " ]]; then
    for app in "${CLIENT_APPS[@]}"; do echo "    \"client/$app\"," >> "$PROJECT_NAME/Cargo.toml"; done
    echo "    \"client/common\"," >> "$PROJECT_NAME/Cargo.toml"
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " server " ]]; then
    for service in "${SERVER_SERVICES[@]}"; do echo "    \"server/$service\"," >> "$PROJECT_NAME/Cargo.toml"; done
    echo "    \"server/common\"," >> "$PROJECT_NAME/Cargo.toml"
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " database " ]]; then
    echo "    \"database\"," >> "$PROJECT_NAME/Cargo.toml"
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " libs " ]]; then
    for lib in "${LIBS_MODULES[@]}"; do echo "    \"libs/$lib\"," >> "$PROJECT_NAME/Cargo.toml"; done
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " binaries " ]]; then
    for app in "${BINARY_APPS[@]}"; do echo "    \"binaries/$app\"," >> "$PROJECT_NAME/Cargo.toml"; done
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " ai " ]]; then
    echo "    \"ai\"," >> "$PROJECT_NAME/Cargo.toml"
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " edge " ]]; then
    echo "    \"edge\"," >> "$PROJECT_NAME/Cargo.toml"
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " embedded " ]]; then
    echo "    \"embedded\"," >> "$PROJECT_NAME/Cargo.toml"
fi

# Remove trailing comma and finish workspace config
sed -i '$ s/,$//' "$PROJECT_NAME/Cargo.toml"
echo "]
resolver = \"2\"" >> "$PROJECT_NAME/Cargo.toml"

# Create Cargo.toml files and sample code
create_cargo_file() {
    local path=$1
    local name=$2
    local extra_deps=$3
    
    cat > "$path/Cargo.toml" <<EOL
[package]
name = "$name"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
$extra_deps
EOL
}

# Create basic files for each component
if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " libs " ]]; then
    log_info "Generating library code..."
    for lib in "${LIBS_MODULES[@]}"; do
        create_cargo_file "$PROJECT_NAME/libs/$lib" "$lib" ""
        
        # Create library source file
        mkdir -p "$PROJECT_NAME/libs/$lib/src"
        cat > "$PROJECT_NAME/libs/$lib/src/lib.rs" <<EOL
pub fn greet() -> String {
    "Hello from $lib library!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(greet(), "Hello from $lib library!".to_string());
    }
}
EOL
    done
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " client " ]]; then
    log_info "Generating client code..."
    # Create client/common
    create_cargo_file "$PROJECT_NAME/client/common" "common" ""
    
    # Create client apps
    for i in "${!CLIENT_APPS[@]}"; do
        app=${CLIENT_APPS[$i]}
        framework=${CLIENT_FRAMEWORKS[$i]}
        
        # Build dependencies
        deps=""
        if [ "$framework" == "dioxus" ]; then
            deps="dioxus = { version = \"0.4\", features = [\"web\"] }"
        elif [ "$framework" == "tauri" ]; then
            deps="tauri = { version = \"1.0\", features = [\"all-api\"] }"
        fi
        
        # Add common dependency
        deps="${deps}
common = { path = \"../common\" }"
        
        # Add models if it exists
        if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " libs " ]] && [[ " ${LIBS_MODULES[@]} " =~ " models " ]]; then
            deps="${deps}
models = { path = \"../../libs/models\" }"
        fi
        
        create_cargo_file "$PROJECT_NAME/client/$app" "$app" "$deps"
        
        # Create app source file
        mkdir -p "$PROJECT_NAME/client/$app/src"
        cat > "$PROJECT_NAME/client/$app/src/main.rs" <<EOL
fn main() {
    println!("Hello from $app client application!");
}
EOL
    done
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " server " ]]; then
    log_info "Generating server code..."
    # Create server/common
    create_cargo_file "$PROJECT_NAME/server/common" "common" ""
    
    # Create server services
    for i in "${!SERVER_SERVICES[@]}"; do
        service=${SERVER_SERVICES[$i]}
        framework=${SERVER_FRAMEWORKS[$i]}
        
        # Build dependencies
        deps="$framework = \"1.3\"
common = { path = \"../common\" }"
        
        # Add models if it exists
        if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " libs " ]]; then
            if [[ " ${LIBS_MODULES[@]} " =~ " models " ]]; then
                deps="${deps}
models = { path = \"../../libs/models\" }"
            fi
            if [[ " ${LIBS_MODULES[@]} " =~ " auth " ]]; then
                deps="${deps}
auth = { path = \"../../libs/auth\" }"
            fi
        fi
        
        create_cargo_file "$PROJECT_NAME/server/$service" "$service" "$deps"
        
        # Create service source file
        mkdir -p "$PROJECT_NAME/server/$service/src"
        cat > "$PROJECT_NAME/server/$service/src/main.rs" <<EOL
fn main() {
    println!("Hello from $service server application using $framework!");
}
EOL
    done
fi

if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " database " ]]; then
    log_info "Generating database code..."
    
    # Build database dependencies based on configured engines
    db_deps=""
    for engine in "${DATABASE_ENGINES[@]}"; do
        case "$engine" in
            "postgres")
                db_deps="${db_deps}sqlx = { version = \"0.7\", features = [\"runtime-tokio\", \"tls-rustls\", \"postgres\", \"macros\", \"migrate\"] }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "mysql")
                db_deps="${db_deps}sqlx = { version = \"0.7\", features = [\"runtime-tokio\", \"tls-rustls\", \"mysql\", \"macros\", \"migrate\"] }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "sqlite")
                db_deps="${db_deps}sqlx = { version = \"0.7\", features = [\"runtime-tokio\", \"sqlite\", \"macros\", \"migrate\"] }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "redis")
                db_deps="${db_deps}redis = { version = \"0.24\", features = [\"tokio-comp\"] }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "sea-orm")
                db_deps="${db_deps}sea-orm = { version = \"0.12\", features = [\"runtime-tokio-rustls\", \"sqlx-postgres\", \"macros\"] }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "diesel")
                db_deps="${db_deps}diesel = { version = \"2.1\", features = [\"postgres\", \"r2d2\"] }"
                ;;
            "neo4j")
                db_deps="${db_deps}neo4rs = { version = \"0.6\" }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "milvus")
                db_deps="${db_deps}milvus-sdk = { version = \"0.1\" }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "qdrant")
                db_deps="${db_deps}qdrant-client = { version = \"1.6\" }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "typedb")
                db_deps="${db_deps}typedb-client = { version = \"0.1\" }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "dgraph")
                db_deps="${db_deps}dgraph-client = { version = \"0.3\" }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "scylladb")
                db_deps="${db_deps}scylla = { version = \"0.11\", features = [\"ssl\", \"tokio-comp\"] }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "arangodb")
                db_deps="${db_deps}arangors = { version = \"0.5\", features = [\"reqwest_async\"] }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "iroh")
                db_deps="${db_deps}iroh = { version = \"0.12\" }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            "hypercore")
                db_deps="${db_deps}hypercore = { version = \"0.1\" }
tokio = { version = \"1.0\", features = [\"full\"] }"
                ;;
            *)
                log_warning "Unknown database engine: $engine. Skipping..."
                ;;
        esac
    done
    
    # Add dotenv for environment configuration
    db_deps="${db_deps}
dotenv = \"0.15\"
serde = { version = \"1.0\", features = [\"derive\"] }
chrono = { version = \"0.4\", features = [\"serde\"] }"
    
    create_cargo_file "$PROJECT_NAME/database" "database" "$db_deps"
    
    # Create database source file based on migration tool
    mkdir -p "$PROJECT_NAME/database/src"
    
    # Create sample .env file for database connection
    cat > "$PROJECT_NAME/database/.env.example" <<EOL
# Database Configuration
# Copy this file to .env and modify as needed

# SQL Databases
DATABASE_URL="postgresql://postgres:postgres@localhost:5432/mydb"
MYSQL_URL="mysql://user:password@localhost:3306/mydb"
SQLITE_PATH="./database.sqlite"

# NoSQL Databases
REDIS_URL="redis://127.0.0.1:6379"
MONGODB_URI="mongodb://localhost:27017/mydb"

# Vector Databases
MILVUS_URL="http://localhost:19530"
QDRANT_URL="http://localhost:6333"

# Graph Databases
NEO4J_URI="bolt://localhost:7687"
NEO4J_USER="neo4j"
NEO4J_PASSWORD="password"
DGRAPH_ALPHA_URL="http://localhost:8080"
ARANGO_URL="http://localhost:8529"
ARANGO_DB="mydb"
ARANGO_USER="root"
ARANGO_PASSWORD="password"

# Time Series
SCYLLA_NODES="127.0.0.1:9042"
SCYLLA_KEYSPACE="mykeyspace"

# Knowledge/Semantic Databases
TYPEDB_URI="localhost:1729"
TYPEDB_DATABASE="mydb"

# P2P Databases
IROH_PATH="./iroh-data"
HYPERCORE_PATH="./hypercore-data"

# Connection settings
MAX_CONNECTIONS=5
CONNECTION_TIMEOUT=5
EOL
    
    # Create sample migrations directory structure based on migration tool
    case "$DATABASE_MIGRATION_TOOL" in
        "sqlx")
            # Create sample migration
            mkdir -p "$PROJECT_NAME/database/migrations"
            cat > "$PROJECT_NAME/database/migrations/20250328000001_initial_schema.sql" <<EOL
-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
EOL
            ;;
        "diesel")
            # Create diesel.toml
            cat > "$PROJECT_NAME/database/diesel.toml" <<EOL
# For documentation on how to configure this file,
# see https://diesel.rs/guides/configuring-diesel-cli

[print_schema]
file = "src/schema.rs"
custom_type_derives = ["diesel::query_builder::QueryId"]

[migrations_directory]
dir = "migrations"
EOL
            
            # Create sample migration
            mkdir -p "$PROJECT_NAME/database/migrations/20250328000001_initial_schema"
            cat > "$PROJECT_NAME/database/migrations/20250328000001_initial_schema/up.sql" <<EOL
-- Add migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
EOL
            cat > "$PROJECT_NAME/database/migrations/20250328000001_initial_schema/down.sql" <<EOL
-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS users;
EOL
            ;;
        "sea-orm")
            # Create sample migration
            mkdir -p "$PROJECT_NAME/database/migration/src/m20250328_000001_create_user_table"
            cat > "$PROJECT_NAME/database/migration/src/m20250328_000001_create_user_table/mod.rs" <<EOL
use sea_orm_migration::{prelude::*, sea_orm::Statement};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250328_000001_create_user_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(100) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX idx_users_email ON users(email);
        "#;
        
        manager.get_connection().execute(Statement::from_string(
            manager.get_database_backend(),
            sql.to_string(),
        )).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
            DROP TABLE IF EXISTS users;
        "#;
        
        manager.get_connection().execute(Statement::from_string(
            manager.get_database_backend(),
            sql.to_string(),
        )).await?;

        Ok(())
    }
}
EOL
            ;;
    esac
    
    # Create main database connection module
    cat > "$PROJECT_NAME/database/src/lib.rs" <<EOL
use dotenv::dotenv;
use std::env;

pub mod config;
pub mod error;
pub mod models;

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " postgres " || " ${DATABASE_ENGINES[@]} " =~ " mysql " || " ${DATABASE_ENGINES[@]} " =~ " sqlite " ]]; then
echo 'pub async fn connect() -> Result<sqlx::Pool<sqlx::Any>, error::DatabaseError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| error::DatabaseError::ConfigError("DATABASE_URL not set".to_string()))?;
    
    sqlx::any::AnyPoolOptions::new()
        .max_connections(
            env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        )
        .connect(&database_url)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " redis " ]]; then
echo 'pub async fn connect_redis() -> Result<redis::aio::ConnectionManager, error::DatabaseError> {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL")
        .map_err(|_| error::DatabaseError::ConfigError("REDIS_URL not set".to_string()))?;
    
    let client = redis::Client::open(redis_url)
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))?;
    
    redis::aio::ConnectionManager::new(client)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " neo4j " ]]; then
echo 'pub async fn connect_neo4j() -> Result<neo4rs::Graph, error::DatabaseError> {
    dotenv().ok();
    let uri = env::var("NEO4J_URI")
        .map_err(|_| error::DatabaseError::ConfigError("NEO4J_URI not set".to_string()))?;
    let user = env::var("NEO4J_USER")
        .map_err(|_| error::DatabaseError::ConfigError("NEO4J_USER not set".to_string()))?;
    let password = env::var("NEO4J_PASSWORD")
        .map_err(|_| error::DatabaseError::ConfigError("NEO4J_PASSWORD not set".to_string()))?;
    
    let config = neo4rs::ConfigBuilder::new()
        .uri(uri)
        .user(user)
        .password(password)
        .build()
        .map_err(|e| error::DatabaseError::ConfigError(e.to_string()))?;
    
    neo4rs::Graph::connect(config)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " milvus " ]]; then
echo 'pub async fn connect_milvus() -> Result<milvus_sdk::client::Client, error::DatabaseError> {
    dotenv().ok();
    let milvus_url = env::var("MILVUS_URL")
        .map_err(|_| error::DatabaseError::ConfigError("MILVUS_URL not set".to_string()))?;
    
    milvus_sdk::client::Client::new(&milvus_url)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " qdrant " ]]; then
echo 'pub fn connect_qdrant() -> Result<qdrant_client::client::QdrantClient, error::DatabaseError> {
    dotenv().ok();
    let qdrant_url = env::var("QDRANT_URL")
        .map_err(|_| error::DatabaseError::ConfigError("QDRANT_URL not set".to_string()))?;
    
    qdrant_client::client::QdrantClient::from_url(&qdrant_url)
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " typedb " ]]; then
echo 'pub fn connect_typedb() -> Result<typedb_client::connection::Connection, error::DatabaseError> {
    dotenv().ok();
    let typedb_uri = env::var("TYPEDB_URI")
        .map_err(|_| error::DatabaseError::ConfigError("TYPEDB_URI not set".to_string()))?;
    let typedb_database = env::var("TYPEDB_DATABASE")
        .map_err(|_| error::DatabaseError::ConfigError("TYPEDB_DATABASE not set".to_string()))?;
    
    let connection = typedb_client::connection::Connection::new(vec![typedb_uri], None)
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))?;
    
    // Ensure database exists or create it
    if !connection.databases().all().contains(&typedb_database) {
        connection.databases().create(&typedb_database)
            .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))?;
    }
    
    Ok(connection)
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " scylladb " ]]; then
echo 'pub async fn connect_scylla() -> Result<std::sync::Arc<scylla::Session>, error::DatabaseError> {
    dotenv().ok();
    let nodes: Vec<String> = env::var("SCYLLA_NODES")
        .map_err(|_| error::DatabaseError::ConfigError("SCYLLA_NODES not set".to_string()))?
        .split(",")
        .map(|s| s.to_string())
        .collect();
    
    let keyspace = env::var("SCYLLA_KEYSPACE")
        .map_err(|_| error::DatabaseError::ConfigError("SCYLLA_KEYSPACE not set".to_string()))?;
    
    let session = scylla::SessionBuilder::new()
        .known_nodes(&nodes)
        .build()
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))?;
    
    // Ensure keyspace exists
    session.query(
        format!(
            "CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{\
            'class': 'SimpleStrategy', 'replication_factor': 1 }}", 
            keyspace
        ),
        &[]
    )
    .await
    .map_err(|e| error::DatabaseError::QueryError(e.to_string()))?;
    
    session.use_keyspace(keyspace, false)
        .await
        .map_err(|e| error::DatabaseError::QueryError(e.to_string()))?;
    
    Ok(std::sync::Arc::new(session))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " dgraph " ]]; then
echo 'pub fn connect_dgraph() -> Result<dgraph_client::Client, error::DatabaseError> {
    dotenv().ok();
    let dgraph_url = env::var("DGRAPH_ALPHA_URL")
        .map_err(|_| error::DatabaseError::ConfigError("DGRAPH_ALPHA_URL not set".to_string()))?;
    
    dgraph_client::Client::new(vec![dgraph_url])
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " arangodb " ]]; then
echo 'pub async fn connect_arangodb() -> Result<arangors::client::reqwest::ReqwestClient, error::DatabaseError> {
    dotenv().ok();
    let arango_url = env::var("ARANGO_URL")
        .map_err(|_| error::DatabaseError::ConfigError("ARANGO_URL not set".to_string()))?;
    let arango_db = env::var("ARANGO_DB")
        .map_err(|_| error::DatabaseError::ConfigError("ARANGO_DB not set".to_string()))?;
    let arango_user = env::var("ARANGO_USER")
        .map_err(|_| error::DatabaseError::ConfigError("ARANGO_USER not set".to_string()))?;
    let arango_password = env::var("ARANGO_PASSWORD")
        .map_err(|_| error::DatabaseError::ConfigError("ARANGO_PASSWORD not set".to_string()))?;
    
    let conn = arangors::Connection::establish_basic_auth(&arango_url, &arango_user, &arango_password)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))?;
    
    // Ensure database exists
    if !conn.has_database(&arango_db).await.unwrap_or(false) {
        conn.create_database(&arango_db)
            .await
            .map_err(|e| error::DatabaseError::QueryError(e.to_string()))?;
    }
    
    conn.db(&arango_db)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " iroh " ]]; then
echo 'pub async fn connect_iroh() -> Result<iroh::client::Iroh, error::DatabaseError> {
    dotenv().ok();
    let iroh_path = env::var("IROH_PATH")
        .map_err(|_| error::DatabaseError::ConfigError("IROH_PATH not set".to_string()))?;
    
    iroh::client::Iroh::new(iroh_path)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " hypercore " ]]; then
echo 'pub fn connect_hypercore() -> Result<hypercore::Hypercore, error::DatabaseError> {
    dotenv().ok();
    let hypercore_path = env::var("HYPERCORE_PATH")
        .map_err(|_| error::DatabaseError::ConfigError("HYPERCORE_PATH not set".to_string()))?;
    
    hypercore::Hypercore::open(hypercore_path)
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " sea-orm " ]]; then
echo 'pub async fn connect_sea_orm() -> Result<sea_orm::DatabaseConnection, error::DatabaseError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| error::DatabaseError::ConfigError("DATABASE_URL not set".to_string()))?;
    
    sea_orm::Database::connect(&database_url)
        .await
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

$(if [[ " ${DATABASE_ENGINES[@]} " =~ " diesel " ]]; then
echo 'pub fn connect_diesel() -> Result<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, error::DatabaseError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| error::DatabaseError::ConfigError("DATABASE_URL not set".to_string()))?;
    
    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url);
    diesel::r2d2::Pool::builder()
        .max_size(
            env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        )
        .build(manager)
        .map_err(|e| error::DatabaseError::ConnectionError(e.to_string()))
}'
fi)

// Helper function to get connection based on the database type
pub fn get_connection() -> Result<(), error::DatabaseError> {
    // For demonstration purposes, this is a placeholder
    // In a real application, you would retrieve the appropriate connection
    // based on configuration or return the appropriate connection type
    Ok(())
}
EOL
{{ ... }}

    # Create database config module
    mkdir -p "$PROJECT_NAME/database/src/config"
    cat > "$PROJECT_NAME/database/src/config/mod.rs" <<EOL
use dotenv::dotenv;
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        dotenv().ok();
        
        Self {
            url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/mydb".to_string()),
            max_connections: env::var("MAX_CONNECTIONS").unwrap_or_else(|_| "5".to_string()).parse().unwrap_or(5),
            connection_timeout: env::var("CONNECTION_TIMEOUT").unwrap_or_else(|_| "5".to_string()).parse().unwrap_or(5),
        }
    }
}
EOL

    # Create database error module
    mkdir -p "$PROJECT_NAME/database/src/error"
    cat > "$PROJECT_NAME/database/src/error/mod.rs" <<EOL
use std::fmt;

#[derive(Debug)]
pub enum DatabaseError {
    ConfigError(String),
    ConnectionError(String),
    QueryError(String),
    MigrationError(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConfigError(msg) => write!(f, "Database configuration error: {}", msg),
            DatabaseError::ConnectionError(msg) => write!(f, "Database connection error: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "Database query error: {}", msg),
            DatabaseError::MigrationError(msg) => write!(f, "Database migration error: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}
EOL

    # Create models module with user example
    mkdir -p "$PROJECT_NAME/database/src/models"
    cat > "$PROJECT_NAME/database/src/models/mod.rs" <<EOL
pub mod user;
EOL

    # Create user model
    cat > "$PROJECT_NAME/database/src/models/user.rs" <<EOL
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
EOL
fi

# Create binary applications if configured
create_binary_apps() {
    if [[ " ${ACTIVE_COMPONENTS[@]} " =~ " binaries " ]]; then
        log_info "Generating binary applications..."
        
        # If no binary apps defined, use default hello world
        if [ ${#BINARY_APPS[@]} -eq 0 ]; then
            BINARY_APPS=("hello")
        fi
        
        for app in "${BINARY_APPS[@]}"; do
            log_info "Creating binary application: $app"
            mkdir -p "$PROJECT_NAME/binaries/$app/src"
            
            # Create binary Cargo.toml
            cat > "$PROJECT_NAME/binaries/$app/Cargo.toml" <<EOL
[package]
name = "$app"
version = "0.1.0"
edition = "2021"

[dependencies]
EOL
            
            # Create minimal hello world for the minimal template
            if [ "$TEMPLATE" = "minimal" ] || [ "$TEMPLATE" = "hello-world" ]; then
                cat > "$PROJECT_NAME/binaries/$app/src/main.rs" <<EOL
fn main() {
    println!("Hello, world from FerrisUp!");
}
EOL
            else
                # Create a more structured application for non-minimal templates
                cat > "$PROJECT_NAME/binaries/$app/src/main.rs" <<EOL
//! $app binary

mod cli;
mod config;

fn main() {
    println!("Starting $app application...");
    
    // Initialize configuration
    let config = config::load_config().expect("Failed to load configuration");
    
    // Run the application
    if let Err(e) = run(config) {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}

/// Run the application with the provided configuration
fn run(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running with configuration: {:?}", config);
    Ok(())
}
EOL
                
                # Create supporting modules for structured applications
                mkdir -p "$PROJECT_NAME/binaries/$app/src/cli"
                cat > "$PROJECT_NAME/binaries/$app/src/cli/mod.rs" <<EOL
//! Command-line interface module
EOL
                
                mkdir -p "$PROJECT_NAME/binaries/$app/src/config"
                cat > "$PROJECT_NAME/binaries/$app/src/config/mod.rs" <<EOL
//! Configuration module

/// Application configuration
#[derive(Debug)]
pub struct Config {
    pub app_name: String,
    pub version: String,
}

/// Load configuration from environment or file
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        app_name: "$app".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
EOL
            fi
        done
    fi
}

# Add deployment and scaling configurations
create_deployment_configs() {
    if [ "$SCALE_PROJECT" = true ]; then
        log_info "Adding scaling and deployment configurations..."
        
        # Create docker configuration if enabled
        if [ "$DOCKER_ENABLED" = true ]; then
            log_info "Adding Docker configuration..."
            
            # Create Dockerfile
            cat > "$PROJECT_NAME/Dockerfile" <<EOL
FROM rust:1.73 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/$PROJECT_NAME /app/

# Install necessary runtime dependencies
RUN apt-get update && \\
    apt-get install -y --no-install-recommends \\
    ca-certificates && \\
    rm -rf /var/lib/apt/lists/*

EXPOSE 8080

CMD ["/app/$PROJECT_NAME"]
EOL
            
            # Create .dockerignore
            cat > "$PROJECT_NAME/.dockerignore" <<EOL
target/
.git/
.github/
.gitignore
**/*.rs.bk
EOL
        fi
        
        # Create kubernetes configuration if enabled
        if [ "$K8S_ENABLED" = true ]; then
            log_info "Adding Kubernetes configuration..."
            mkdir -p "$PROJECT_NAME/k8s"
            
            # Create kubernetes deployment file
            cat > "$PROJECT_NAME/k8s/deployment.yaml" <<EOL
apiVersion: apps/v1
kind: Deployment
metadata:
  name: $PROJECT_NAME
spec:
  replicas: 1
  selector:
    matchLabels:
      app: $PROJECT_NAME
  template:
    metadata:
      labels:
        app: $PROJECT_NAME
    spec:
      containers:
      - name: $PROJECT_NAME
        image: $PROJECT_NAME:latest
        ports:
        - containerPort: 8080
        resources:
          limits:
            cpu: "0.5"
            memory: "512Mi"
          requests:
            cpu: "0.1"
            memory: "128Mi"
EOL
            
            # Create kubernetes service file
            cat > "$PROJECT_NAME/k8s/service.yaml" <<EOL
apiVersion: v1
kind: Service
metadata:
  name: $PROJECT_NAME
spec:
  selector:
    app: $PROJECT_NAME
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
EOL
        fi
        
        # Create CI/CD configuration if enabled
        if [ "$CI_CD_ENABLED" = true ]; then
            log_info "Adding CI/CD configuration..."
            mkdir -p "$PROJECT_NAME/.github/workflows"
            
            # Create GitHub Actions workflow file
            cat > "$PROJECT_NAME/.github/workflows/ci.yml" <<EOL
name: CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
        components: rustfmt, clippy
    
    - name: Check formatting
      uses: actions-rs/cargo@v1
      with:
        command: fmt
        args: --all -- --check
    
    - name: Lint with clippy
      uses: actions-rs/cargo@v1
      with:
        command: clippy
        args: --all-targets --all-features -- -D warnings
    
    - name: Build
      uses: actions-rs/cargo@v1
      with:
        command: build
        args: --verbose
    
    - name: Test
      uses: actions-rs/cargo@v1
      with:
        command: test
        args: --verbose
EOL
        fi
    fi
}

# Initialize Git repository if not skipped
if [ "$SKIP_GIT" = "false" ]; then
    log_info "Initializing Git repository..."
    if ! command -v git &> /dev/null; then
        log_warning "Git not found. Skipping Git initialization."
    else
        (cd "$PROJECT_NAME" && git init && git add . && 
         git commit -m "Initial commit: Bootstrap Rust workspace with FerrisUp")
        
        # Prompt for remote repository
        echo "Enter remote repository URL (or leave empty to skip):"
        read REMOTE_URL
        
        if [ -n "$REMOTE_URL" ]; then
            log_info "Pushing to remote repository..."
            (cd "$PROJECT_NAME" && git remote add origin "$REMOTE_URL" && 
             git push -u origin master) || log_warning "Failed to push to remote repository."
            log_success "Project pushed to remote repository."
        fi
    fi
else
    log_info "Git initialization skipped."
fi

log_success "Workspace setup complete! Project created at: $PROJECT_NAME"
log_info "Use 'cd $PROJECT_NAME' to navigate to your new Rust workspace."

# Call functions to create components
create_client_apps
create_server_services
create_database
create_libs
create_binary_apps
create_ai_components
create_edge_components
create_embedded_components
create_deployment_configs

log_success "FerrisUp has initialized your $TEMPLATE Rust workspace: $PROJECT_NAME"
log_info "To build your workspace, run: cd $PROJECT_NAME && cargo build"

if [ -z "$TRANSFORM_TARGET" ]; then
    log_success "FerrisUp has initialized your $TEMPLATE Rust workspace: $PROJECT_NAME"
    log_info "To transform this project to a different template later, use:"
    log_info "  $0 --transform=full-stack --project=$PROJECT_NAME"
fi
