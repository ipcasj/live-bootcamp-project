//! External logging service integrations for production-ready log shipping
//!
//! This module provides comprehensive external logging service integrations including:
//! - OpenTelemetry OTLP export for observability platforms
//! - AWS CloudWatch integration for cloud-native applications
//! - Grafana Loki integration for modern log aggregation
//! - Elasticsearch integration for enterprise search and analytics
//! - Custom webhook integration for flexible log shipping
//! - Batched, async log shipping for high performance

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use tracing::{Event, Metadata, Subscriber};
use tracing_subscriber::Layer;
use color_eyre::eyre::{eyre, Result};
use tokio::sync::mpsc;
use reqwest::Client;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use chrono::Timelike;
use tracing::field::Visit;

/// Configuration for external logging services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalLoggingConfig {
    /// Enable external logging
    pub enabled: bool,
    /// Service type: otlp, cloudwatch, loki, elasticsearch, webhook
    pub service_type: String,
    /// Service endpoint URL
    pub endpoint: String,
    /// Authentication token/key
    pub auth_token: Option<String>,
    /// AWS region (for CloudWatch)
    pub aws_region: Option<String>,
    /// Batch size for log shipping
    pub batch_size: usize,
    /// Flush interval in seconds
    pub flush_interval_secs: u64,
    /// Enable compression
    pub enable_compression: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for ExternalLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("EXTERNAL_LOGGING_ENABLED")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            service_type: std::env::var("EXTERNAL_LOGGING_SERVICE")
                .unwrap_or_else(|_| "otlp".to_string()),
            endpoint: std::env::var("EXTERNAL_LOGGING_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),
            auth_token: std::env::var("EXTERNAL_LOGGING_AUTH_TOKEN").ok(),
            aws_region: std::env::var("AWS_REGION").ok(),
            batch_size: std::env::var("EXTERNAL_LOGGING_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            flush_interval_secs: std::env::var("EXTERNAL_LOGGING_FLUSH_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            enable_compression: std::env::var("EXTERNAL_LOGGING_COMPRESSION")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            max_retries: 3,
        }
    }
}

impl ExternalLoggingConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }
}

/// Structured log entry for external services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp in RFC3339 format
    pub timestamp: String,
    /// Log level
    pub level: String,
    /// Log message
    pub message: String,
    /// Service name
    pub service: String,
    /// Environment (dev, test, prod)
    pub environment: String,
    /// Version
    pub version: String,
    /// Structured fields
    pub fields: serde_json::Value,
    /// Trace ID for correlation
    pub trace_id: Option<String>,
    /// Span ID
    pub span_id: Option<String>,
}

impl LogEntry {
    /// Create a new log entry from tracing event
    pub fn from_tracing_event(event: &Event<'_>, metadata: &Metadata<'_>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Extract the actual log message from the event
        let message = {
            let mut visitor = MessageVisitor::new();
            event.record(&mut visitor);
            if visitor.message.is_empty() {
                format!("Event from {}", metadata.target())
            } else {
                visitor.message
            }
        };
        
        Self {
            timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0)
                .unwrap_or_default()
                .to_rfc3339(),
            level: metadata.level().to_string(),
            message,
            service: env!("CARGO_PKG_NAME").to_string(),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            fields: serde_json::json!({}), // TODO: Extract fields from event
            trace_id: None, // TODO: Extract from tracing context
            span_id: None,  // TODO: Extract from tracing context
        }
    }
}

/// Visitor to extract the message from tracing events
struct MessageVisitor {
    message: String,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Remove quotes from the debug output
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len()-1].to_string();
            }
        }
    }
    
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

/// External logging service client
pub struct ExternalLoggingClient {
    config: ExternalLoggingConfig,
    http_client: Client,
    log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    sender: Option<mpsc::UnboundedSender<LogEntry>>,
}

