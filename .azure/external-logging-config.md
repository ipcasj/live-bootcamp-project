# External Logging Configuration Guide

This guide explains how to configure the auth service and app service to send logs to external services like Elasticsearch, Grafana Loki, AWS CloudWatch, and OpenTelemetry-compatible systems.

## Quick Start

Enable external logging by setting these environment variables:

```bash
# Enable external logging
EXTERNAL_LOGGING_ENABLED=true

# Choose service type: otlp, loki, elasticsearch, cloudwatch, webhook
EXTERNAL_LOGGING_SERVICE_TYPE=loki

# Set service endpoint
EXTERNAL_LOGGING_ENDPOINT=http://loki:3100/loki/api/v1/push

# Optional: Configure batching and performance
EXTERNAL_LOGGING_BATCH_SIZE=100
EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=5
```

## Service Configurations

### 1. Grafana Loki

Loki is a log aggregation system designed to work with Grafana:

```bash
# Basic Loki configuration
EXTERNAL_LOGGING_ENABLED=true
EXTERNAL_LOGGING_SERVICE_TYPE=loki
EXTERNAL_LOGGING_ENDPOINT=http://loki:3100/loki/api/v1/push

# With authentication (if required)
EXTERNAL_LOGGING_USERNAME=admin
EXTERNAL_LOGGING_PASSWORD=your-password

# Performance tuning
EXTERNAL_LOGGING_BATCH_SIZE=100
EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=5
EXTERNAL_LOGGING_COMPRESS_PAYLOADS=true
EXTERNAL_LOGGING_MAX_RETRY_ATTEMPTS=3
```

**Docker Compose Example:**

```yaml
version: '3.8'
services:
  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"
    command: -config.file=/etc/loki/local-config.yaml
    
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

### 2. AWS CloudWatch

Send logs to AWS CloudWatch Logs:

```bash
# CloudWatch configuration
EXTERNAL_LOGGING_ENABLED=true
EXTERNAL_LOGGING_SERVICE_TYPE=cloudwatch
EXTERNAL_LOGGING_ENDPOINT=https://logs.us-east-1.amazonaws.com

# AWS credentials (alternatively use IAM roles)
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
AWS_REGION=us-east-1

# CloudWatch specific settings
EXTERNAL_LOGGING_USERNAME=log-group-name
EXTERNAL_LOGGING_PASSWORD=log-stream-name
```

**IAM Policy Required:**

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "*"
    }
  ]
}
```

### 3. Elasticsearch

Send structured logs to Elasticsearch:

```bash
# Elasticsearch configuration
EXTERNAL_LOGGING_ENABLED=true
EXTERNAL_LOGGING_SERVICE_TYPE=elasticsearch
EXTERNAL_LOGGING_ENDPOINT=http://elasticsearch:9200

# With authentication
EXTERNAL_LOGGING_USERNAME=elastic
EXTERNAL_LOGGING_PASSWORD=your-password

# Index configuration
EXTERNAL_LOGGING_INDEX_NAME=auth-service-logs
```

**Docker Compose Example:**

```yaml
version: '3.8'
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
    ports:
      - "9200:9200"
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
```

### 4. OpenTelemetry (OTLP)

Send traces and logs to OTLP-compatible systems like Jaeger:

```bash
# OTLP configuration
EXTERNAL_LOGGING_ENABLED=true
EXTERNAL_LOGGING_SERVICE_TYPE=otlp
EXTERNAL_LOGGING_ENDPOINT=http://jaeger:4317

# OpenTelemetry specific settings
OTEL_SERVICE_NAME=auth-service
OTEL_SERVICE_VERSION=0.1.0
OTEL_RESOURCE_ATTRIBUTES=deployment.environment=production
```

**Docker Compose Example:**

```yaml
version: '3.8'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "4317:4317"
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

### 5. Generic Webhook

Send logs to any HTTP endpoint:

```bash
# Webhook configuration
EXTERNAL_LOGGING_ENABLED=true
EXTERNAL_LOGGING_SERVICE_TYPE=webhook
EXTERNAL_LOGGING_ENDPOINT=https://your-webhook-endpoint.com/logs

# Custom headers (JSON format)
EXTERNAL_LOGGING_HEADERS='{"Authorization":"Bearer token","Content-Type":"application/json"}'

# HTTP method (default: POST)
EXTERNAL_LOGGING_HTTP_METHOD=POST
```

## Performance Tuning

### Batching Configuration

```bash
# Batch size: Number of log entries to batch before sending
EXTERNAL_LOGGING_BATCH_SIZE=100

# Flush interval: Maximum time to wait before sending partial batch
EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=5

