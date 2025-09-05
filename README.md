## Quick Integration Test (REST & gRPC)
To verify all REST and gRPC endpoints after any change, run:

```sh
cd auth-service
cargo test --test api -- --nocapture
```
This runs all REST and gRPC regression tests. All tests should pass for a healthy project.

**gRPC regression tests:**
- See `tests/api/grpc_regression.rs` for gRPC endpoint coverage.
- The gRPC server must be running and listening on `127.0.0.1:50051` for these tests to pass (adjust address in the test if needed).
## Quick Test Commands (Auth Service)
After starting the auth-service, you can quickly verify all features are working with these commands:

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
## Quick Test Commands
See [`auth-service/QUICK_TEST.md`](auth-service/QUICK_TEST.md) for commands to quickly verify the API is working (health, OpenAPI, signup, graceful shutdown).
## Setup & Building
```bash
cargo install cargo-watch
cd app-service
cargo build
cd ..
cd auth-service
cargo build
cd ..
```

## Run servers locally (Manually)
#### App service
```bash
cd app-service
cargo watch -q -c -w src/ -w assets/ -w templates/ -x run
```

visit http://localhost:8000

#### Auth service
```bash
cd auth-service
cargo watch -q -c -w src/ -w assets/ -x run
```

visit http://localhost:3000


## Run servers locally (Docker)
```bash
./docker.sh
```

visit http://localhost:8000 and http://localhost:3000

or from http://localhost:8000 click the 'Log in' in the right top corner to open http://localhost:3000