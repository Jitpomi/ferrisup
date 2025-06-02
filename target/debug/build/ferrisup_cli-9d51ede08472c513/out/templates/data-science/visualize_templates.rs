// Template Visualizer for Burn Templates
// Creates ASCII diagrams to visualize the structure of the templates

fn main() {
    println!("Burn Templates Visualization");
    println!("===========================\n");

    visualize_overview();
    visualize_image_classifier();
    visualize_text_analyzer();
    visualize_data_predictor();
}

fn visualize_overview() {
    println!("Overview of Burn Templates");
    println!("-------------------------\n");
    println!("                            ┌─────────────────┐");
    println!("                     ┌──────┤ Image Classifier │");
    println!("                     │      └─────────────────┘");
    println!("                     │");
    println!("┌──────────────┐     │      ┌─────────────────┐");
    println!("│ Burn Framework├─────┼──────┤ Text Analyzer   │");
    println!("└──────────────┘     │      └─────────────────┘");
    println!("                     │");
    println!("                     │      ┌─────────────────┐");
    println!("                     └──────┤ Data Predictor  │");
    println!("                            └─────────────────┘\n");
    println!("All templates share a common structure:");
    println!("- Central config.rs with tweakable parameters");
    println!("- Clear customization points marked with comments");
    println!("- Comprehensive documentation");
    println!("- Complete template.json configuration\n");
}

fn visualize_image_classifier() {
    println!("\nImage Classifier Template");
    println!("------------------------\n");
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│                  config.rs                          │");
    println!("│  (Centralized configuration for all parameters)     │");
    println!("└───────────────────────┬─────────────────────────────┘");
    println!("            ┌───────────┼───────────────┐");
    println!("            │           │               │");
    println!("┌───────────▼───────┐ ┌─▼─────────────┐ ┌▼─────────────────┐");
    println!("│     data.rs       │ │   model.rs    │ │    main.rs       │");
    println!("│ (Data processing) │ │ (CNN model)   │ │ (CLI interface)  │");
    println!("└───────────────────┘ └───────────────┘ └──────────────────┘");
    println!("            │               │                  │");
    println!("            └───────────────┼──────────────────┘");
    println!("                            │");
    println!("                 ┌──────────▼─────────┐");
    println!("                 │  Image Classification │");
    println!("                 └────────────────────┘\n");
    println!("Features:");
    println!("- CNN architecture for image classification");
    println!("- Image loading and preprocessing");
    println!("- Data augmentation options");
    println!("- Multi-class classification\n");
}

fn visualize_text_analyzer() {
    println!("\nText Analyzer Template");
    println!("---------------------\n");
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│                  config.rs                          │");
    println!("│  (Centralized configuration for all parameters)     │");
    println!("└───────────────────────┬─────────────────────────────┘");
    println!("            ┌───────────┼───────────────┐");
    println!("            │           │               │");
    println!("┌───────────▼───────┐ ┌─▼─────────────┐ ┌▼─────────────────┐");
    println!("│     data.rs       │ │   model.rs    │ │    main.rs       │");
    println!("│ (Text processing) │ │ (LSTM model)  │ │ (CLI interface)  │");
    println!("└───────────────────┘ └───────────────┘ └──────────────────┘");
    println!("            │               │                  │");
    println!("            └───────────────┼──────────────────┘");
    println!("                            │");
    println!("                 ┌──────────▼─────────┐");
    println!("                 │  Sentiment Analysis │");
    println!("                 └────────────────────┘\n");
    println!("Features:");
    println!("- LSTM architecture for text processing");
    println!("- Word embedding and tokenization");
    println!("- Text augmentation options");
    println!("- Sentiment classification (positive/negative/neutral)\n");
}

fn visualize_data_predictor() {
    println!("\nData Predictor Template");
    println!("----------------------\n");
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│                  config.rs                          │");
    println!("│  (Centralized configuration for all parameters)     │");
    println!("└───────────────────────┬─────────────────────────────┘");
    println!("            ┌───────────┼───────────────┐");
    println!("            │           │               │");
    println!("┌───────────▼───────┐ ┌─▼─────────────┐ ┌▼─────────────────┐");
    println!("│     data.rs       │ │   model.rs    │ │    main.rs       │");
    println!("│ (Data processing) │ │ (MLP model)   │ │ (CLI interface)  │");
    println!("└───────────────────┘ └───────────────┘ └──────────────────┘");
    println!("            │               │                  │");
    println!("            └───────────────┼──────────────────┘");
    println!("                            │");
    println!("                 ┌──────────▼─────────┐");
    println!("                 │  Value Prediction   │");
    println!("                 └────────────────────┘\n");
    println!("Features:");
    println!("- Neural network for numerical prediction");
    println!("- CSV data loading and normalization");
    println!("- Regression analysis");
    println!("- Housing price prediction example\n");
}
