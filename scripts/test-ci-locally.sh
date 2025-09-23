#!/bin/bash
# Local CI simulation script
# Run this to test exactly like GitHub Actions

set -e

echo "🚀 Starting Local CI Simulation..."

# Start services (like GitHub Actions does)
echo "📦 Starting PostgreSQL and Redis..."
docker run --name ci-postgres -e POSTGRES_PASSWORD=password -e POSTGRES_DB=postgres -p 5432:5432 -d postgres:15
docker run --name ci-redis -p 6379:6379 -d redis:7

# Wait for services to be ready
echo "⏳ Waiting for services to start..."
sleep 10

# Set exact CI environment variables
export ENVIRONMENT=test
export DATABASE_URL="postgresql://postgres:password@localhost:5432/postgres"
export JWT_SECRET="my-super-secret-jwt-key-that-is-definitely-long-enough-for-validation"

echo "🧪 Running all tests..."

cd auth-service

# Run exactly what CI runs
cargo test --lib
cargo test --test api

echo "✅ All tests completed!"

# Cleanup
echo "🧹 Cleaning up..."
docker stop ci-postgres ci-redis
docker rm ci-postgres ci-redis

echo "🎉 Local CI simulation complete!"