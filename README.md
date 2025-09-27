# Live Bootcamp Project - Auth Service

A production-ready authentication service with Redis-backed 2FA, PostgreSQL user storage, modern configuration management, and comprehensive security features.

## 🚀 Features

- **Modern Configuration Management**: Hierarchical config-rs system with environment profiles
- **Redis-backed 2FA Code Store**: Persistent, scalable 2FA code management with automatic expiration
- **PostgreSQL User Storage**: Robust user data persistence with Argon2 password hashing
- **JWT Authentication**: Secure token-based authentication with banned token tracking
- **Postmark Email Integration**: Production-ready email delivery for 2FA codes and notifications
- **Runtime-Configurable Logging**: Advanced tracing with adjustable log levels, structured JSON output, and sensitive data filtering
- **External Logging Integration**: Ship logs to Grafana Loki, AWS CloudWatch, Elasticsearch, OpenTelemetry OTLP, and webhooks
- **Enhanced File Logging**: Log rotation, compression, and cleanup with configurable policies
- **Environment Profiles**: Development, test, and production configurations
- **Type-Safe Configuration**: Validated configuration with graceful error handling
- **REST & gRPC APIs**: Complete API coverage with OpenAPI documentation
- **Docker Support**: Containerized deployment with Docker Compose

## 📋 Prerequisites

- **Rust** (latest stable)
- **Docker** and **Docker Compose**
- **PostgreSQL** (15.2+ recommended)
- **Redis** (7.0+ recommended)

## 🛠️ Setup & Installation

### 1. Database Setup (Required)

Start PostgreSQL and Redis using Docker:

```bash
# PostgreSQL Database
docker run --name ps-db \
  -e POSTGRES_PASSWORD=SecurePass2024! \
  -p 5432:5432 \
  -d postgres:15.2-alpine

# Redis Database
docker run --name redis-db \
  -p 6379:6379 \
  -d redis:7.0-alpine
```

### 2. Build the Project

```bash
# Install cargo-watch for development
cargo install cargo-watch

# Build both services
cd app-service && cargo build && cd ..
cd auth-service && cargo build && cd ..
```

## 🚦 Running the Services

### Development Mode (Manual)

#### Auth Service
```bash
cd auth-service
DATABASE_URL='postgres://postgres:SecurePass2024!@localhost:5432' \
cargo watch -q -c -w src/ -w assets/ -x "run --bin auth-service"
```
Visit: http://localhost:3000

#### App Service  
```bash
cd app-service
cargo watch -q -c -w src/ -w assets/ -w templates/ -x run
```
Visit: http://localhost:8000

### Production Mode (Docker)

```bash
# Start all services with Docker Compose
./docker.sh
```

Visit: 
- App Service: http://localhost:8000
- Auth Service: http://localhost:3000
- Access login via the "Log in" button at http://localhost:8000

## 🧪 Testing

### Prerequisites for Testing
Ensure PostgreSQL and Redis containers are running before testing.

### Run All Tests
```bash
cd auth-service
DATABASE_URL='postgres://postgres:SecurePass2024!@localhost:5432' cargo test
```

### CI/CD Testing
The GitHub Actions workflow runs comprehensive tests against real database containers:
- **PostgreSQL 15.2-alpine** service container for integration tests
- **Redis 7.0-alpine** service container for 2FA functionality
- **Health checks** ensure databases are ready before tests run
- **SQLx offline mode** for faster compilation while maintaining database testing

This approach ensures what's tested in CI is exactly what runs in production.

### Run Specific Test Categories
```bash
# Integration tests (REST & gRPC)
DATABASE_URL='postgres://postgres:SecurePass2024!@localhost:5432' cargo test --test api

# Unit tests only
cargo test --lib

# Specific test
DATABASE_URL='postgres://postgres:SecurePass2024!@localhost:5432' \
cargo test should_return_206_if_valid_credentials_and_2fa_enabled
```

### Quick API Verification
```bash
# Health check
curl -i http://localhost:3000/health

# OpenAPI documentation
curl -s http://localhost:3000/openapi.json | jq .

# User signup with 2FA enabled
curl -i -X POST http://localhost:3000/signup \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123","requires_2fa":true}'

# Login (triggers 2FA flow)
curl -i -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

## 🔧 Configuration

The application uses a modern, hierarchical configuration system powered by config-rs.

### Configuration Hierarchy (Priority: High → Low)

1. **Environment Variables** (Highest priority)
   - Modern prefix: `AUTH_DATABASE__URL`, `AUTH_REDIS__HOST`, etc.
   - Legacy compatibility: `DATABASE_URL`, `REDIS_HOST_NAME`, `JWT_SECRET`
2. **Environment-specific config files** (e.g., `config/production.toml`)
3. **Base config file** (`config/default.toml`)  
4. **Built-in defaults** (Lowest priority)

### Environment Profiles

The application supports different environments via the `ENVIRONMENT` variable:

**Development** (`config/development.toml`):
```toml
environment = "development"
[server]
host = "127.0.0.1"
port = 3000

