# Auth Service UI-Backend Integration Guide

## Overview
This guide explains how to verify that the UI and backend work together correctly.

## Backend Setup

### 1. Prerequisites
- Rust installed
- Environment variables configured (already in `.env`)

### 2. Running the Auth Service
```bash
cd auth-service
cargo run --bin auth-service
```

The service will start on `http://localhost:3000`

**Note:** If you get an Xcode license error, you have two options:
1. Accept the license: `sudo xcodebuild -license`
2. Use Docker: `docker compose up auth-service`

### 3. Verify Service is Running
```bash
curl http://localhost:3000/health
```

Should return: `{"status":"ok"}`

## UI-Backend Integration

### API Endpoints Verification

The UI (app.js) expects these endpoints, which are provided by the backend:

| UI Call | Backend Route | Method | Status |
|---------|---------------|--------|---------|
| `/login` | `/login` | POST | ✅ Match |
| `/signup` | `/signup` | POST | ✅ Match |
| `/forgot-password` | `/forgot-password` | POST | ✅ Match |
| `/reset-password` | `/reset-password` | POST | ✅ Match |
| `/verify-2fa` | `/verify-2fa` | POST | ✅ Match |
| `/account/settings` | `/account/settings` | PATCH | ✅ Match |
| `/delete-account` | `/delete-account` | DELETE | ✅ Match |

### Static File Serving

The backend serves static files from the `/assets` directory:
- `http://localhost:3000/` → serves `assets/index.html`
- `http://localhost:3000/app.js` → serves `assets/app.js`
- `http://localhost:3000/styles.css` → serves `assets/styles.css`

## Testing the Integration

### 1. Update Playwright Tests for Live Backend

Update the test files to use `http://localhost:3000` instead of file:// URLs:

```javascript
// In tests/demo.spec.ts
await page.goto('http://localhost:3000');
```

### 2. Run End-to-End Tests

```bash
cd tests/ui
npm install
npx playwright test tests/demo.spec.ts --project=chromium
```

### 3. Manual Testing

1. Start the auth service: `cargo run --bin auth-service`
2. Open browser to `http://localhost:3000`
3. Test the flows:
   - Registration with email/password
   - Login with credentials
   - Form validation
   - Navigation between sections

## Authentication Flow Testing

### Expected API Responses

1. **Signup Success:**
```json
{
  "message": "Account created successfully"
}
```

2. **Login Success:**
```json
{
  "message": "Login successful"
}
```

3. **Error Response:**
```json
{
  "code": "USER_ALREADY_EXISTS",
  "error": "User already exists",
  "trace_id": "uuid"
}
```

## Troubleshooting

### Port 3000 Already in Use
```bash
lsof -ti:3000
kill -9 <PID>
```

### Xcode License Issue
```bash
sudo xcodebuild -license
```

### CORS Issues
The backend is configured to allow all origins (`*`) by default. If needed, set:
```bash
export CORS_ALLOWED_ORIGINS="http://localhost:3000,http://127.0.0.1:3000"
```

## Development Workflow

1. **Backend Changes:** Restart `cargo run --bin auth-service`
2. **UI Changes:** Refresh browser (static files served from assets/)
3. **Test Changes:** Re-run `npx playwright test`

## Production Considerations

- Set proper CORS origins
- Use HTTPS
- Configure proper JWT secrets
- Set up proper email service for 2FA