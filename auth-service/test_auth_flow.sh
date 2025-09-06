#!/bin/bash
# Test the full authentication flow for the live-bootcamp-project
# 1. Signup (2FA off)
# 2. Login (save cookie)
# 3. Verify token
# 4. Access app service with cookie

set -e

EMAIL="testuser@example.com"
PASSWORD="password123"

# 1. Signup
curl -X POST http://localhost:3000/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "'$EMAIL'",
    "password": "'$PASSWORD'",
    "requires2FA": false
  }'
echo -e "\n[Signup complete]"

# 2. Login (save cookie)
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "email": "'$EMAIL'",
    "password": "'$PASSWORD'"
  }'
echo -e "\n[Login complete]"

# 3. Extract JWT from cookie and verify token
JWT=$(grep 'jwt' cookies.txt | awk '{print $7}')
echo "Extracted JWT: $JWT"
curl -X POST http://localhost:3000/verify-token \
  -H "Content-Type: application/json" \
  -d '{"token": "'$JWT'"}'
echo -e "\n[Token verification complete]"

# 4. Access app service with cookie
curl -b cookies.txt http://localhost:8000/
echo -e "\n[App service access complete]"