[auth]
jwt_expiration = 3600         # 1 hour
refresh_token_expiration = 86400  # 1 day
jwt_cookie_name = "jwt_dev"
two_fa_code_expiration = 300  # 5 minutes
```

**Test** (`config/test.toml`):
```toml
environment = "test"
[server]
port = 0  # Random port for tests

[redis]
database = 1  # Isolated test database

[auth]
jwt_expiration = 60           # 1 minute (fast testing)
jwt_cookie_name = "jwt_test"
```

**Production** (`config/production.toml`):
```toml
environment = "production"
[server]
host = "0.0.0.0"
port = 3000

[redis]
host = "redis"  # Docker service name
max_connections = 20

[auth]
jwt_expiration = 7200         # 2 hours
refresh_token_expiration = 2592000  # 30 days
```

### Environment Variable Configuration

**Modern Approach (Recommended):**
```bash
# Hierarchical configuration with AUTH_ prefix
export AUTH_DATABASE__URL="postgres://user:pass@localhost:5432/auth"
export AUTH_REDIS__HOST="localhost"
export AUTH_AUTH__JWT_SECRET="your-secret-key"
export ENVIRONMENT="development"
```

**Legacy Compatibility (Still Supported):**
```bash
# Existing environment variables continue to work
export DATABASE_URL="postgres://postgres:SecurePass2024!@localhost:5432/auth"
export REDIS_HOST_NAME="localhost"
export JWT_SECRET="g4iNvB23GraeR2d1SsIDL9lxqynITs/8c9JOSL0BvY5aR6a1Lv69gl1Gq0N6vJLY5ntgpRg3WOvzqXVojUGdBA=="
export POSTGRES_PASSWORD="SecurePass2024!"
```

### Configuration Structure

```rust
AppConfig {
  environment: String,           // development, test, production
  server: {
    host: String,               // Server bind address
    port: u16,                  // Server port
  },
  database: {
    url: String,                // PostgreSQL connection URL
    max_connections: u32,       // Connection pool size
    connection_timeout: u64,    // Timeout in seconds
  },
  redis: {
    host: String,               // Redis hostname
    port: u16,                  // Redis port (default: 6379)
    database: u8,               // Redis database number
    password: Option<String>,   // Optional Redis password
  },
  auth: {
    jwt_secret: String,         // JWT signing secret (validated length)
    jwt_expiration: u64,        // JWT token lifetime (seconds)
    refresh_token_expiration: u64,  // Refresh token lifetime
    jwt_cookie_name: String,    // JWT cookie name
    two_fa_code_expiration: u64,    // 2FA code lifetime
  },
  email: {
    base_url: String,           // Postmark API base URL
    sender: String,             // From email address
    timeout: u64,               // Request timeout (seconds)
    postmark_auth_token: String, // Postmark server token
  }
}
```

## 📊 Advanced Logging & Observability

The auth service features a comprehensive, production-ready logging system with runtime-configurable levels and structured output.

### Logging Features

- **Runtime Log Level Adjustment**: Change log levels without restarting the service
- **Environment-Aware Defaults**: Automatic configuration based on deployment environment
- **Structured JSON Logging**: Machine-parseable logs for production environments  
- **Sensitive Data Filtering**: Automatic redaction of passwords, tokens, and PII
- **Request Correlation IDs**: Track requests across service boundaries
- **Performance Metrics**: Built-in timing and performance instrumentation
- **Async Logging**: Non-blocking log output for optimal performance

### Environment-Based Log Configuration

| Environment | Default Level | Format | Sensitive Data | File Logging |
|-------------|---------------|---------|----------------|--------------|
| **Development** | `debug` | Pretty Console | ✅ Included | ❌ Console Only |
| **Testing** | `info` | Structured | ❌ Filtered | ❌ Console Only |
| **Production** | `warn` | JSON | ❌ Filtered | ✅ With Rotation |

### Environment Variables

```bash
# Environment: development, testing, production
ENVIRONMENT=development

# Log level: trace, debug, info, warn, error  
LOG_LEVEL=debug

# Use JSON format for structured logging
LOG_JSON_FORMAT=false

# Include sensitive data in logs (development only)
LOG_INCLUDE_SENSITIVE=true

# Enable file logging with rotation
LOG_TO_FILE=false
LOG_FILE_PATH=/var/log/auth-service/app.log
```

### Runtime Log Control API

The auth service exposes HTTP endpoints for runtime log level management:

```bash
# Get current logging configuration
curl http://localhost:3000/logging/config

