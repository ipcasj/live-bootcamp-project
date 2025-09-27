#!/bin/bash
# Simple webhook receiver using netcat
# This demonstrates how minimal a webhook can be

PORT=${1:-8080}
echo "🚀 Starting simple webhook receiver on port $PORT"
echo "📍 Webhook endpoint: http://localhost:$PORT/webhook"

while true; do
    echo "⏳ Waiting for webhook request..."
    
    # Use netcat to listen for HTTP requests
    response=$(echo -e "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{\"status\": \"received\", \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" | nc -l -p $PORT)
    
    echo "📨 Received webhook at $(date):"
    echo "$response" | head -20
    echo "---"
done