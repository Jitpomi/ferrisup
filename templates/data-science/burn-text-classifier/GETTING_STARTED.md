# Getting Started with Text Categorization in Rust

This guide will walk you through using this template to build your own text classifier in Rust.

## Prerequisites

- Rust installed on your system
- Basic understanding of programming concepts
- A set of text data with category labels (e.g., positive/negative reviews, spam/not spam)
- No prior machine learning knowledge required!

## Step 1: Understanding the Project Structure

This template includes:

- **src/main.rs**: The main application with training and evaluation commands
- **src/model.rs**: The neural network architecture
- **src/data.rs**: Code for loading and processing text data
- **src/tokenizer.rs**: Code for converting text to numerical tokens
- **Cargo.toml**: Project dependencies

## Step 2: Preparing Your Data

1. Create a CSV file with your text data:
   ```
   text,label
   "This movie was amazing!",positive
   "Terrible service, would not recommend.",negative
   "The product works as expected.",neutral
   ```

2. Make sure your CSV file has:
   - A header row with column names
   - A column for the text content
   - A column for the category label

3. For testing, you can create a small sample dataset:
   ```rust
   // Add this code to generate a sample CSV
   fn create_sample_csv() -> Result<()> {
       let file = File::create("data.csv")?;
       let mut wtr = csv::Writer::from_writer(file);
       
       wtr.write_record(&["text", "label"])?;
       wtr.write_record(&["This movie was amazing!", "positive"])?;
       wtr.write_record(&["Terrible service, would not recommend.", "negative"])?;
       wtr.write_record(&["The product works as expected.", "neutral"])?;
       // Add more examples...
       
       wtr.flush()?;
       Ok(())
   }
   ```

## Step 3: Running Your First Training

1. Build and run the project:
   ```bash
   cargo run -- train --data data.csv --text-col text --label-col label --epochs 10
   ```

2. Watch the training progress:
   ```
   Epoch 1/10:
   [00:00:05] ████████████████████████████████████ 3/3 Train Loss: 1.0986, Train Accuracy: 0.3333
   [00:00:01] ████████████████████████████████████ 1/1 Valid Loss: 1.0986, Valid Accuracy: 0.0000
   ...
   ```

3. After training completes, three files will be saved:
   - `model.json`: The trained model
   - `model.classes.json`: The category names
   - `model.tokenizer.json`: The text tokenizer

## Step 4: Evaluating Your Model

Test how well your model performs on new data:

```bash
cargo run -- evaluate --model model.json --data test.csv --text-col text --label-col label
```

You'll see results like:
```
Test Loss: 0.6931, Test Accuracy: 0.6667
```

## Step 5: Making Predictions

Classify new text:

```bash
cargo run -- predict --model model.json --text "This product is fantastic!"
```

You'll see results like:
```
Prediction: positive (0)
Confidence: 87.65%
Top predictions:
  1. positive - 87.65%
  2. neutral - 10.21%
  3. negative - 2.14%
```

## Step 6: Improving Your Model

If you want better accuracy:

1. Add more training examples
   - More diverse examples help the model generalize better
   - Try to have a balanced number of examples for each category

2. Train for longer
   ```bash
   cargo run -- train --data data.csv --epochs 50
   ```

3. Adjust the model architecture in `model.rs`
   - Increase the embedding dimension
   - Add more LSTM layers
   - Change the hidden size

4. Improve the tokenizer in `tokenizer.rs`
   - Implement a more sophisticated tokenization method
   - Build a vocabulary from your specific dataset

## Troubleshooting

### Common Issues

1. **CSV parsing errors**:
   - Check your CSV file format
   - Make sure column names match what you specify in the command
   - Handle any special characters or quotes properly

2. **Low accuracy**:
   - Add more training examples
   - Make sure your categories are distinct
   - Try training for more epochs

3. **Out of memory errors**:
   - Reduce the maximum sequence length: `--max-length 50`
   - Reduce batch size: `--batch-size 16`

## Next Steps

Once you're comfortable with this example:

1. Try more advanced text preprocessing techniques
2. Experiment with different model architectures like Transformers
3. Add data augmentation for text (synonym replacement, random insertion)
4. Apply your classifier to a real-world problem like sentiment analysis

## Need Help?

- Check the [Burn documentation](https://burn.dev/)
- Visit the [Rust NLP community](https://github.com/rust-lang/rust-analyzer)
- Explore text classification tutorials online
