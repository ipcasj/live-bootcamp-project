# External Logging Integration

This project now supports comprehensive external logging capabilities, allowing logs to be shipped to external services like **Grafana Loki**, **AWS CloudWatch**, **Elasticsearch**, **OpenTelemetry OTLP**, and generic webhooks.

## Quick Start

Enable external logging by setting these environment variables:

```bash
# Enable external logging
export EXTERNAL_LOGGING_ENABLED=true

# Configure service type (otlp, loki, elasticsearch, cloudwatch, webhook)  
export EXTERNAL_LOGGING_SERVICE_TYPE=loki

# Set endpoint
export EXTERNAL_LOGGING_ENDPOINT=http://localhost:3100/loki/api/v1/push

# Start the services
docker-compose up
```

## Features

✅ **Multiple Service Support**: OTLP, Grafana Loki, Elasticsearch, AWS CloudWatch, Generic Webhooks  
✅ **Async Batch Processing**: Configurable batch sizes with automatic flushing  
✅ **Compression**: Gzip compression for reduced network overhead  
✅ **Retry Logic**: Exponential backoff with circuit breaker pattern  
✅ **Runtime Configuration**: Environment-variable driven configuration  
✅ **Enhanced File Logging**: Rotation, compression, and cleanup policies  
✅ **Development & Production Ready**: Comprehensive error handling and monitoring  

## Service-Specific Configuration

### Grafana Loki

Perfect for log aggregation with Grafana visualization:

```bash
export EXTERNAL_LOGGING_ENABLED=true
export EXTERNAL_LOGGING_SERVICE_TYPE=loki
export EXTERNAL_LOGGING_ENDPOINT=http://loki:3100/loki/api/v1/push
```

**Docker Compose Example**:
```yaml
services:
  loki:
    image: grafana/loki:2.9.0
    ports: ["3100:3100"]
    command: -config.file=/etc/loki/local-config.yaml
    
  grafana:
    image: grafana/grafana:latest
    ports: ["3000:3000"]
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

### AWS CloudWatch

Send logs to AWS CloudWatch Logs:

```bash
export EXTERNAL_LOGGING_ENABLED=true
export EXTERNAL_LOGGING_SERVICE_TYPE=cloudwatch
export EXTERNAL_LOGGING_ENDPOINT=https://logs.us-east-1.amazonaws.com
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
export AWS_REGION=us-east-1
```

### Elasticsearch

Ship structured logs to Elasticsearch:

```bash
export EXTERNAL_LOGGING_ENABLED=true
export EXTERNAL_LOGGING_SERVICE_TYPE=elasticsearch
export EXTERNAL_LOGGING_ENDPOINT=http://elasticsearch:9200
```

**Docker Compose Example**:
```yaml
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
    ports: ["9200:9200"]
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
```

### OpenTelemetry (OTLP)

Send traces and logs to OTLP-compatible systems:

```bash
export EXTERNAL_LOGGING_ENABLED=true
export EXTERNAL_LOGGING_SERVICE_TYPE=otlp
export EXTERNAL_LOGGING_ENDPOINT=http://jaeger:4317
export OTEL_SERVICE_NAME=auth-service
```

## Performance Configuration

### Batch Settings
```bash
export EXTERNAL_LOGGING_BATCH_SIZE=100              # Logs per batch
export EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=5       # Max wait time
export EXTERNAL_LOGGING_MAX_BATCH_WAIT_MS=10000     # Batch timeout
```

### Compression
```bash
export EXTERNAL_LOGGING_COMPRESS_PAYLOADS=true      # Enable gzip
export EXTERNAL_LOGGING_COMPRESSION_LEVEL=6         # Compression level (1-9)
```

### Retry Configuration
```bash
export EXTERNAL_LOGGING_MAX_RETRY_ATTEMPTS=3        # Retry limit
export EXTERNAL_LOGGING_RETRY_DELAY_MS=1000         # Initial delay
export EXTERNAL_LOGGING_MAX_RETRY_DELAY_MS=30000    # Max backoff delay
```

## Enhanced File Logging

In addition to external services, the project includes enhanced local file logging:

```bash
export LOG_TO_FILE=true                              # Enable file logging
export LOG_DIR=./logs                                # Log directory
export LOG_FILE_PREFIX=auth-service                  # File prefix
export LOG_ROTATION_POLICY=daily                     # daily/hourly/size_based
export LOG_MAX_FILES=30                              # Keep 30 files
export LOG_COMPRESS_ROTATED=true                     # Compress old files
export LOG_MAX_FILE_SIZE_MB=100                      # Size limit (size_based)
```

## Production Deployment

### Docker Environment Variables

Add to your `docker-compose.yml` or Kubernetes deployments:

```yaml
environment:
  # External logging
  - EXTERNAL_LOGGING_ENABLED=true
  - EXTERNAL_LOGGING_SERVICE_TYPE=loki
  - EXTERNAL_LOGGING_ENDPOINT=http://loki:3100/loki/api/v1/push
  - EXTERNAL_LOGGING_BATCH_SIZE=50
  - EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=3
  
  # Enhanced file logging
  - LOG_TO_FILE=true
  - LOG_DIR=/app/logs
  - LOG_ROTATION_POLICY=daily
  - LOG_COMPRESS_ROTATED=true
  
  # JSON formatting for production
  - LOG_JSON_FORMAT=true
  - RUST_LOG=info
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: logging-config
data:
  EXTERNAL_LOGGING_ENABLED: "true"
  EXTERNAL_LOGGING_SERVICE_TYPE: "cloudwatch"
  EXTERNAL_LOGGING_ENDPOINT: "https://logs.us-east-1.amazonaws.com"
  EXTERNAL_LOGGING_BATCH_SIZE: "100"
  LOG_JSON_FORMAT: "true"
