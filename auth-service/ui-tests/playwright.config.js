// playwright.config.js
// Basic Playwright config for running UI tests
/** @type {import('@playwright/test').PlaywrightTestConfig} */
module.exports = {
  webServer: {
    command: 'cargo run --bin auth-service',
    port: 3000,
    timeout: 120 * 1000,
    reuseExistingServer: !process.env.CI,
  },
  use: {
    baseURL: 'http://localhost:3000',
    headless: true,
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
};
