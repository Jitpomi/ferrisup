//! Data processor module for transforming and analyzing data.

use serde::{Deserialize, Serialize};
use crate::Result;

/// A trait for processing data of any type.
pub trait DataProcessor<T, U> {
    /// Process the input data and produce output.
    fn process(&self, input: T) -> Result<U>;
}

/// A basic data record that can be processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    /// Unique identifier for the record
    pub id: String,
    /// Value of the record
    pub value: f64,
    /// Timestamp when the record was created
    pub timestamp: String,
}

/// A simple processor that filters data records based on a value threshold.
pub struct ThresholdProcessor {
    /// The threshold value used for filtering
    pub threshold: f64,
}

impl ThresholdProcessor {
    /// Create a new threshold processor with the given threshold value.
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }
}

impl DataProcessor<Vec<DataRecord>, Vec<DataRecord>> for ThresholdProcessor {
    fn process(&self, input: Vec<DataRecord>) -> Result<Vec<DataRecord>> {
        Ok(input
            .into_iter()
            .filter(|record| record.value >= self.threshold)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_processor() {
        let records = vec![
            DataRecord {
                id: "1".to_string(),
                value: 10.0,
                timestamp: "2023-01-01".to_string(),
            },
            DataRecord {
                id: "2".to_string(),
                value: 5.0,
                timestamp: "2023-01-02".to_string(),
            },
        ];

        let processor = ThresholdProcessor::new(7.0);
        let result = processor.process(records).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "1");
    }
}
