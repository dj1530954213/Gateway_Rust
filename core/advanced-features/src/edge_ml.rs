/*!
# Edge Machine Learning Engine

Lightweight ML capabilities for real-time inference at the edge.
*/

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use crate::{ComponentStatus, HealthLevel};

/// Edge ML engine for lightweight inference
pub struct EdgeMLEngine {
    models: Arc<RwLock<HashMap<String, MLModel>>>,
    config: MLConfig,
    status: Arc<RwLock<ComponentStatus>>,
    inference_cache: Arc<RwLock<HashMap<String, CachedInference>>>,
}

/// ML configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    pub enable_gpu: bool,
    pub max_models: usize,
    pub cache_size: usize,
    pub inference_timeout_ms: u64,
    pub model_directory: PathBuf,
    pub auto_quantization: bool,
    pub batch_inference: bool,
    pub max_batch_size: usize,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            enable_gpu: false,
            max_models: 10,
            cache_size: 1000,
            inference_timeout_ms: 5000,
            model_directory: PathBuf::from("models"),
            auto_quantization: true,
            batch_inference: true,
            max_batch_size: 32,
        }
    }
}

/// ML model representation
#[derive(Debug, Clone)]
pub struct MLModel {
    pub id: String,
    pub name: String,
    pub model_type: ModelType,
    pub version: String,
    pub input_schema: Vec<InputField>,
    pub output_schema: Vec<OutputField>,
    pub model_path: PathBuf,
    pub metadata: HashMap<String, String>,
    pub performance_stats: ModelPerformance,
}

/// Model type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    LinearRegression,
    LogisticRegression,
    DecisionTree,
    RandomForest,
    NeuralNetwork,
    SVM,
    KMeans,
    AnomalyDetection,
    TimeSeries,
    Custom(String),
}

/// Input field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub description: String,
}

/// Output field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputField {
    pub name: String,
    pub field_type: FieldType,
    pub description: String,
}

/// Field type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Float,
    Integer,
    Boolean,
    String,
    Array,
    Object,
}

/// Model performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub total_inferences: u64,
    pub average_inference_time_ms: f64,
    pub accuracy: Option<f64>,
    pub error_rate: f64,
    pub last_inference: Option<chrono::DateTime<chrono::Utc>>,
    pub memory_usage_mb: f64,
}

impl Default for ModelPerformance {
    fn default() -> Self {
        Self {
            total_inferences: 0,
            average_inference_time_ms: 0.0,
            accuracy: None,
            error_rate: 0.0,
            last_inference: None,
            memory_usage_mb: 0.0,
        }
    }
}

/// ML inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_id: String,
    pub input_data: HashMap<String, serde_json::Value>,
    pub request_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// ML inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: Option<String>,
    pub model_id: String,
    pub output_data: HashMap<String, serde_json::Value>,
    pub confidence: Option<f64>,
    pub inference_time_ms: f64,
    pub status: InferenceStatus,
    pub error_message: Option<String>,
}

/// Inference status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceStatus {
    Success,
    Error,
    Timeout,
    InvalidInput,
    ModelNotFound,
}

/// Cached inference result
#[derive(Debug, Clone)]
struct CachedInference {
    input_hash: String,
    response: InferenceResponse,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl EdgeMLEngine {
    /// Create new edge ML engine
    pub async fn new() -> Result<Self> {
        Self::new_with_config(MLConfig::default()).await
    }

    /// Create new edge ML engine with configuration
    pub async fn new_with_config(config: MLConfig) -> Result<Self> {
        let models = Arc::new(RwLock::new(HashMap::new()));
        let status = Arc::new(RwLock::new(ComponentStatus::default()));
        let inference_cache = Arc::new(RwLock::new(HashMap::new()));

        // Create model directory if it doesn't exist
        if !config.model_directory.exists() {
            tokio::fs::create_dir_all(&config.model_directory).await
                .context("Failed to create model directory")?;
        }

        let engine = Self {
            models,
            config,
            status,
            inference_cache,
        };

        info!("Edge ML engine created");
        Ok(engine)
    }

    /// Start ML engine
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;

        // Load models from directory
        self.load_models_from_directory().await?;

        info!("Edge ML engine started");
        Ok(())
    }

