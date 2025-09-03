# Postman & Newman API Test Automation for auth-service

## 1. Export OpenAPI Spec

Run this command to export your OpenAPI spec from the running service:

```
curl http://localhost:3000/openapi.json -o openapi.json
```

## 2. Import OpenAPI Spec into Postman

- Open Postman.
- Click "Import" → "File" and select `openapi.json`.
- Postman will generate a collection with all endpoints.

## 3. Add Pre-configured Tests in Postman

For each request, go to the "Tests" tab and add:

```js
pm.test("Status code is 200", function () {
    pm.response.to.have.status(200);
});
pm.test("Response has JSON body", function () {
    pm.response.to.be.json;
});
```

You can add more checks for fields, error codes, etc.

## 4. Export Postman Collection

- Click the collection name → "Export" → Collection v2.1 (recommended).
- Save as `auth-service.postman_collection.json` in your project root.

## 5. Install Newman (CLI)

```
npm install -g newman
```

## 6. Run Automated API Tests

```
newman run auth-service.postman_collection.json
```

## 7. (Optional) Automate on File Change

Install `cargo-watch` if you want to re-run tests after every build:

```
cargo install cargo-watch
```

Then run:

```
cargo watch -x 'run' -s 'newman run auth-service.postman_collection.json'
```

---

**Summary:**
- Export OpenAPI → Import to Postman → Add tests → Export collection → Run with Newman → (Optional: automate with cargo-watch)

See the Postman and Newman docs for advanced usage and CI integration.
