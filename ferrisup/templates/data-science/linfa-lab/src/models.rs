use anyhow::{anyhow, Result};
use linfa::prelude::*;
use linfa_clustering::{Dbscan, DbscanFitter, DbscanParams, CommonNearestNeighbour, L2Dist};
use linfa_linear::{LinearRegression, FittedLinearRegression};
use linfa_logistic::{LogisticRegression, FittedLogisticRegression};
use linfa_svm::{Svm, SvmParams, Kernel};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2};
use std::collections::HashMap;

/// A trait for model wrappers
pub trait ModelWrapper {
    /// Predict using the model
    fn predict(&self, features: &Array2<f64>) -> Array1<f64>;
    
    /// Predict classes for classification models
    fn predict_classes(&self, features: &Array2<f64>) -> Array1<usize> {
        // Default implementation for regression models
        // Just round the predictions to the nearest integer and convert to usize
        self.predict(features)
            .mapv(|x| x.round() as usize)
    }
}

/// Create a new model of the specified type
pub fn create_model(
    model_type: &str,
    features: &Array2<f64>,
    targets: &Array1<usize>,
    params: Option<HashMap<String, String>>,
) -> Result<Box<dyn ModelWrapper>> {
    match model_type.to_lowercase().as_str() {
        "svm" => {
            use linfa_svm::{Kernel, SvmParams};
            
            let dataset = Dataset::from(features.to_owned())
                .with_targets(targets.to_owned());
            
            let kernel = if let Some(params) = &params {
                if let Some(kernel_type) = params.get("kernel") {
                    match kernel_type.to_lowercase().as_str() {
                        "linear" => Kernel::linear(),
                        "rbf" => {
                            let gamma = params.get("gamma")
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or(1.0);
                            Kernel::rbf(gamma)
                        },
                        "polynomial" => {
                            let degree = params.get("degree")
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(3);
                            Kernel::polynomial(degree as u32)
                        },
                        _ => Kernel::linear(),
                    }
                } else {
                    Kernel::linear()
                }
            } else {
                Kernel::linear()
            };
            
            let c = if let Some(params) = &params {
                params.get("c")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.0)
            } else {
                1.0
            };
            
            let model = SvmWrapper::new()
                .c(c)
                .kernel(kernel)
                .fit(&dataset)?;
            
            Ok(Box::new(model))
        },
        "logistic_regression" => {
            let dataset = Dataset::from(features.to_owned())
                .with_targets(targets.to_owned());
            
            let model = LogisticRegressionWrapper::new()
                .max_iterations(100)
                .fit(&dataset)?;
            
            Ok(Box::new(model))
        },
        "linear_regression" => {
            // Convert targets to f64 for regression
            let targets_f64 = targets.mapv(|x| x as f64);
            
            let dataset = Dataset::from(features.to_owned())
                .with_targets(targets_f64);
            
            let model = LinearRegressionWrapper::new()
                .fit(&dataset)?;
            
            Ok(Box::new(model))
        },
        "dbscan" => {
            // For clustering, we don't use targets
            let dataset = Dataset::from(features.to_owned());
            
            let min_points = if let Some(params) = &params {
                params.get("min_points")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(5)
            } else {
                5
            };
            
            let tolerance = if let Some(params) = &params {
                params.get("tolerance")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.5)
            } else {
                0.5
            };
            
            let model = DbscanWrapper::new(min_points)
                .tolerance(tolerance)
                .fit(&dataset)?;
            
            Ok(Box::new(model))
        },
        "decision_tree" => {
            use linfa_trees::DecisionTree;
            
            let dataset = Dataset::from(features.to_owned())
                .with_targets(targets.to_owned());
            
            let max_depth = if let Some(params) = &params {
                params.get("max_depth")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(5)
            } else {
                5
            };
            
            let model = DecisionTree::params()
                .max_depth(max_depth)
                .fit(&dataset)?;
            
            // Use a simple wrapper for the decision tree
            Ok(Box::new(SimpleModelWrapper::new(model)))
        },
        _ => Err(anyhow!("Unknown model type: {}", model_type)),
    }
}

/// A simple wrapper for models that implement the Predict trait
pub struct SimpleModelWrapper<M, T> {
    model: M,
    _phantom: std::marker::PhantomData<T>,
}

