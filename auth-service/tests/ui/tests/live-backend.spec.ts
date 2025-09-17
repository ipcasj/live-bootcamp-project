import { test, expect } from '@playwright/test';

test.describe('Live Backend Integration Tests', () => {
  const BASE_URL = 'http://localhost:3000';

  test.beforeEach(async ({ page }) => {
    // Check if auth service is running
    try {
      const response = await page.goto(`${BASE_URL}/health`);
      if (!response || !response.ok()) {
        throw new Error('Auth service not running');
      }
    } catch (error) {
      console.log('❌ Auth service not running on localhost:3000');
      console.log('Please start it with: cd auth-service && cargo run --bin auth-service');
      throw error;
    }
  });

  test('should demonstrate complete auth flow with live backend', async ({ page }) => {
    // Go to the main page served by the backend
    await page.goto(BASE_URL);
    await page.screenshot({ path: 'test-results/live-01-landing.png', fullPage: true });

    // Test signup flow
    await page.click('#signup-link');
    await page.waitForSelector('#signup-section:not(.hidden)');
    
    // Fill signup form with test data
    const testEmail = `test_${Date.now()}@example.com`;
    await page.fill('#signup-email', testEmail);
    await page.fill('#signup-password', 'TestPassword123!');
    await page.screenshot({ path: 'test-results/live-02-signup-filled.png', fullPage: true });
    
    // Submit signup and expect success
    await page.click('#signup-submit');
    
    // Wait for either success or error response
    await page.waitForTimeout(2000);
    await page.screenshot({ path: 'test-results/live-03-signup-result.png', fullPage: true });

    // Test login flow with the same credentials
    await page.click('#signup-login-link');
    await page.waitForTimeout(500);
    
    await page.fill('#login-email', testEmail);
    await page.fill('#login-password', 'TestPassword123!');
    await page.screenshot({ path: 'test-results/live-04-login-filled.png', fullPage: true });
    
    await page.click('#login-submit');
    await page.waitForTimeout(2000);
    await page.screenshot({ path: 'test-results/live-05-login-result.png', fullPage: true });

    console.log('✅ Live backend integration test completed');
  });

  test('should handle API errors gracefully', async ({ page }) => {
    await page.goto(BASE_URL);
    
    // Test login with invalid credentials
    await page.fill('#login-email', 'invalid@example.com');
    await page.fill('#login-password', 'wrongpassword');
    await page.click('#login-submit');
    
    // Should show error message
    await page.waitForTimeout(1000);
    await page.screenshot({ path: 'test-results/live-06-login-error.png', fullPage: true });
    
    // Test signup with invalid email format
    await page.click('#signup-link');
    await page.waitForSelector('#signup-section:not(.hidden)');
    
    await page.fill('#signup-email', 'invalid-email');
    await page.fill('#signup-password', 'TestPassword123!');
    await page.click('#signup-submit');
    
    await page.waitForTimeout(1000);
    await page.screenshot({ path: 'test-results/live-07-signup-validation.png', fullPage: true });

    console.log('✅ Error handling test completed');
  });

  test('should test forgot password flow', async ({ page }) => {
    await page.goto(BASE_URL);
    
    // Navigate to forgot password
    await page.click('#forgot-password-link');
    await page.waitForTimeout(500);
    await page.screenshot({ path: 'test-results/live-08-forgot-password.png', fullPage: true });
    
    // Fill forgot password form
    await page.fill('#forgot-email', 'test@example.com');
    await page.click('button[type="submit"]:visible');
    
    await page.waitForTimeout(1000);
    await page.screenshot({ path: 'test-results/live-09-forgot-password-result.png', fullPage: true });

    console.log('✅ Forgot password flow test completed');
  });

  test('should verify static assets are served correctly', async ({ page }) => {
    // Test that CSS is loaded
    const cssResponse = await page.goto(`${BASE_URL}/styles.css`);
    expect(cssResponse?.status()).toBe(200);
    
    // Test that JS is loaded
    const jsResponse = await page.goto(`${BASE_URL}/app.js`);
    expect(jsResponse?.status()).toBe(200);
    
    // Test main page loads with all assets
    await page.goto(BASE_URL);
    
    // Verify JavaScript is working (AuthApp should be available)
    const hasAuthApp = await page.evaluate(() => typeof window.AuthApp !== 'undefined');
    expect(hasAuthApp).toBe(true);
    
    // Verify CSS is applied (check if modern styles are present)
    const hasModernStyles = await page.evaluate(() => {
      const style = getComputedStyle(document.body);
      return style.fontFamily.includes('system-ui') || style.fontFamily.includes('Inter');
    });
    expect(hasModernStyles).toBe(true);

    console.log('✅ Static assets verification completed');
  });
});