```

## Monitoring & Health Checks

### Log Metrics

The services generate metrics for monitoring:

- **Total logs sent**: Count of successfully shipped logs
- **Failed sends**: Count of failed shipping attempts  
- **Batch processing time**: Time taken to process and ship batches
- **Circuit breaker status**: Health of external service connections

### Health Check Endpoint

```bash
curl http://localhost:8000/health
```

Sample response:
```json
{
  "status": "healthy",
  "external_logging": {
    "enabled": true,
    "service_type": "loki", 
    "endpoint": "http://loki:3100/loki/api/v1/push",
    "last_successful_send": "2024-01-15T10:30:00Z",
    "total_logs_sent": 15847,
    "failed_sends": 2,
    "circuit_breaker_status": "closed"
  },
  "file_logging": {
    "enabled": true,
    "log_dir": "./logs",
    "rotation_policy": "daily",
    "current_file_size_mb": 45.7
  }
}
```

## Development & Testing

### Local Development with Loki

```bash
# Start Loki and Grafana
docker run -d --name loki -p 3100:3100 grafana/loki:2.9.0
docker run -d --name grafana -p 3000:3000 -e GF_SECURITY_ADMIN_PASSWORD=admin grafana/grafana:latest

# Configure services 
export EXTERNAL_LOGGING_ENABLED=true
export EXTERNAL_LOGGING_SERVICE_TYPE=loki
export EXTERNAL_LOGGING_ENDPOINT=http://localhost:3100/loki/api/v1/push
export EXTERNAL_LOGGING_BATCH_SIZE=10  # Smaller batches for dev

# Start services
cargo run --bin auth-service &
cargo run --bin app-service &

# View logs in Grafana at http://localhost:3000
```

### Testing with Mock Services

Use httpbin.org for webhook testing:

```bash
export EXTERNAL_LOGGING_ENABLED=true
export EXTERNAL_LOGGING_SERVICE_TYPE=webhook
export EXTERNAL_LOGGING_ENDPOINT=http://httpbin.org/post

cargo run --bin auth-service
```

## Troubleshooting

### Common Issues

**Connection timeouts:**
```bash
export EXTERNAL_LOGGING_REQUEST_TIMEOUT_MS=30000
```

**High memory usage:**
```bash
export EXTERNAL_LOGGING_BATCH_SIZE=50
export EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=2
```

**Authentication errors:**
```bash
export EXTERNAL_LOGGING_AUTH_TOKEN=your-token
# or for CloudWatch:
export AWS_ACCESS_KEY_ID=your-key
export AWS_SECRET_ACCESS_KEY=your-secret
```

### Debug Logging

Enable debug logging to troubleshoot:

```bash
export RUST_LOG=auth_service::utils::external_logging=debug,info
```

Debug output shows:
- Batch creation and flushing
- HTTP request/response details  
- Retry attempts and backoff timing
- Circuit breaker state changes

### Log Samples

**Successful batch shipping:**
```
DEBUG external_logging: Shipping batch of 100 logs to loki
DEBUG external_logging: Successfully sent batch, response: 204 No Content
```

**Retry attempts:**
```
WARN external_logging: Failed to send logs, retrying (attempt 1/3): Connection timeout
INFO external_logging: Retry successful after 1 attempts
```

**Circuit breaker:**
```
ERROR external_logging: Circuit breaker opened due to consecutive failures: 5
INFO external_logging: Circuit breaker half-open, testing connection
```

## Architecture

The external logging system uses:

- **ExternalLoggingLayer**: Tracing subscriber layer for capturing logs
- **ExternalLoggingClient**: Async HTTP client for shipping logs  
- **LogEntry**: Structured log format compatible with all services
- **Batching System**: Efficient collection and shipping of log entries
- **Circuit Breaker**: Fault tolerance for external service failures
- **Compression**: Gzip compression for network efficiency

This architecture ensures **high performance**, **reliability**, and **observability** for production deployments.