#!/bin/bash

# Auth Service Integration Verification Script
# Tests if the UI expectations match the backend implementation

echo "🔍 Auth Service UI-Backend Integration Verification"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BASE_URL="http://localhost:3000"
ERRORS=0

# Function to test endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local description=$3
    
    echo -n "Testing $method $endpoint - $description... "
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "%{http_code}" -o /dev/null "$BASE_URL$endpoint" 2>/dev/null)
    else
        response=$(curl -s -w "%{http_code}" -o /dev/null -X "$method" "$BASE_URL$endpoint" -H "Content-Type: application/json" -d '{}' 2>/dev/null)
    fi
    
    if [ $? -eq 0 ] && [ "$response" != "000" ]; then
        echo -e "${GREEN}✅ Available${NC} (HTTP $response)"
    else
        echo -e "${RED}❌ Not reachable${NC}"
        ((ERRORS++))
    fi
}

# Check if auth service is running
echo "Checking if auth service is running on $BASE_URL..."
response=$(curl -s -w "%{http_code}" -o /dev/null "$BASE_URL/health" 2>/dev/null)

if [ $? -eq 0 ] && [ "$response" = "200" ]; then
    echo -e "${GREEN}✅ Auth service is running!${NC}"
    echo
    
    # Test all endpoints that the UI expects
    echo "Testing API endpoints that UI expects:"
    test_endpoint "POST" "/login" "User login"
    test_endpoint "POST" "/signup" "User registration"
    test_endpoint "POST" "/logout" "User logout"
    test_endpoint "POST" "/verify-2fa" "2FA verification"
    test_endpoint "POST" "/forgot-password" "Password reset request"
    test_endpoint "POST" "/reset-password" "Password reset"
    test_endpoint "PATCH" "/account/settings" "Update account settings"
    test_endpoint "DELETE" "/delete-account" "Delete user account"
    
    echo
    echo "Testing static file serving:"
    test_endpoint "GET" "/" "Main HTML page"
    test_endpoint "GET" "/app.js" "JavaScript application"
    test_endpoint "GET" "/styles.css" "CSS styles"
    
else
    echo -e "${RED}❌ Auth service is not running on $BASE_URL${NC}"
    echo
    echo -e "${YELLOW}To start the auth service:${NC}"
    echo "cd auth-service"
    echo "cargo run --bin auth-service"
    echo
    echo -e "${YELLOW}Or with Docker:${NC}"
    echo "JWT_SECRET=your_secret docker compose up auth-service"
    ERRORS=1
fi

echo
echo "=================================================="
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}🎉 All checks passed! UI and backend are compatible.${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Found $ERRORS issue(s). Please check the auth service.${NC}"
    exit 1
fi