# Update log level at runtime
curl -X PUT http://localhost:3000/logging/config \
  -H "Content-Type: application/json" \
  -d '{"level": "debug", "json_format": false}'

# Test logging at different levels
curl http://localhost:3000/logging/test?level=debug
```

### Structured Log Examples

**Development (Pretty Console):**
```
2025-09-25T10:30:15.123Z  INFO auth_service::routes::login: 🔐 User login successful - no 2FA required
    email: user@example.com
    requires_2fa: false
    request_id: f47ac10b-58cc-4372-a567-0e02b2c3d479
```

**Production (JSON):**
```json
{
  "timestamp": "2025-09-25T10:30:15.123456Z",
  "level": "INFO",
  "fields": {
    "message": "User login successful - no 2FA required",
    "email": "[REDACTED]",
    "requires_2fa": false,
    "request_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
  },
  "target": "auth_service::routes::login"
}
```

### External Logging Integration

Ship logs to external services for centralized logging, monitoring, and analysis:

**Supported Services:**
- **Grafana Loki**: Log aggregation with powerful querying
- **AWS CloudWatch**: Cloud-native logging with AWS integration
- **Elasticsearch**: Full-text search and analytics
- **OpenTelemetry OTLP**: Distributed tracing and observability
- **Generic Webhooks**: Custom HTTP endpoints

**Quick Setup:**
```bash
# Enable external logging to Grafana Loki
export EXTERNAL_LOGGING_ENABLED=true
export EXTERNAL_LOGGING_SERVICE_TYPE=loki
export EXTERNAL_LOGGING_ENDPOINT=http://loki:3100/loki/api/v1/push

# Performance configuration
export EXTERNAL_LOGGING_BATCH_SIZE=100
export EXTERNAL_LOGGING_FLUSH_INTERVAL_SECS=5
export EXTERNAL_LOGGING_COMPRESS_PAYLOADS=true
```

**Advanced Features:**
- ✅ **Async Batch Processing**: Configurable batch sizes with automatic flushing
- ✅ **Compression**: Gzip compression for reduced network overhead  
- ✅ **Retry Logic**: Exponential backoff with circuit breaker pattern
- ✅ **Enhanced File Logging**: Rotation, compression, and cleanup policies

📖 **Complete Configuration Guide**: [External Logging Documentation](.azure/external-logging-config.md)

## 📁 Project Structure

```
├── auth-service/          # Authentication microservice
│   ├── config/            # Configuration files
│   │   ├── default.toml   # Base configuration
│   │   ├── development.toml # Development settings
│   │   ├── test.toml      # Test environment settings
│   │   └── production.toml # Production settings
│   ├── src/
│   │   ├── config.rs      # Configuration types and loading
│   │   ├── domain/        # Business logic and data types
│   │   ├── routes/        # API endpoints
│   │   ├── services/      # Data stores and external services
│   │   └── utils/         # Utilities and auth services
│   ├── tests/             # Integration tests
│   └── Dockerfile
├── app-service/           # Frontend application service  
├── compose.yml            # Docker Compose configuration
└── README.md
```

## ⚙️ Configuration Benefits

### Modern Configuration Management
- **Type Safety**: Configuration validated at startup with descriptive error messages
- **Environment Profiles**: Seamless switching between development, test, and production
- **Hierarchical Loading**: File-based configs with environment variable overrides
- **Validation**: Automatic validation of JWT secret length, database URLs, etc.
- **Backward Compatibility**: Existing environment variables continue to work

### Configuration Examples

**Quick Development Setup:**
```bash
# Set environment and required secrets
export ENVIRONMENT=development
export DATABASE_URL="postgres://postgres:SecurePass2024!@localhost:5432/auth"
export JWT_SECRET="your-development-secret-key-here"
cargo run
```

**Testing with Isolated Environment:**
```bash
# Test environment uses Redis database 1 and shorter token expiration
export ENVIRONMENT=test
export DATABASE_URL="postgres://postgres:SecurePass2024!@localhost:5432/auth_test"
export JWT_SECRET="test-secret-key-minimum-32-characters"
cargo test
```

**Production Deployment:**
```bash
# Production uses optimized settings from config/production.toml
export ENVIRONMENT=production
export AUTH_DATABASE__URL="postgres://prod_user:secure_pass@db:5432/prod_auth"
export AUTH_REDIS__HOST="redis-cluster"
export AUTH_AUTH__JWT_SECRET="${PRODUCTION_JWT_SECRET}"
```

## 🔐 Security Features

- **Argon2id Password Hashing**: Industry-standard password security
- **JWT Token Management**: Secure authentication with token blacklisting
- **2FA Code Management**: Redis-backed with automatic expiration
- **SQL Injection Protection**: Parameterized queries with SQLx
- **CORS Configuration**: Secure cross-origin resource sharing

## 🚀 Production Deployment

### Local Docker Compose
```bash
# Start complete stack locally (PostgreSQL + Redis + Services + Caddy)
docker compose up --build -d