impl ExternalLoggingClient {
    /// Create a new external logging client
    pub fn new(config: ExternalLoggingConfig) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| eyre!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            http_client,
            log_buffer: Arc::new(Mutex::new(VecDeque::new())),
            sender: None,
        })
    }

    /// Start the async log shipping background task
    pub fn start_shipping(&mut self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("External logging disabled");
            return Ok(());
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<LogEntry>();
        self.sender = Some(tx);

        let client = self.http_client.clone();
        let config = self.config.clone();
        let _buffer = self.log_buffer.clone();

        tokio::spawn(async move {
            let mut flush_interval = tokio::time::interval(Duration::from_secs(config.flush_interval_secs));
            let mut local_buffer = Vec::with_capacity(config.batch_size);

            loop {
                tokio::select! {
                    // Receive new log entries
                    log_entry = rx.recv() => {
                        if let Some(entry) = log_entry {
                            local_buffer.push(entry);
                            
                            // Flush if buffer is full
                            if local_buffer.len() >= config.batch_size {
                                if let Err(e) = Self::flush_logs(&client, &config, &mut local_buffer).await {
                                    tracing::error!("Failed to flush logs: {}", e);
                                }
                            }
                        } else {
                            break; // Channel closed
                        }
                    }
                    
                    // Periodic flush
                    _ = flush_interval.tick() => {
                        if !local_buffer.is_empty() {
                            if let Err(e) = Self::flush_logs(&client, &config, &mut local_buffer).await {
                                tracing::error!("Failed to flush logs on timer: {}", e);
                            }
                        }
                    }
                }
            }
        });

        tracing::info!(
            service_type = %self.config.service_type,
            endpoint = %self.config.endpoint,
            batch_size = self.config.batch_size,
            "External logging client started"
        );

        Ok(())
    }

    /// Send a log entry to external service
    pub fn send_log(&self, entry: LogEntry) {
        if let Some(sender) = &self.sender {
            if let Err(e) = sender.send(entry) {
                tracing::error!("Failed to send log to external service: {}", e);
            }
        }
    }

    /// Flush logs to external service
    async fn flush_logs(
        client: &Client,
        config: &ExternalLoggingConfig,
        logs: &mut Vec<LogEntry>,
    ) -> Result<()> {
        if logs.is_empty() {
            return Ok(());
        }

        let payload = match config.service_type.as_str() {
            "otlp" => Self::prepare_otlp_payload(logs)?,
            "loki" => {
                let loki_payload = Self::prepare_loki_payload(logs)?;
                tracing::error!("DEBUG: Sending Loki payload: {}", loki_payload);
                loki_payload
            },
            "elasticsearch" => Self::prepare_elasticsearch_payload(logs)?,
            "cloudwatch" => Self::prepare_cloudwatch_payload(logs)?,
            "webhook" => Self::prepare_webhook_payload(logs)?,
            _ => return Err(eyre!("Unsupported service type: {}", config.service_type)),
        };

        // Compress payload if enabled
        let body = if config.enable_compression {
            Self::compress_payload(&payload)?
        } else {
            payload.into_bytes()
        };

        // Build request
        let mut request = client.post(&config.endpoint);
        
        if let Some(token) = &config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        if config.enable_compression {
            request = request.header("Content-Encoding", "gzip");
        }

        request = request.header("Content-Type", "application/json");

        // Send with retries
        let mut retries = 0;
        while retries < config.max_retries {
            match request.try_clone().unwrap().body(body.clone()).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        tracing::debug!("Successfully shipped {} logs", logs.len());
                        logs.clear();
                        return Ok(());
                    } else {
                        let status = response.status();
                        let error_body = response.text().await.unwrap_or_else(|_| "No response body".to_string());
                        tracing::warn!(
                            status = %status,
                            error_body = %error_body,
                            "Failed to ship logs, retrying..."
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("HTTP error shipping logs: {}", e);
                }
            }
            
            retries += 1;
            if retries < config.max_retries {
                tokio::time::sleep(Duration::from_secs(2_u64.pow(retries - 1))).await; // Exponential backoff
            }
        }

        Err(eyre!("Failed to ship logs after {} retries", config.max_retries))
    }

    /// Prepare payload for OpenTelemetry OTLP
    fn prepare_otlp_payload(logs: &[LogEntry]) -> Result<String> {
        let otlp_logs = serde_json::json!({
            "resourceLogs": [{
                "resource": {
                    "attributes": [{
                        "key": "service.name",
                        "value": {"stringValue": env!("CARGO_PKG_NAME")}
                    }]
                },
                "scopeLogs": [{
                    "logRecords": logs.iter().map(|log| {
                        serde_json::json!({
                            "timeUnixNano": log.timestamp,
                            "severityText": log.level,
                            "body": {"stringValue": log.message},
                            "attributes": log.fields
                        })
                    }).collect::<Vec<_>>()
                }]
            }]
        });

        serde_json::to_string(&otlp_logs)
            .map_err(|e| eyre!("Failed to serialize OTLP payload: {}", e))
    }

    /// Prepare payload for Grafana Loki
    fn prepare_loki_payload(logs: &[LogEntry]) -> Result<String> {
        if logs.is_empty() {
            return Err(eyre!("Cannot prepare Loki payload: no logs provided"));
        }

        // Group logs by their labels to create proper streams
        let mut streams_map: std::collections::HashMap<String, Vec<(String, String)>> = std::collections::HashMap::new();
        
        for log in logs {
            // Create proper Prometheus-compatible labels (only alphanumeric and underscores)
            let labels_key = format!("service_{}_level_{}", 
                log.service.replace("-", "_"), 
                log.level.to_lowercase());
            
            // Use the log's original timestamp, converted to nanoseconds
            let timestamp_nanos = match chrono::DateTime::parse_from_rfc3339(&log.timestamp) {
                Ok(dt) => dt.timestamp_nanos_opt().unwrap_or_else(|| chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()).to_string(),
                Err(_) => chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default().to_string()
            };
            
            streams_map.entry(labels_key)
                .or_insert_with(Vec::new)
                .push((timestamp_nanos, log.message.clone()));
        }
        
        let streams: Vec<serde_json::Value> = streams_map.into_iter().map(|(labels_key, values)| {
            // Parse the labels key back to create proper labels
            let parts: Vec<&str> = labels_key.split('_').collect();
            let service = if parts.len() >= 2 { parts[1] } else { "unknown" };
            let level = if parts.len() >= 4 { parts[3] } else { "info" };
            
            serde_json::json!({
                "stream": {
                    "job": service,
                    "level": level,
                    "instance": "docker"
                },
                "values": values
            })
        }).collect();

        let loki_payload = serde_json::json!({
            "streams": streams
        });

        let payload_str = serde_json::to_string(&loki_payload)
            .map_err(|e| eyre!("Failed to serialize Loki payload: {}", e))?;
        
        tracing::warn!("DEBUG: Loki payload with {} logs: {}", logs.len(), payload_str);
        
        Ok(payload_str)
    }

    /// Prepare payload for Elasticsearch
    fn prepare_elasticsearch_payload(logs: &[LogEntry]) -> Result<String> {
        let mut ndjson = String::new();
        for log in logs {
            // Index directive
            ndjson.push_str(&serde_json::to_string(&serde_json::json!({
                "index": {
                    "_index": format!("auth-service-{}", chrono::Utc::now().format("%Y-%m")),
                    "_type": "_doc"
                }
            }))?);
            ndjson.push('\n');
            
            // Document
            ndjson.push_str(&serde_json::to_string(log)?);
            ndjson.push('\n');
        }
        
        Ok(ndjson)
    }

    /// Prepare payload for AWS CloudWatch
    fn prepare_cloudwatch_payload(logs: &[LogEntry]) -> Result<String> {
        let log_events: Vec<_> = logs.iter().map(|log| {
            serde_json::json!({
                "timestamp": chrono::DateTime::parse_from_rfc3339(&log.timestamp)
                    .unwrap()
                    .timestamp_millis(),
                "message": serde_json::to_string(log).unwrap()
            })
        }).collect();

        let payload = serde_json::json!({
            "logGroupName": format!("/aws/lambda/{}", env!("CARGO_PKG_NAME")),
            "logStreamName": format!("{}-{}", 
                std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
                chrono::Utc::now().format("%Y-%m-%d")
            ),
            "logEvents": log_events
        });

        serde_json::to_string(&payload)
            .map_err(|e| eyre!("Failed to serialize CloudWatch payload: {}", e))
    }

    /// Prepare payload for generic webhook
    fn prepare_webhook_payload(logs: &[LogEntry]) -> Result<String> {
        let payload = serde_json::json!({
            "logs": logs,
            "metadata": {
                "service": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "count": logs.len()
            }
        });

        serde_json::to_string(&payload)
            .map_err(|e| eyre!("Failed to serialize webhook payload: {}", e))
    }

    /// Compress payload using gzip
    fn compress_payload(payload: &str) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload.as_bytes())
            .map_err(|e| eyre!("Failed to compress payload: {}", e))?;
        encoder.finish()
            .map_err(|e| eyre!("Failed to finish compression: {}", e))
    }
}

