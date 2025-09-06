# Auth Service – Banned Token Store (JWT Revocation)

## Overview
This service now supports server-side JWT revocation using a production-ready, trait-based banned token store. Logging out will ban the JWT, and all endpoints (including `/verify-token`) will reject banned tokens.

## Implementation Summary
- **Trait-based abstraction**: `BannedTokenStore` trait for async ban/check.
- **HashSet implementation**: In-memory, thread-safe, async HashSet for banned tokens.
- **AppState integration**: Store is part of `AppState` and injected everywhere.
- **Logout/verify-token logic**: Logout bans the token; verify-token checks for ban.
- **Tests**: Full integration and unit tests for ban logic.

## How It Works
- On logout, the JWT is added to the banned token store.
- All token validation (including `/verify-token`) checks if the token is banned.
- If banned, a 401 is returned with error: `Token has been banned (revoked)`.

## How to Test
Run all tests:
```sh
cargo test --all-features
```

### Manual Test Script
1. **Signup & Login**
   - POST `/signup` with email/password
   - POST `/login` with same credentials
   - Extract JWT from `Set-Cookie`
2. **Verify Token (should succeed)**
   - POST `/verify-token` with `{ "token": "<jwt>" }` → 200
3. **Logout**
   - POST `/logout` with JWT cookie
4. **Verify Token (should fail)**
   - POST `/verify-token` with `{ "token": "<jwt>" }` → 401, error: `Token has been banned (revoked)`

## Relevant Code
- Trait: `src/domain/data_stores.rs`
- HashSet impl: `src/services/hashset_banned_token_store.rs`
- AppState: `src/lib.rs`
- Logout: `src/routes/logout.rs`
- Verify-token: `src/routes/verify_token.rs`
- Tests: `tests/api/logout.rs`, `tests/api/verify_token.rs`

## Security Note
- The in-memory store is suitable for dev/single-instance prod. For distributed deployments, implement a persistent store (e.g., Redis) via the same trait.