    /// Stop ML engine
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = false;

        // Unload all models
        let mut models = self.models.write().await;
        models.clear();

        info!("Edge ML engine stopped");
        Ok(())
    }

    /// Load model
    pub async fn load_model(&self, model: MLModel) -> Result<()> {
        let mut models = self.models.write().await;
        
        if models.len() >= self.config.max_models {
            return Err(anyhow::anyhow!("Maximum number of models reached"));
        }

        models.insert(model.id.clone(), model.clone());
        info!("Loaded ML model: {} ({})", model.name, model.id);
        Ok(())
    }

    /// Unload model
    pub async fn unload_model(&self, model_id: &str) -> Result<()> {
        let mut models = self.models.write().await;
        
        if models.remove(model_id).is_some() {
            info!("Unloaded ML model: {}", model_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model not found: {}", model_id))
        }
    }

    /// Perform inference
    pub async fn inference(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let start_time = std::time::Instant::now();

        // Check cache first
        if let Some(cached) = self.check_cache(&request).await {
            return Ok(cached);
        }

        // Get model
        let models = self.models.read().await;
        let model = models.get(&request.model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", request.model_id))?;

        // Validate input
        self.validate_input(&request, model)?;

        // Perform inference based on model type
        let result = self.perform_inference(&request, model).await;

        let inference_time = start_time.elapsed().as_millis() as f64;

        let response = match result {
            Ok(output_data) => InferenceResponse {
                request_id: request.request_id.clone(),
                model_id: request.model_id.clone(),
                output_data,
                confidence: Some(0.95), // Placeholder
                inference_time_ms: inference_time,
                status: InferenceStatus::Success,
                error_message: None,
            },
            Err(e) => InferenceResponse {
                request_id: request.request_id.clone(),
                model_id: request.model_id.clone(),
                output_data: HashMap::new(),
                confidence: None,
                inference_time_ms: inference_time,
                status: InferenceStatus::Error,
                error_message: Some(e.to_string()),
            },
        };

        // Cache successful results
        if matches!(response.status, InferenceStatus::Success) {
            self.cache_inference(&request, &response).await;
        }

        // Update model performance
        self.update_model_performance(&request.model_id, &response).await;

        Ok(response)
    }

    /// Get model list
    pub async fn list_models(&self) -> Vec<MLModel> {
        let models = self.models.read().await;
        models.values().cloned().collect()
    }

    /// Get model by ID
    pub async fn get_model(&self, model_id: &str) -> Option<MLModel> {
        let models = self.models.read().await;
        models.get(model_id).cloned()
    }

    /// Get component status
    pub async fn get_status(&self) -> Result<ComponentStatus> {
        let status = self.status.read().await;
        Ok(status.clone())
    }

    /// Load models from directory
    async fn load_models_from_directory(&self) -> Result<()> {
        // This is a placeholder - in a real implementation,
        // you would scan the directory and load model files
        debug!("Loading models from directory: {:?}", self.config.model_directory);
        
        // Create a sample linear regression model
        let sample_model = MLModel {
            id: "sample_linear_regression".to_string(),
            name: "Sample Linear Regression".to_string(),
            model_type: ModelType::LinearRegression,
            version: "1.0.0".to_string(),
            input_schema: vec![
                InputField {
                    name: "temperature".to_string(),
                    field_type: FieldType::Float,
                    required: true,
                    min_value: Some(-50.0),
                    max_value: Some(100.0),
                    description: "Temperature in Celsius".to_string(),
                },
                InputField {
                    name: "pressure".to_string(),
                    field_type: FieldType::Float,
                    required: true,
                    min_value: Some(0.0),
                    max_value: Some(1000.0),
                    description: "Pressure in kPa".to_string(),
                },
            ],
            output_schema: vec![
                OutputField {
                    name: "efficiency".to_string(),
                    field_type: FieldType::Float,
                    description: "Predicted efficiency percentage".to_string(),
                },
            ],
            model_path: self.config.model_directory.join("linear_regression.model"),
            metadata: HashMap::new(),
            performance_stats: ModelPerformance::default(),
        };

        self.load_model(sample_model).await?;
        Ok(())
    }

    /// Check inference cache
    async fn check_cache(&self, request: &InferenceRequest) -> Option<InferenceResponse> {
        let input_hash = self.hash_input(&request.input_data);
        let cache = self.inference_cache.read().await;
        
        if let Some(cached) = cache.get(&input_hash) {
            // Check if cache entry is still valid (5 minutes)
            let age = chrono::Utc::now().signed_duration_since(cached.timestamp);
            if age.num_minutes() < 5 {
                debug!("Cache hit for request");
                return Some(cached.response.clone());
            }
        }
        
        None
    }

    /// Cache inference result
    async fn cache_inference(&self, request: &InferenceRequest, response: &InferenceResponse) {
        let input_hash = self.hash_input(&request.input_data);
        let cached = CachedInference {
            input_hash: input_hash.clone(),
            response: response.clone(),
            timestamp: chrono::Utc::now(),
        };

        let mut cache = self.inference_cache.write().await;
        
        // Simple cache eviction - remove oldest if at capacity
        if cache.len() >= self.config.cache_size {
            let oldest_key = cache.iter()
                .min_by_key(|(_, v)| v.timestamp)
                .map(|(k, _)| k.clone());
            
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }

        cache.insert(input_hash, cached);
    }

    /// Hash input data for caching
    fn hash_input(&self, input_data: &HashMap<String, serde_json::Value>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let serialized = serde_json::to_string(input_data).unwrap_or_default();
        serialized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Validate input against model schema
    fn validate_input(&self, request: &InferenceRequest, model: &MLModel) -> Result<()> {
        for field in &model.input_schema {
            if field.required && !request.input_data.contains_key(&field.name) {
                return Err(anyhow::anyhow!("Required field missing: {}", field.name));
            }

            if let Some(value) = request.input_data.get(&field.name) {
                // Basic type validation
                match (&field.field_type, value) {
                    (FieldType::Float, serde_json::Value::Number(_)) => {},
                    (FieldType::Integer, serde_json::Value::Number(n)) if n.is_i64() => {},
                    (FieldType::Boolean, serde_json::Value::Bool(_)) => {},
                    (FieldType::String, serde_json::Value::String(_)) => {},
                    (FieldType::Array, serde_json::Value::Array(_)) => {},
                    (FieldType::Object, serde_json::Value::Object(_)) => {},
                    _ => return Err(anyhow::anyhow!("Type mismatch for field: {}", field.name)),
                }

                // Range validation for numeric fields
                if let (Some(min), Some(max), serde_json::Value::Number(n)) = 
                    (field.min_value, field.max_value, value) {
                    if let Some(val) = n.as_f64() {
                        if val < min || val > max {
                            return Err(anyhow::anyhow!(
                                "Value {} for field {} out of range [{}, {}]",
                                val, field.name, min, max
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Perform actual inference
    async fn perform_inference(
        &self,
        request: &InferenceRequest,
        model: &MLModel,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // This is a placeholder implementation
        // In a real system, you would load and execute the actual model
        
        match model.model_type {
            ModelType::LinearRegression => {
                // Simple linear regression: efficiency = 50 + 0.5 * temperature + 0.1 * pressure
                let temperature = request.input_data.get("temperature")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let pressure = request.input_data.get("pressure")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let efficiency = 50.0 + 0.5 * temperature + 0.1 * pressure;
                
                let mut result = HashMap::new();
                result.insert("efficiency".to_string(), 
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(efficiency).unwrap_or(serde_json::Number::from(0))
                    ));
                
                Ok(result)
            }
            
            ModelType::AnomalyDetection => {
                // Simple anomaly detection based on z-score
                let value = request.input_data.values()
                    .find_map(|v| v.as_f64())
                    .unwrap_or(0.0);

                let is_anomaly = value.abs() > 2.0; // Simple threshold
                let anomaly_score = value.abs() / 2.0;

                let mut result = HashMap::new();
                result.insert("is_anomaly".to_string(), 
                    serde_json::Value::Bool(is_anomaly));
                result.insert("anomaly_score".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(anomaly_score).unwrap_or(serde_json::Number::from(0))
                    ));

                Ok(result)
            }

            _ => {
                Err(anyhow::anyhow!("Model type not implemented: {:?}", model.model_type))
            }
        }
    }

    /// Update model performance statistics
    async fn update_model_performance(&self, model_id: &str, response: &InferenceResponse) {
        let mut models = self.models.write().await;
        
        if let Some(model) = models.get_mut(model_id) {
            let stats = &mut model.performance_stats;
            stats.total_inferences += 1;
            stats.average_inference_time_ms = 
                (stats.average_inference_time_ms + response.inference_time_ms) / 2.0;
            stats.last_inference = Some(chrono::Utc::now());

            if matches!(response.status, InferenceStatus::Error) {
                stats.error_rate = (stats.error_rate + 1.0) / stats.total_inferences as f64;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_edge_ml_engine_creation() {
        let engine = EdgeMLEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_model_loading() {
        let engine = EdgeMLEngine::new().await.unwrap();
        engine.start().await.unwrap();

        let models = engine.list_models().await;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "Sample Linear Regression");
    }

    #[tokio::test]
    async fn test_inference() {
        let engine = EdgeMLEngine::new().await.unwrap();
        engine.start().await.unwrap();

        let mut input_data = HashMap::new();
        input_data.insert("temperature".to_string(), serde_json::Value::Number(serde_json::Number::from(20)));
        input_data.insert("pressure".to_string(), serde_json::Value::Number(serde_json::Number::from(100)));

        let request = InferenceRequest {
            model_id: "sample_linear_regression".to_string(),
            input_data,
            request_id: Some("test_request".to_string()),
            metadata: HashMap::new(),
        };

        let response = engine.inference(request).await.unwrap();
        assert!(matches!(response.status, InferenceStatus::Success));
        assert!(response.output_data.contains_key("efficiency"));
    }

    #[tokio::test]
    async fn test_input_validation() {
        let engine = EdgeMLEngine::new().await.unwrap();
        engine.start().await.unwrap();

        // Test 1: Missing required field (pressure)
        let mut input_data = HashMap::new();
        input_data.insert("temperature".to_string(), serde_json::Value::Number(serde_json::Number::from(20)));
        // Missing pressure

        let request = InferenceRequest {
            model_id: "sample_linear_regression".to_string(),
            input_data,
            request_id: None,
            metadata: HashMap::new(),
        };

        let response = engine.inference(request).await.unwrap();
        assert!(matches!(response.status, InferenceStatus::Error));
        assert!(response.error_message.is_some());
        assert!(response.error_message.unwrap().contains("Required field missing: pressure"));

        // Test 2: Value out of range
        let mut input_data = HashMap::new();
        input_data.insert("temperature".to_string(), serde_json::Value::Number(serde_json::Number::from(150))); // Out of range (max 100)
        input_data.insert("pressure".to_string(), serde_json::Value::Number(serde_json::Number::from(50)));

        let request = InferenceRequest {
            model_id: "sample_linear_regression".to_string(),
            input_data,
            request_id: None,
            metadata: HashMap::new(),
        };

        let response = engine.inference(request).await.unwrap();
        assert!(matches!(response.status, InferenceStatus::Error));

        // Test 3: Wrong data type
        let mut input_data = HashMap::new();
        input_data.insert("temperature".to_string(), serde_json::Value::String("not a number".to_string()));
        input_data.insert("pressure".to_string(), serde_json::Value::Number(serde_json::Number::from(50)));

        let request = InferenceRequest {
            model_id: "sample_linear_regression".to_string(),
            input_data,
            request_id: None,
            metadata: HashMap::new(),
        };

        let response = engine.inference(request).await.unwrap();
        assert!(matches!(response.status, InferenceStatus::Error));
    }
}