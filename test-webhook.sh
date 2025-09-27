#!/bin/bash
# Test script for the webhook receiver

echo "🧪 Testing Rust Webhook Receiver"
echo "================================="

WEBHOOK_URL="http://localhost:8080"

echo "1. Health Check..."
curl -s "$WEBHOOK_URL/health" | jq '.' 2>/dev/null || echo "Health check failed or jq not installed"
echo ""

echo "2. Testing webhook with JSON payload..."
curl -X POST \
  -H "Content-Type: application/json" \
  -H "X-Test-Header: webhook-test" \
  -d '{
    "timestamp": "'$(date -Iseconds)'",
    "service": "auth-service", 
    "level": "INFO",
    "message": "User login attempt",
    "fields": {
      "user_id": "12345",
      "ip_address": "192.168.1.100"
    }
  }' \
  "$WEBHOOK_URL/webhook" | jq '.' 2>/dev/null || echo "Webhook test failed or jq not installed"
echo ""

echo "3. Testing webhook with plain text payload..."
curl -X POST \
  -H "Content-Type: text/plain" \
  -H "X-Source: app-service" \
  -d "This is a plain text log message from app-service at $(date)" \
  "$WEBHOOK_URL/webhook" | jq '.' 2>/dev/null || echo "Plain text test failed or jq not installed"
echo ""

echo "4. Getting logs info..."
curl -s "$WEBHOOK_URL/logs" | jq '.' 2>/dev/null || echo "Logs endpoint failed or jq not installed"

echo ""
echo "✅ Webhook receiver test completed!"
echo "Check the webhook receiver console output to see the logged data."