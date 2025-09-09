# Auth Service – Project Notes & TODO

## Planned Improvements

### Security & Production Hardening
- [ ] Use strong password hashing (already done: Argon2)
- [ ] Add per-IP and per-user rate limiting (login/signup)
- [ ] Implement account lockout after repeated failed logins
- [ ] Integrate real email/SMS delivery for 2FA codes
- [ ] Add audit logging for all authentication events

### User Experience
- [ ] Implement password reset flow (with email link)
- [ ] Require email verification on signup
- [ ] Add session management with refresh tokens

### API & Extensibility
- [ ] Expand OpenAPI docs for all endpoints and error responses
- [ ] Ensure gRPC parity for all REST features
- [ ] Add admin endpoints (list, delete, ban users)

### Testing & Observability
- [ ] Add property-based and chaos testing
- [ ] Integrate Prometheus metrics and distributed tracing

### Architecture
- [ ] Make user/token/2FA stores pluggable (e.g., Redis, Postgres)
- [ ] Make password/2FA policies configurable

---

Add notes, progress, or links below as you work on each item.
