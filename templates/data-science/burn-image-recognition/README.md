# Burn MNIST Image Recognition Template

> **Part of the FerrisUp Image Classification Template Family**
>
> This template is grouped under **Image Classification** alongside the General Image Classifier template (`burn-image-classifier`).
>
> - Use **this template** for a quickstart, minimal example using MNIST (grayscale digit recognition).
> - Use **burn-image-classifier** for general-purpose image classification with RGB images (e.g., CIFAR-10 or your own dataset), more advanced configuration, and custom data support.
>
> **Summary of Differences:**
> - This template is simpler, focused on grayscale digits, and best for demos or teaching.
> - The classifier template supports custom datasets, RGB images, and is more configurable.

This project provides a robust, up-to-date workflow for MNIST digit recognition using the [Burn](https://burn.dev) deep learning framework (v0.16.1). It features:

- Modern data batching and normalization
- Flexible model definition
- CLI for training, evaluation, and prediction
- Out-of-the-box usage with CPU (NDArray backend)

## Quickstart

### 1. Install Rust and Clone

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/your-org/your-repo.git
cd your-repo
```

### 2. Build

```sh
cargo build --release
```

### 3. Train a Model

```sh
cargo run --bin {{ project_name }} -- train --epochs 10 --batch-size 64 --learning-rate 0.001 --model-path ./model.json
```

### 4. Evaluate

```sh
cargo run --bin {{ project_name }} -- evaluate --model-path ./model.json --batch-size 64
```

### 5. Predict

```sh
cargo run --bin {{ project_name }} -- predict --model-path ./model.json --image-path ./some_digit.png
```

## Features

- **Data:** Uses Burn’s built-in MNIST utilities and normalization ([src/data.rs](src/data.rs))
- **Model:** Convolutional neural network ([src/model.rs](src/model.rs))
- **Training/Eval:** Progress bars and metrics ([src/training.rs](src/training.rs))
- **CLI:** Train, evaluate, or predict from the command line ([src/main.rs](src/main.rs))
- **Dependencies:** All required crates included in `Cargo.toml`

## Troubleshooting

- **MNIST Data Not Found:**
  - Run `./download_mnist.sh` before training or evaluating.
  - The CLI will print an error if the data is missing.
- **Model File Not Found:**
  - Make sure you have trained a model before evaluating or predicting, or specify the correct `--model-path`.
- **Image File Not Found:**
  - Double-check your `--image-path` for predictions.
- **Dependency Issues:**
  - Ensure your Rust toolchain is up to date (`rustup update`).
  - If you update Burn or other dependencies, check for breaking changes.
- **Platform Support:**
  - The template is set up for CPU (NDArray backend). For GPU or Torch, see customization below.

## Testing

Run smoke tests to verify the model builds and can do a forward pass:

```sh
cargo test
```

## Customization

- **Swap out the backend:** To use GPU or Torch, edit `src/data.rs` and update the backend. You may need to add additional dependencies to `Cargo.toml`.
- **Edit the model architecture:** Modify `src/model.rs` to change the model's architecture. You can add or remove layers, change activation functions, and more.
- **Add new CLI commands or options:** Extend the CLI in `src/main.rs` to add new commands or options. You can use the existing commands as a reference.

## Advanced Extensions

Although this template is designed as a minimal example for MNIST digit recognition, it provides a solid foundation that can be extended for more advanced image processing tasks:

### Object Detection
The convolutional backbone can be adapted for:
- Simple digit/object localization by adding bounding box outputs
- Single-digit detection in more complex images
- Multi-digit detection with appropriate post-processing

### Image Segmentation
The CNN architecture can be transformed for:
- Digit segmentation (pixel-level classification)
- Simple U-Net style implementations for medical image segmentation
- Handwriting extraction from documents

### Multi-label Classification
The classification head can be modified for:
- Recognizing multiple digits in a single image
- Identifying both digit value and properties (slant, thickness, style)
- Character + symbol recognition in combined datasets

### Image Similarity Search
The feature extraction layers can be repurposed for:
- Handwriting style comparison and matching
- Similar digit retrieval from databases
- Style-based clustering of handwritten characters

### Few-Shot Learning
The pretrained model serves as an excellent base for:
- Adapting to new handwritten character sets with minimal examples
- Quick adaptation to different languages or writing systems
- Personalized handwriting recognition with few samples

### Anomaly Detection
This architecture can be transformed for:
- Detecting unusual or incorrect digit formations
- Quality control in automated digit recognition systems
- Identifying outliers in handwritten data

While the MNIST template is simpler than the full classifier template, these extensions are still possible with appropriate modifications to the model architecture and training pipeline. For more complex implementations, consider starting with the more robust `burn-image-classifier` template.

---

For more details, see the code and comments in each module. If you encounter any issues or want to extend the workflow, feel free to open an issue or PR!
