// Text Tokenizer for Natural Language Processing
// This file handles converting text into numerical tokens that our model can process

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Tokenizer - converts text to sequences of token IDs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tokenizer {
    // Maximum sequence length (number of tokens per text)
    max_length: usize,
    // Special tokens
    pad_token: usize,  // Padding token (0)
    unk_token: usize,  // Unknown token (1)
    // Vocabulary size (number of unique tokens)
    vocab_size: usize,
}

impl Tokenizer {
    // Create a new tokenizer with the specified maximum sequence length
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            pad_token: 0,
            unk_token: 1,
            // Start with 1000 tokens (this is a simple tokenizer)
            // In a real application, you would build a vocabulary from your dataset
            vocab_size: 1000,
        }
    }
    
    // Get the vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocab_size
    }
    
    // Tokenize a text into a sequence of token IDs
    pub fn tokenize(&self, text: &str) -> Vec<usize> {
        // This is a very simple tokenizer that just uses character hashing
        // In a real application, you would use a more sophisticated tokenizer
        // like WordPiece, BPE, or SentencePiece
        
        // Split the text into words
        let words: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();
        
        // Convert words to token IDs using a simple hash function
        let mut tokens = Vec::with_capacity(words.len());
        
        for word in words {
            // Simple hash function to convert words to token IDs
            let token_id = self.hash_word(word);
            tokens.push(token_id);
            
            // Stop if we reach the maximum sequence length
            if tokens.len() >= self.max_length {
                break;
            }
        }
        
        // Pad or truncate to the maximum sequence length
        self.pad_or_truncate(tokens)
    }
    
    // Hash a word to a token ID
    fn hash_word(&self, word: &str) -> usize {
        // Simple hash function
        let word = word.to_lowercase();
        let mut hash = 0;
        
        for c in word.chars() {
            hash = ((hash << 5) + hash) + c as u32;
        }
        
        // Map to the range [2, vocab_size-1]
        // (0 and 1 are reserved for padding and unknown tokens)
        2 + (hash as usize % (self.vocab_size - 2))
    }
    
    // Pad or truncate a sequence to the maximum length
    fn pad_or_truncate(&self, mut tokens: Vec<usize>) -> Vec<usize> {
        // Truncate if too long
        if tokens.len() > self.max_length {
            tokens.truncate(self.max_length);
        }
        
        // Pad if too short
        while tokens.len() < self.max_length {
            tokens.push(self.pad_token);
        }
        
        tokens
    }
}

// A more realistic tokenizer would look something like this:
/*
pub struct BetterTokenizer {
    vocab: HashMap<String, usize>,
    reverse_vocab: HashMap<usize, String>,
    max_length: usize,
    pad_token: usize,
    unk_token: usize,
}

impl BetterTokenizer {
    // Build a vocabulary from a dataset
    pub fn build_vocab(texts: &[String], max_vocab_size: usize) -> Self {
        let mut word_counts = HashMap::new();
        
        // Count word frequencies
        for text in texts {
            for word in text.split_whitespace() {
                *word_counts.entry(word.to_lowercase()).or_insert(0) += 1;
            }
        }
        
        // Sort by frequency
        let mut word_counts: Vec<_> = word_counts.into_iter().collect();
        word_counts.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Create vocabulary
        let mut vocab = HashMap::new();
        let mut reverse_vocab = HashMap::new();
        
        // Add special tokens
        vocab.insert("<PAD>".to_string(), 0);
        vocab.insert("<UNK>".to_string(), 1);
        reverse_vocab.insert(0, "<PAD>".to_string());
        reverse_vocab.insert(1, "<UNK>".to_string());
        
        // Add most frequent words
        let mut idx = 2;
        for (word, _) in word_counts.iter().take(max_vocab_size - 2) {
            vocab.insert(word.clone(), idx);
            reverse_vocab.insert(idx, word.clone());
            idx += 1;
        }
        
        Self {
            vocab,
            reverse_vocab,
            max_length: 100,
            pad_token: 0,
            unk_token: 1,
        }
    }
    
    // Tokenize a text
    pub fn tokenize(&self, text: &str) -> Vec<usize> {
        let words: Vec<&str> = text
            .split_whitespace()
            .collect();
        
        let mut tokens = Vec::with_capacity(words.len());
        
        for word in words {
            let token_id = self.vocab
                .get(&word.to_lowercase())
                .copied()
                .unwrap_or(self.unk_token);
            
            tokens.push(token_id);
            
            if tokens.len() >= self.max_length {
                break;
            }
        }
        
        // Pad or truncate
        if tokens.len() > self.max_length {
            tokens.truncate(self.max_length);
        } else {
            while tokens.len() < self.max_length {
                tokens.push(self.pad_token);
            }
        }
        
        tokens
    }
}
*/
