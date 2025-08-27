# Quick Test Commands for Auth Service

After starting the auth-service (see above), you can quickly verify all features are working with these commands:

```sh
# Health check
curl -i http://localhost:3000/health

# OpenAPI JSON
curl -s http://localhost:3000/openapi.json | jq .

# Signup example
curl -i -X POST http://localhost:3000/signup \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123","requires2FA":false}'

# To test graceful shutdown, press Ctrl+C in the server terminal
```

You can run these in any terminal to confirm the API is healthy, documented, and accepting signups.
