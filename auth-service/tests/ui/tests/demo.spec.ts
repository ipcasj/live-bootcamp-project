import { test, expect } from '@playwright/test';

/**
 * Core functionality demonstration tests
 * These tests focus on the essential features with simplified selectors
 */

test.describe('Modern UI Demonstration', () => {
  
  test.beforeEach(async ({ page }) => {
    // Mock all API endpoints
    await page.route('**/signup', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Account created successfully' })
      });
    });

    await page.route('**/login', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ 
          message: 'Login successful',
          user: { email: 'demo@example.com' }
        })
      });
    });

    await page.route('**/forgot-password', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Reset code sent' })
      });
    });
  });

  test('should demonstrate modern UI design and core flows', async ({ page }) => {
    const htmlPath = 'file:///Users/igor/rust_projects/lgr/live-bootcamp-project/auth-service/assets/index.html';
    await page.goto(htmlPath);
    
    // Take initial screenshot
    await page.screenshot({ path: 'test-results/01-login-page.png', fullPage: true });
    
    // Test login form validation
    await page.click('#login-submit');
    await page.screenshot({ path: 'test-results/02-validation-errors.png', fullPage: true });
    
    // Fill login form
    await page.fill('#login-email', 'demo@example.com');
    await page.fill('#login-password', 'SecurePassword123!');
    await page.screenshot({ path: 'test-results/03-login-filled.png', fullPage: true });
    
    // Submit login (shows loading/processing state)
    await page.click('#login-submit');
    await page.waitForTimeout(500); // Allow for animation
    await page.screenshot({ path: 'test-results/04-login-processing.png', fullPage: true });
    
    // Navigate to signup
    await page.click('#signup-link');
    await page.waitForSelector('#signup-section:not(.hidden)', { timeout: 5000 });
    await page.screenshot({ path: 'test-results/05-signup-page.png', fullPage: true });
    
    // Fill signup form
    await page.fill('#signup-email', 'newuser@example.com');
    await page.fill('#signup-password', 'NewPassword123!');
    await page.check('#signup-2fa');
    await page.screenshot({ path: 'test-results/06-signup-filled.png', fullPage: true });
    
    // Submit signup
    await page.click('#signup-submit');
    await page.waitForTimeout(500); // Allow for success animation
    await page.screenshot({ path: 'test-results/07-signup-processing.png', fullPage: true });
    
    // Navigate back to login to show forgot password
    await page.click('#signup-login-link');
    await page.waitForTimeout(100);
    await page.screenshot({ path: 'test-results/08-back-to-login.png', fullPage: true });
    
    // Navigate to forgot password
    await page.click('#forgot-password-link');
    await page.waitForTimeout(100);
    await page.screenshot({ path: 'test-results/09-forgot-password.png', fullPage: true });
    
    console.log('✅ All core navigation and form interactions completed successfully');
  });

  test('should demonstrate responsive design', async ({ page }) => {
    const htmlPath = 'file:///Users/igor/rust_projects/lgr/live-bootcamp-project/auth-service/assets/index.html';
    
    // Mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto(htmlPath);
    await page.screenshot({ path: 'test-results/11-mobile-login.png', fullPage: true });
    
    await page.click('#signup-link');
    await page.screenshot({ path: 'test-results/12-mobile-signup.png', fullPage: true });
    
    // Tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto(htmlPath);
    await page.screenshot({ path: 'test-results/13-tablet-login.png', fullPage: true });
    
    // Desktop viewport
    await page.setViewportSize({ width: 1200, height: 800 });
    await page.goto(htmlPath);
    await page.screenshot({ path: 'test-results/14-desktop-login.png', fullPage: true });
  });

  test('should demonstrate form validation and error states', async ({ page }) => {
    const htmlPath = 'file:///Users/igor/rust_projects/lgr/live-bootcamp-project/auth-service/assets/index.html';
    await page.goto(htmlPath);
    
    // Test invalid email
    await page.fill('#login-email', 'invalid-email');
    await page.click('#login-password');
    await page.screenshot({ path: 'test-results/15-invalid-email.png', fullPage: true });
    
    // Test short password
    await page.fill('#login-email', 'valid@example.com');
    await page.fill('#login-password', '123');
    await page.click('#login-email');
    await page.screenshot({ path: 'test-results/16-short-password.png', fullPage: true });
  });

  test('should demonstrate loading states', async ({ page }) => {
    const htmlPath = 'file:///Users/igor/rust_projects/lgr/live-bootcamp-project/auth-service/assets/index.html';
    await page.goto(htmlPath);
    await page.click('#signup-link');
    
    await page.fill('#signup-email', 'loading@example.com');
    await page.fill('#signup-password', 'password123');
    
    // Click submit and immediately take screenshot to capture loading state
    const submitPromise = page.click('#signup-submit');
    await page.waitForTimeout(100); // Brief wait to capture loading state
    await page.screenshot({ path: 'test-results/18-loading-state.png', fullPage: true });
    
    await submitPromise;
    await page.screenshot({ path: 'test-results/19-after-loading.png', fullPage: true });
  });

  test('should demonstrate toast notifications', async ({ page }) => {
    const htmlPath = 'file:///Users/igor/rust_projects/lgr/live-bootcamp-project/auth-service/assets/index.html';
    await page.goto(htmlPath);
    
    // Signup to trigger success toast
    await page.click('#signup-link');
    await page.waitForSelector('#signup-section:not(.hidden)', { timeout: 5000 });
    await page.fill('#signup-email', 'toast@example.com');
    await page.fill('#signup-password', 'password123');
    await page.click('#signup-submit');
    
    await page.waitForTimeout(500);
    await page.screenshot({ path: 'test-results/20-success-toast.png', fullPage: true });
  });
});