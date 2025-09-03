# Postman Collection Usage & Maintenance Guide

## Using the Starter Collection

- The file `auth-service.postman_collection.json` is a ready-to-use Postman collection for your project.
- Import it into Postman (File → Import → select the JSON file).
- It covers health check, signup, and OpenAPI endpoints with basic tests.

## Keeping the Collection Up to Date

1. **After adding or changing endpoints:**
   - Rebuild and run your service.
   - Export the latest OpenAPI spec:
     ```sh
     curl http://localhost:3000/openapi.json -o openapi.json
     ```
   - In Postman, re-import the updated `openapi.json` (as a new collection or merge new endpoints into your main collection).
   - Add or update tests for new/changed endpoints in the "Tests" tab.
   - Export and overwrite `auth-service.postman_collection.json` in your repo.

2. **Automated Testing:**
   - Use Newman to run all tests:
     ```sh
     newman run auth-service.postman_collection.json
     ```
   - Add this to your CI pipeline for automated checks.

3. **Best Practices:**
   - Always keep your OpenAPI spec and Postman collection in sync with your codebase.
   - Add meaningful tests for each endpoint (status, body, error cases).
   - Version your collection if needed (e.g., `auth-service.postman_collection.v2.json`).
   - Document custom tests for your team.

## Summary for Future Sprints
- Update OpenAPI and Postman collection after every backend change.
- Add/adjust tests for new endpoints.
- Export and commit the updated collection.
- Use Newman for automated and CI testing.

---

For more details, see `API_TEST_AUTOMATION.md` in this repo.