# Max batch wait: Maximum time to wait for a full batch
EXTERNAL_LOGGING_MAX_BATCH_WAIT_MS=10000
```

### Compression

```bash
# Enable gzip compression for payloads
EXTERNAL_LOGGING_COMPRESS_PAYLOADS=true

# Compression level (1-9, default: 6)
EXTERNAL_LOGGING_COMPRESSION_LEVEL=6
```

### Retry Logic

```bash
# Maximum retry attempts for failed requests
EXTERNAL_LOGGING_MAX_RETRY_ATTEMPTS=3

# Initial retry delay in milliseconds
EXTERNAL_LOGGING_RETRY_DELAY_MS=1000

# Maximum retry delay (exponential backoff)
EXTERNAL_LOGGING_MAX_RETRY_DELAY_MS=30000
```

### Circuit Breaker

```bash
# Enable circuit breaker to prevent cascading failures
EXTERNAL_LOGGING_ENABLE_CIRCUIT_BREAKER=true

# Failure threshold before opening circuit
EXTERNAL_LOGGING_CIRCUIT_BREAKER_THRESHOLD=5

# Circuit breaker timeout in seconds
EXTERNAL_LOGGING_CIRCUIT_BREAKER_TIMEOUT_SECS=60
```

## Local Development Setup

For local development with Docker Compose:

```yaml
version: '3.8'
services:
  auth-service:
    build: ./auth-service
    environment:
      - EXTERNAL_LOGGING_ENABLED=true
      - EXTERNAL_LOGGING_SERVICE_TYPE=loki
      - EXTERNAL_LOGGING_ENDPOINT=http://loki:3100/loki/api/v1/push
      - EXTERNAL_LOGGING_BATCH_SIZE=10  # Smaller batches for development
    depends_on:
      - loki
      
  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"
    command: -config.file=/etc/loki/local-config.yaml
    
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

## Production Deployment

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: auth-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: auth-service
  template:
    metadata:
      labels:
        app: auth-service
    spec:
      containers:
      - name: auth-service
        image: auth-service:latest
        env:
        - name: EXTERNAL_LOGGING_ENABLED
          value: "true"
        - name: EXTERNAL_LOGGING_SERVICE_TYPE
          value: "cloudwatch"
        - name: EXTERNAL_LOGGING_ENDPOINT
          value: "https://logs.us-east-1.amazonaws.com"
        - name: AWS_REGION
          value: "us-east-1"
        # Use secrets for sensitive data
        - name: EXTERNAL_LOGGING_USERNAME
          valueFrom:
            secretKeyRef:
              name: logging-secrets
              key: log-group-name
```

### Monitoring and Alerting

Monitor external logging health:

```bash
# Enable health check endpoint
EXTERNAL_LOGGING_ENABLE_HEALTH_CHECK=true
EXTERNAL_LOGGING_HEALTH_CHECK_ENDPOINT=/health/external-logging

# Metrics collection
EXTERNAL_LOGGING_ENABLE_METRICS=true
```

Health check response:
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
  }
}
```

## Troubleshooting

### Common Issues

1. **Connection Timeouts**
   ```bash
   # Increase timeout values
   EXTERNAL_LOGGING_REQUEST_TIMEOUT_MS=30000
   EXTERNAL_LOGGING_CONNECTION_TIMEOUT_MS=5000
   ```

2. **Authentication Failures**
   ```bash
   # Verify credentials
   EXTERNAL_LOGGING_USERNAME=correct-username
   EXTERNAL_LOGGING_PASSWORD=correct-password
   ```

3. **High Memory Usage**
   ```bash
   # Reduce batch size and flush more frequently
   EXTERNAL_LOGGING_BATCH_SIZE=50
   EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=2
   ```

### Debugging

Enable debug logging:

```bash
RUST_LOG=auth_service::utils::external_logging=debug
```

This will show detailed information about:
- Batch creation and flushing
- HTTP request/response details
- Retry attempts and backoff
- Circuit breaker state changes

### Log Examples

**Successful log shipping:**
```
2024-01-15T10:30:15.123Z DEBUG external_logging: Shipping batch of 100 logs to loki
2024-01-15T10:30:15.234Z DEBUG external_logging: Successfully sent batch, response: 204 No Content
```

**Retry attempts:**
```
2024-01-15T10:30:15.123Z WARN external_logging: Failed to send logs, retrying (attempt 1/3): Connection timeout
2024-01-15T10:30:16.234Z INFO external_logging: Retry successful after 1 attempts
```

**Circuit breaker activation:**
```
2024-01-15T10:30:15.123Z ERROR external_logging: Circuit breaker opened due to consecutive failures: 5
2024-01-15T10:30:75.123Z INFO external_logging: Circuit breaker half-open, testing connection
```