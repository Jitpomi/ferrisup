# Text Categorization with Rust

This template helps you build a text classification system using the Burn framework in Rust. You can train it to categorize emails, reviews, social media posts, or any other text into different categories.

## 🔍 What This Template Does

- Loads and processes text data from CSV files
- Converts text into numerical features that a neural network can understand
- Trains a neural network to classify text into categories
- Evaluates the model's accuracy
- Makes predictions on new text

## 🚀 Getting Started

### Preparing Your Data

1. Organize your text data in a CSV file with at least two columns:
   - A column containing the text to classify
   - A column containing the category label

   Example:
   ```
   text,label
   "This movie was amazing!",positive
   "Terrible service, would not recommend.",negative
   "The product works as expected.",neutral
   ```

2. Make sure your CSV file has a header row

### Running the Example

1. Train the model:
   ```bash
   cargo run -- train --data data.csv --text-col text --label-col label --epochs 10
   ```

2. Evaluate the model:
   ```bash
   cargo run -- evaluate --model model.json --data test.csv --text-col text --label-col label
   ```

3. Classify new text:
   ```bash
   cargo run -- predict --model model.json --text "This is amazing!"
   ```

## 📊 Understanding the Code

### Model Architecture

This template uses a Recurrent Neural Network (RNN) with:

- **Embedding layer**: Converts words to numerical vectors
- **LSTM layers**: Learn patterns in text sequences
- **Dropout**: Prevent overfitting
- **Fully connected layers**: Make the final classification

### Key Components

- **data.rs**: Handles loading and processing text data
- **model.rs**: Defines the neural network architecture
- **tokenizer.rs**: Converts text to numerical tokens
- **main.rs**: Contains the training and evaluation logic

## 🔧 Customizing for Your Needs

### Adjusting for Your Text

- Modify the vocabulary size in `tokenizer.rs` for your specific text domain
- Adjust the maximum sequence length based on your typical text length
- Change the number of output classes to match your categories

### Improving Performance

- Increase training time with more epochs
- Use a larger model by adding more LSTM units
- Try different learning rates

## 🛠️ Troubleshooting

### Common Issues

If you encounter issues with the Burn framework, try:

1. **Use an older version**: Try specifying `burn = "0.8.0"` in Cargo.toml
2. **Reduce batch size**: If you run out of memory, use a smaller batch size

### Text Processing Issues

- Make sure your CSV file is properly formatted
- Check that your text doesn't contain unusual characters
- Try preprocessing your text (removing special characters, lowercasing)

## 📚 Learning Resources

- [Burn Documentation](https://burn.dev/)
- [Recurrent Neural Networks Explained](https://colah.github.io/posts/2015-08-Understanding-LSTMs/)
- [Text Classification Tutorial](https://www.tensorflow.org/tutorials/keras/text_classification)

## 📝 License

This template is based on examples from the Burn framework and is available under the same license as Burn.