/// Tracing layer for external logging services
pub struct ExternalLoggingLayer {
    client: Arc<Mutex<ExternalLoggingClient>>,
}

impl ExternalLoggingLayer {
    /// Create a new external logging layer
    pub fn new(config: ExternalLoggingConfig) -> Result<Self> {
        let mut client = ExternalLoggingClient::new(config)?;
        client.start_shipping()?;
        
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }
}

impl<S> Layer<S> for ExternalLoggingLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Prevent infinite recursion - don't externally log external logging events
        if event.metadata().target().contains("external_logging") {
            return;
        }
        
        if let Ok(client) = self.client.lock() {
            let log_entry = LogEntry::from_tracing_event(event, event.metadata());
            client.send_log(log_entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_logging_config_default() {
        let config = ExternalLoggingConfig::default();
        assert_eq!(config.service_type, "otlp");
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.flush_interval_secs, 5);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            level: "INFO".to_string(),
            message: "test message".to_string(),
            service: "test-service".to_string(),
            environment: "test".to_string(),
            version: "1.0.0".to_string(),
            fields: serde_json::json!({}),
            trace_id: None,
            span_id: None,
        };

        assert_eq!(entry.service, "test-service");
        assert_eq!(entry.level, "INFO");
    }

    #[tokio::test]
    async fn test_otlp_payload_preparation() {
        let logs = vec![LogEntry {
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            level: "INFO".to_string(),
            message: "test".to_string(),
            service: "test".to_string(),
            environment: "test".to_string(),
            version: "1.0.0".to_string(),
            fields: serde_json::json!({}),
            trace_id: None,
            span_id: None,
        }];

        let payload = ExternalLoggingClient::prepare_otlp_payload(&logs).unwrap();
        assert!(payload.contains("resourceLogs"));
        assert!(payload.contains("logRecords"));
    }
}