impl<M, T> SimpleModelWrapper<M, T> {
    pub fn new(model: M) -> Self {
        Self {
            model,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M> ModelWrapper for SimpleModelWrapper<M, f64>
where
    M: Predict<Array2<f64>, Array1<f64>>,
{
    fn predict(&self, features: &Array2<f64>) -> Array1<f64> {
        self.model.predict(features)
    }
}

impl<M> ModelWrapper for SimpleModelWrapper<M, usize>
where
    M: Predict<Array2<f64>, Array1<usize>>,
{
    fn predict(&self, features: &Array2<f64>) -> Array1<f64> {
        // Convert usize predictions to f64
        self.model.predict(features).mapv(|x| x as f64)
    }
    
    fn predict_classes(&self, features: &Array2<f64>) -> Array1<usize> {
        self.model.predict(features)
    }
}

/// A wrapper for LogisticRegression models
pub struct LogisticRegressionWrapper {
    params: LogisticRegression<f64>,
    model: Option<FittedLogisticRegression<f64, usize>>,
}

impl LogisticRegressionWrapper {
    /// Create a new LogisticRegression model with default parameters
    pub fn new() -> Self {
        Self {
            params: LogisticRegression::default(),
            model: None,
        }
    }

    /// Set the maximum number of iterations
    pub fn max_iterations(mut self, max_iterations: usize) -> Self {
        self.params = self.params.max_iterations(max_iterations);
        self
    }

    /// Set the learning rate
    pub fn learning_rate(mut self, learning_rate: f64) -> Self {
        self.params = self.params.learning_rate(learning_rate);
        self
    }

    /// Set the L2 regularization parameter
    pub fn l2_regularization(mut self, lambda: f64) -> Self {
        self.params = self.params.l2_regularization(lambda);
        self
    }
}

impl Fit<Array2<f64>, Array1<usize>> for LogisticRegressionWrapper {
    type Object = Self;
    type Err = anyhow::Error;

    fn fit(&self, dataset: &Dataset<f64, usize>) -> Result<Self::Object, Self::Err> {
        let fitted_model = self.params.fit(dataset)?;
        
        Ok(Self {
            params: self.params.clone(),
            model: Some(fitted_model),
        })
    }
}

impl Predict<Array2<f64>, Array1<usize>> for LogisticRegressionWrapper {
    fn predict(&self, x: &Array2<f64>) -> Array1<usize> {
        match &self.model {
            Some(model) => model.predict(x),
            None => panic!("Model has not been trained yet"),
        }
    }
}

impl ModelWrapper for LogisticRegressionWrapper {
    fn predict(&self, features: &Array2<f64>) -> Array1<f64> {
        // For logistic regression, we return the probabilities as f64
        match &self.model {
            Some(model) => {
                // Get the raw predictions (classes)
                let predictions = model.predict(features);
                
                // Convert to f64
                predictions.mapv(|x| x as f64)
            },
            None => panic!("Model has not been trained yet"),
        }
    }
    
    fn predict_classes(&self, features: &Array2<f64>) -> Array1<usize> {
        match &self.model {
            Some(model) => model.predict(features),
            None => panic!("Model has not been trained yet"),
        }
    }
}

/// A wrapper for LinearRegression models
pub struct LinearRegressionWrapper {
    params: LinearRegression<f64>,
    model: Option<FittedLinearRegression<f64>>,
}

impl LinearRegressionWrapper {
    /// Create a new LinearRegression model with default parameters
    pub fn new() -> Self {
        Self {
            params: LinearRegression::default(),
            model: None,
        }
    }
}

impl Fit<Array2<f64>, Array1<f64>> for LinearRegressionWrapper {
    type Object = Self;
    type Err = anyhow::Error;

    fn fit(&self, dataset: &Dataset<f64, f64>) -> Result<Self::Object, Self::Err> {
        let fitted_model = self.params.fit(dataset)?;
        
        Ok(Self {
            params: self.params.clone(),
            model: Some(fitted_model),
        })
    }
}

impl Predict<Array2<f64>, Array1<f64>> for LinearRegressionWrapper {
    fn predict(&self, x: &Array2<f64>) -> Array1<f64> {
        match &self.model {
            Some(model) => model.predict(x),
            None => panic!("Model has not been trained yet"),
        }
    }
}

impl ModelWrapper for LinearRegressionWrapper {
    fn predict(&self, features: &Array2<f64>) -> Array1<f64> {
        match &self.model {
            Some(model) => model.predict(features),
            None => panic!("Model has not been trained yet"),
        }
    }
}

/// A wrapper for SVM models
pub struct SvmWrapper {
    params: SvmParams<f64>,
    model: Option<Svm<f64, usize>>,
}

impl SvmWrapper {
    /// Create a new SVM model with default parameters
    pub fn new() -> Self {
        Self {
            params: SvmParams::default(),
            model: None,
        }
    }

    /// Set the C parameter (regularization)
    pub fn c(mut self, c: f64) -> Self {
        self.params = self.params.c(c);
        self
    }

    /// Set the kernel
    pub fn kernel(mut self, kernel: Kernel<f64>) -> Self {
        self.params = self.params.kernel(kernel);
        self
    }
}

impl Fit<Array2<f64>, Array1<usize>> for SvmWrapper {
    type Object = Self;
    type Err = anyhow::Error;

    fn fit(&self, dataset: &Dataset<f64, usize>) -> Result<Self::Object, Self::Err> {
        let fitted_model = linfa_svm::fit(dataset, &self.params)?;
        
        Ok(Self {
            params: self.params.clone(),
            model: Some(fitted_model),
        })
    }
}

impl Predict<Array2<f64>, Array1<usize>> for SvmWrapper {
    fn predict(&self, x: &Array2<f64>) -> Array1<usize> {
        match &self.model {
            Some(model) => model.predict(x),
            None => panic!("Model has not been trained yet"),
        }
    }
}

impl ModelWrapper for SvmWrapper {
    fn predict(&self, features: &Array2<f64>) -> Array1<f64> {
        // Convert usize predictions to f64
        self.predict_classes(features).mapv(|x| x as f64)
    }
    
    fn predict_classes(&self, features: &Array2<f64>) -> Array1<usize> {
        match &self.model {
            Some(model) => model.predict(features),
            None => panic!("Model has not been trained yet"),
        }
    }
}

/// A wrapper for DBSCAN clustering models
pub struct DbscanWrapper {
    params: DbscanParams<f64, L2Dist, CommonNearestNeighbour>,
    model: Option<DbscanFitter<f64, L2Dist, CommonNearestNeighbour>>,
}

impl DbscanWrapper {
    /// Create a new DBSCAN model with default parameters
    pub fn new(min_points: usize) -> Self {
        Self {
            params: Dbscan::params(min_points),
            model: None,
        }
    }

    /// Set the tolerance parameter
    pub fn tolerance(mut self, tolerance: f64) -> Self {
        self.params = self.params.tolerance(tolerance);
        self
    }
}

impl Fit<Array2<f64>, ()> for DbscanWrapper {
    type Object = Self;
    type Err = anyhow::Error;

    fn fit(&self, dataset: &Dataset<f64, ()>) -> Result<Self::Object, Self::Err> {
        let fitted_model = linfa_clustering::fit(dataset, &self.params)?;
        
        Ok(Self {
            params: self.params.clone(),
            model: Some(fitted_model),
        })
    }
}

impl Predict<Array2<f64>, Array1<usize>> for DbscanWrapper {
    fn predict(&self, x: &Array2<f64>) -> Array1<usize> {
        match &self.model {
            Some(model) => {
                // Create a dataset from the input features
                let dataset = Dataset::from(x.clone());
                
                // Predict cluster assignments
                let clusters = model.predict(&dataset);
                
                // Convert cluster assignments to usize Array1
                clusters.iter().map(|&c| c as usize).collect()
            },
            None => panic!("Model has not been trained yet"),
        }
    }
}

impl ModelWrapper for DbscanWrapper {
    fn predict(&self, features: &Array2<f64>) -> Array1<f64> {
        // Convert usize predictions to f64
        self.predict_classes(features).mapv(|x| x as f64)
    }
    
    fn predict_classes(&self, features: &Array2<f64>) -> Array1<usize> {
        match &self.model {
            Some(model) => {
                // Create a dataset from the input features
                let dataset = Dataset::from(features.clone());
                
                // Predict cluster assignments
                let clusters = model.predict(&dataset);
                
                // Convert cluster assignments to usize Array1
                clusters.iter().map(|&c| c as usize).collect()
            },
            None => panic!("Model has not been trained yet"),
        }
    }
}