# Access services through Caddy reverse proxy
curl http://localhost/auth/health        # Auth service
curl http://localhost/                   # App service
```

### DigitalOcean Deployment

This project includes automated deployment to DigitalOcean via GitHub Actions.

#### Required GitHub Secrets
Configure these in **Settings → Secrets and variables → Actions**:

**Secrets:**
- `POSTGRES_PASSWORD`: Database password (e.g., `SecurePass2024!`)
- `JWT_SECRET`: JWT signing key (see `.env` file for example)
- `POSTMARK_AUTH_TOKEN`: Postmark email service server token (get from Postmark account)
- `DOCKER_USERNAME`: Your Docker Hub username
- `DOCKER_PASSWORD`: Your Docker Hub password/token
- `DROPLET_PASSWORD`: DigitalOcean droplet root password

**Variables:**
- `DROPLET_IP`: Your DigitalOcean droplet's public IP address

#### Deployment Process

1. **Push to main branch** triggers automatic deployment
2. **CI Pipeline**: 
   - Tests run against real PostgreSQL and Redis containers
   - Docker images built and pushed to Docker Hub
3. **Production Deployment**:
   - Services deployed to DigitalOcean droplet
   - PostgreSQL, Redis, auth-service, app-service, and Caddy containers
   - Automatic SSL certificates via Caddy
   - Database migrations run automatically

#### Post-Deployment

- **App Service**: `https://your-droplet-ip/`
- **Auth API**: `https://your-droplet-ip/auth/health`
- **Health Monitoring**: Use `/health` endpoints for load balancer checks
- **Graceful Shutdown**: Supports SIGTERM for zero-downtime deployments

#### DigitalOcean Droplet Requirements
- Ubuntu 20.04+ with Docker and Docker Compose installed
- Root SSH access enabled
- Ports 80 and 443 open for web traffic
- At least 2GB RAM (recommended for PostgreSQL + Redis + services)

## 📚 API Documentation

- **OpenAPI Spec**: Available at http://localhost:3000/openapi.json
- **Interactive Docs**: Use tools like Swagger UI with the OpenAPI spec
- **gRPC Services**: Full gRPC support with reflection

## 🤝 Development

### Quick Integration Test (REST & gRPC)
```sh
cd auth-service
DATABASE_URL='postgres://postgres:SecurePass2024!@localhost:5432' \
cargo test --test api -- --nocapture
```

### Development Workflow
1. Start databases: `docker run --name ps-db ...` and `docker run --name redis-db ...`
2. Run tests: `DATABASE_URL='postgres://...' cargo test`
3. Start services: Use cargo watch commands above
4. Test changes: Use curl commands or run integration tests

## 📝 Additional Documentation

- [`auth-service/QUICK_TEST.md`](auth-service/QUICK_TEST.md): Quick API verification commands
- Integration test examples in `auth-service/tests/api/`
- gRPC regression tests in `tests/api/grpc_regression.rs`

## 🔧 Troubleshooting

### Configuration Issues

**Environment Variable Not Found:**
```bash
# Check if required variables are set
echo $ENVIRONMENT
echo $DATABASE_URL
echo $JWT_SECRET

# Verify configuration loading
export ENVIRONMENT=development
cargo run 2>&1 | grep -i config
```

**Invalid Configuration Values:**
```bash
# JWT secret too short (minimum 32 characters)
export JWT_SECRET="your-secret-key-must-be-at-least-32-characters"

# Invalid database URL format  
export DATABASE_URL="postgres://user:pass@localhost:5432/dbname"

# Invalid Redis configuration
export AUTH_REDIS__HOST="localhost"
export AUTH_REDIS__PORT="6379" 
```

**Configuration File Not Found:**
```bash
# Ensure config files exist
ls -la config/
# Should show: default.toml, development.toml, test.toml, production.toml

# Verify TOML syntax
toml-lint config/development.toml  # if toml-lint is installed
```

### Common Issues

**Database Connection Errors:**
```bash
# Ensure PostgreSQL is running
docker ps | grep postgres

# Check connection
psql postgres://postgres:SecurePass2024!@localhost:5432 -c "SELECT 1;"
```

**Redis Connection Errors:**
```bash
# Ensure Redis is running  
docker ps | grep redis

# Test Redis connection
redis-cli -h localhost -p 6379 ping
```

**GitHub Actions Deployment Failures:**
- Verify all required secrets are configured in GitHub repository settings
- Check droplet has sufficient disk space and Docker is installed
- Ensure droplet ports 80/443 are open for Caddy reverse proxy

**SQLx Compilation Issues:**
- Use `SQLX_OFFLINE=true` for offline compilation
- Run `cargo sqlx prepare` to generate query metadata when database schema changes