# Live Bootcamp Project - Auth Service

A production-ready authentication service with Redis-backed 2FA, PostgreSQL user storage, and comprehensive security features.

## 🚀 Features

- **Redis-backed 2FA Code Store**: Persistent, scalable 2FA code management with automatic expiration
- **PostgreSQL User Storage**: Robust user data persistence with Argon2 password hashing
- **JWT Authentication**: Secure token-based authentication with banned token tracking
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

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (required)
- `REDIS_HOST_NAME`: Redis host (default: localhost)

Example:
```bash
export DATABASE_URL='postgres://postgres:SecurePass2024!@localhost:5432'
export REDIS_HOST_NAME='localhost'
```

### Redis Configuration

The application uses:
- **Database 0**: Production data (banned tokens, 2FA codes)
- **Database 1**: Test data (isolated during testing)
- **TTL Settings**: 2FA codes expire after 10 minutes

## 📁 Project Structure

```
├── auth-service/          # Authentication microservice
│   ├── src/
│   │   ├── domain/        # Business logic and data types
│   │   ├── routes/        # API endpoints
│   │   ├── services/      # Data stores and external services
│   │   └── utils/         # Utilities and constants
│   ├── tests/             # Integration tests
│   └── Dockerfile
├── app-service/           # Frontend application service  
├── compose.yml            # Docker Compose configuration
└── README.md
```

## 🔐 Security Features

- **Argon2id Password Hashing**: Industry-standard password security
- **JWT Token Management**: Secure authentication with token blacklisting
- **2FA Code Management**: Redis-backed with automatic expiration
- **SQL Injection Protection**: Parameterized queries with SQLx
- **CORS Configuration**: Secure cross-origin resource sharing

## 🚀 Production Deployment

1. **Environment Setup**: Configure production DATABASE_URL and REDIS_HOST_NAME
2. **Database Migration**: Automatic SQLx migrations on startup
3. **Health Monitoring**: Use `/health` endpoint for load balancer checks
4. **Graceful Shutdown**: Supports SIGTERM for zero-downtime deployments

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