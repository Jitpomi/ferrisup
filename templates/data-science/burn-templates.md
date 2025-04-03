# Burn Framework Templates for Data Science

This document contains super-simple prompts for various data science tasks using the Burn framework. Each prompt explains what the template does in plain language, perfect for beginners with no prior experience.

## Image Recognition (MNIST)

**Prompt**: "I want my computer to recognize handwritten numbers in images."

**What this does**: This template teaches your computer to look at pictures of handwritten numbers (0-9) and correctly identify which number it sees. It's like teaching a computer to read handwriting!

**Perfect for**: Complete beginners who want to see AI in action without needing to understand complex math or programming concepts.

**Command**: `cargo generate --git https://github.com/tracel-ai/burn.git --name my-digit-recognizer examples/mnist`

## Value Prediction (Regression)

**Prompt**: "I want to predict house prices based on features like size, location, and age."

**What this does**: This template helps you build a system that can predict numbers (like prices, temperatures, or ages) based on other information. For example, it could predict a house's price based on its size, number of bedrooms, and neighborhood.

**Perfect for**: Anyone who needs to make numerical predictions based on existing data.

**Command**: `cargo generate --git https://github.com/tracel-ai/burn.git --name my-value-predictor examples/simple-regression`

## Text Classification

**Prompt**: "I want to automatically sort text messages or emails into categories."

**What this does**: This template builds a system that can read text and decide what category it belongs to. For example, it could sort news articles into topics like "Sports," "Politics," or "Technology," or classify emails as "Important," "Spam," or "Promotional."

**Perfect for**: Projects where you need to automatically organize or filter text content.

**Command**: `cargo generate --git https://github.com/tracel-ai/burn.git --name my-text-classifier examples/text-classification`

## Custom Image Dataset

**Prompt**: "I have my own collection of photos that I want my computer to learn to recognize."

**What this does**: This template helps you teach your computer to recognize specific things in your own photos. For example, you could train it to tell the difference between pictures of dogs and cats, or to identify different types of plants from your garden photos.

**Perfect for**: Projects where you want to work with your own images instead of using standard datasets.

**Command**: `cargo generate --git https://github.com/tracel-ai/burn.git --name my-photo-recognizer examples/custom-image-dataset`

## Custom CSV Dataset

**Prompt**: "I have spreadsheet data (CSV files) that I want to analyze and make predictions from."

**What this does**: This template helps you work with data from spreadsheets or CSV files. It shows how to read this data, process it, and use it to make predictions. For example, you could use customer data to predict which customers might be interested in a new product.

**Perfect for**: Working with any kind of data you might have in Excel or CSV format.

**Command**: `cargo generate --git https://github.com/tracel-ai/burn.git --name my-spreadsheet-analyzer examples/custom-csv-dataset`

## Advanced: Custom Training Loop

**Prompt**: "I want more detailed control over how my AI model learns."

**What this does**: This template gives you more control over the training process. It's like having access to the detailed settings instead of just using the automatic mode.

**Perfect for**: More experienced users who want to fine-tune how their models learn.

**Command**: `cargo generate --git https://github.com/tracel-ai/burn.git --name my-custom-trainer examples/custom-training-loop`

## Web-Based Image Classification

**Prompt**: "I want to build a website where users can upload photos and my AI will identify what's in them."

**What this does**: This template creates a web application where users can upload images and get instant AI-powered analysis. It's like building your own version of Google Lens or similar image recognition services.

**Perfect for**: Creating interactive web applications that use AI to analyze images.

**Command**: `cargo generate --git https://github.com/tracel-ai/burn.git --name my-web-classifier examples/image-classification-web`
