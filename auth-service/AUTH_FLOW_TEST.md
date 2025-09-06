# Authentication Flow Test Guide

This document describes how to test the full authentication flow for the live-bootcamp-project, including signup, login, token verification, and accessing the app service with a JWT cookie.

---

## 1. Signup (2FA Off)

```
curl -X POST http://localhost:3000/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "password123",
    "requires2FA": false
  }'
```

---

## 2. Login (save cookie)

```
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "email": "testuser@example.com",
    "password": "password123"
  }'
```

---

## 3. Verify Token

Extract the JWT from the cookie file and verify it:

```
JWT=$(grep 'jwt' cookies.txt | awk '{print $7}')
curl -X POST http://localhost:3000/verify-token \
  -H "Content-Type: application/json" \
  -d '{"token": "'$JWT'"}'
```

---

## 4. Access App Service (with cookie)

```
curl -b cookies.txt http://localhost:8000/
```

---

## 5. Full Flow Script

You can run the entire flow with:

```
bash auth-service/test_auth_flow.sh
```

---

## Notes
- Make sure Docker and both services are running before testing.
- The script and commands assume default ports (auth: 3000, app: 8000).
- The cookie file (`cookies.txt`) is used to persist the JWT between requests.
