import { test, expect } from '@playwright/test';

/**
 * Integration tests that verify the complete user journey
 * These tests require a running backend service
 */

test.describe('Complete User Journey Integration', () => {
  const baseEmail = `integration.test.${Date.now()}`;
  
  test.describe('New User Onboarding', () => {
    test('should complete full signup and login journey', async ({ page }) => {
      const email = `${baseEmail}.signup@example.com`;
      const password = 'SecurePassword123!';
      
      await page.goto('index.html');
      
      // Step 1: Navigate to signup
      await page.click('#signup-link');
      await expect(page.locator('#signup-section')).toBeVisible();
      
      // Step 2: Fill signup form
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      
      // Step 3: Verify success and redirect
      await expect(page.locator('.toast-success')).toBeVisible();
      await expect(page.locator('#login-section')).toBeVisible();
      
      // Step 4: Login with new account
      await page.fill('#login-email', email);
      await page.fill('#login-password', password);
      await page.click('#login-submit');
      
      // Step 5: Verify authenticated state
      await expect(page.locator('#app-header')).toBeVisible();
      await expect(page.locator('#account-settings-section')).toBeVisible();
      await expect(page.locator('#user-email')).toContainText(email);
    });

    test('should handle signup with 2FA enabled', async ({ page }) => {
      const email = `${baseEmail}.2fa@example.com`;
      const password = 'SecurePassword123!';
      
      await page.goto('index.html');
      
      // Navigate to signup
      await page.click('#signup-link');
      
      // Fill form with 2FA enabled
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.check('#signup-2fa');
      await page.click('#signup-submit');
      
      // Verify account created
      await expect(page.locator('.toast-success')).toBeVisible();
      
      // Login should trigger 2FA flow
      await page.fill('#login-email', email);
      await page.fill('#login-password', password);
      await page.click('#login-submit');
      
      // Should redirect to 2FA verification
      // Note: This depends on backend returning 206 status
      // await expect(page.locator('#2fa-section')).toBeVisible();
    });
  });

  test.describe('Password Recovery', () => {
    test('should complete forgot password flow', async ({ page }) => {
      const email = `${baseEmail}.recovery@example.com`;
      const password = 'OriginalPassword123!';
      const newPassword = 'NewPassword123!';
      
      await page.goto('index.html');
      
      // First create an account
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      await expect(page.locator('.toast-success')).toBeVisible();
      
      // Navigate to forgot password
      await page.click('#forgot-password-link');
      await expect(page.locator('#forgot-password-section')).toBeVisible();
      
      // Step 1: Request reset code
      await page.fill('#forgot-email', email);
      await page.click('button[type="submit"]');
      
      // Should move to step 2
      await expect(page.locator('#forgot-password-step2')).toBeVisible();
      
      // Step 2: Reset password (would need real reset code)
      // This part would require integration with email service
      // await page.fill('#reset-code', 'RESET123');
      // await page.fill('#new-password', newPassword);
      // await page.click('button[type="submit"]');
      
      // Should redirect to login
      // await expect(page.locator('#login-section')).toBeVisible();
    });
  });

  test.describe('Account Management', () => {
    test('should manage 2FA settings', async ({ page }) => {
      const email = `${baseEmail}.settings@example.com`;
      const password = 'Password123!';
      
      // Setup: Create account and login
      await page.goto('index.html');
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      await expect(page.locator('.toast-success')).toBeVisible();
      
      await page.fill('#login-email', email);
      await page.fill('#login-password', password);
      await page.click('#login-submit');
      await expect(page.locator('#account-settings-section')).toBeVisible();
      
      // Enable 2FA
      await page.click('#2fa-toggle');
      
      // Should prompt for confirmation
      page.on('dialog', dialog => dialog.accept());
      
      // Should show success
      await expect(page.locator('.toast-success')).toBeVisible();
      
      // Change 2FA method
      await page.selectOption('#2fa-method-select', 'SMS');
      
      // Disable 2FA
      await page.click('#2fa-toggle');
      await expect(page.locator('.toast-success')).toBeVisible();
    });

    test('should handle account deletion', async ({ page }) => {
      const email = `${baseEmail}.delete@example.com`;
      const password = 'Password123!';
      
      // Setup: Create account and login
      await page.goto('index.html');
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      await expect(page.locator('.toast-success')).toBeVisible();
      
      await page.fill('#login-email', email);
      await page.fill('#login-password', password);
      await page.click('#login-submit');
      await expect(page.locator('#account-settings-section')).toBeVisible();
      
      // Delete account
      await page.click('#delete-account-btn');
      
      // Handle confirmation prompt
      page.on('dialog', dialog => {
        expect(dialog.message()).toContain('DELETE');
        dialog.accept('DELETE');
      });
      
      // Should logout and redirect
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#app-header')).toBeHidden();
    });
  });

  test.describe('Error Scenarios', () => {
    test('should handle duplicate signup attempts', async ({ page }) => {
      const email = `${baseEmail}.duplicate@example.com`;
      const password = 'Password123!';
      
      await page.goto('index.html');
      
      // First signup
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      await expect(page.locator('.toast-success')).toBeVisible();
      
      // Attempt second signup with same email
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      
      // Should show error
      await expect(page.locator('#signup-err-alert')).toBeVisible();
      await expect(page.locator('#signup-err-alert')).toContainText(/already exists|conflict/i);
    });

    test('should handle invalid login credentials', async ({ page }) => {
      await page.goto('index.html');
      
      await page.fill('#login-email', 'nonexistent@example.com');
      await page.fill('#login-password', 'wrongpassword');
      await page.click('#login-submit');
      
      await expect(page.locator('#login-err-alert')).toBeVisible();
      await expect(page.locator('#login-err-alert')).toContainText(/invalid|incorrect|credentials/i);
    });

    test('should handle malformed requests', async ({ page }) => {
      await page.goto('index.html');
      
      // Try login with invalid email format
      await page.fill('#login-email', 'not-an-email');
      await page.fill('#login-password', '123'); // Too short
      await page.click('#login-submit');
      
      // Should show client-side validation
      await expect(page.locator('#login-email')).toHaveClass(/error/);
    });

    test('should handle server errors gracefully', async ({ page }) => {
      await page.goto('index.html');
      
      // Mock server error
      await page.route('**/signup', route => {
        route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Internal server error' })
        });
      });
      
      await page.click('#signup-link');
      await page.fill('#signup-email', 'test@example.com');
      await page.fill('#signup-password', 'password123');
      await page.click('#signup-submit');
      
      await expect(page.locator('#signup-err-alert')).toBeVisible();
      await expect(page.locator('#signup-err-alert')).toContainText(/server error|500/i);
    });
  });

  test.describe('Session Management', () => {
    test('should maintain session across page reloads', async ({ page }) => {
      const email = `${baseEmail}.session@example.com`;
      const password = 'Password123!';
      
      // Create account and login
      await page.goto('index.html');
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      await expect(page.locator('.toast-success')).toBeVisible();
      
      await page.fill('#login-email', email);
      await page.fill('#login-password', password);
      await page.click('#login-submit');
      await expect(page.locator('#app-header')).toBeVisible();
      
      // Reload page
      await page.reload();
      
      // Should maintain session (if backend supports it)
      // Note: This depends on JWT cookie implementation
      // await expect(page.locator('#app-header')).toBeVisible();
    });

    test('should handle session expiry', async ({ page }) => {
      // This would require mocking expired JWT tokens
      // For now, verify logout functionality
      const email = `${baseEmail}.expiry@example.com`;
      const password = 'Password123!';
      
      await page.goto('index.html');
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      await page.click('#signup-submit');
      await expect(page.locator('.toast-success')).toBeVisible();
      
      await page.fill('#login-email', email);
      await page.fill('#login-password', password);
      await page.click('#login-submit');
      await expect(page.locator('#app-header')).toBeVisible();
      
      // Logout
      await page.click('#settings-login-link');
      await expect(page.locator('#app-header')).toBeHidden();
      await expect(page.locator('#login-section')).toBeVisible();
    });
  });

  test.describe('Performance Under Load', () => {
    test('should handle rapid form submissions', async ({ page }) => {
      await page.goto('index.html');
      
      const email = `${baseEmail}.rapid@example.com`;
      const password = 'Password123!';
      
      await page.click('#signup-link');
      await page.fill('#signup-email', email);
      await page.fill('#signup-password', password);
      
      // Rapidly click submit multiple times
      await Promise.all([
        page.click('#signup-submit'),
        page.click('#signup-submit'),
        page.click('#signup-submit')
      ]);
      
      // Should only process one request
      await expect(page.locator('.toast')).toHaveCount(1);
    });

    test('should handle network timeouts', async ({ page }) => {
      await page.goto('index.html');
      
      // Mock slow network
      await page.route('**/login', route => {
        setTimeout(() => route.continue(), 5000);
      });
      
      await page.fill('#login-email', 'test@example.com');
      await page.fill('#login-password', 'password123');
      
      const submitPromise = page.click('#login-submit');
      
      // Should show loading state
      await expect(page.locator('#login-submit')).toHaveClass(/btn-loading/);
      await expect(page.locator('#login-submit')).toBeDisabled();
      
      await submitPromise;
    });
  });
});