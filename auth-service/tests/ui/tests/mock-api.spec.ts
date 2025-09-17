import { test, expect, Page } from '@playwright/test';

/**
 * Mock API tests that demonstrate UI functionality without requiring backend
 * These tests mock all API endpoints to verify frontend behavior in isolation
 */

test.describe('UI Functionality with Mock API', () => {
  
  test.beforeEach(async ({ page }) => {
    // Mock successful signup
    await page.route('**/signup', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'User created successfully' })
      });
    });

    // Mock successful login
    await page.route('**/login', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ 
          message: 'Login successful',
          token: 'mock-jwt-token',
          user: { email: 'test@example.com' }
        })
      });
    });

    // Mock successful forgot password
    await page.route('**/forgot-password', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Reset code sent' })
      });
    });

    // Mock successful password reset
    await page.route('**/verify-password-reset-token', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Password reset successfully' })
      });
    });

    // Mock 2FA endpoints
    await page.route('**/enable-2fa', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: '2FA enabled successfully' })
      });
    });

    await page.route('**/disable-2fa', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: '2FA disabled successfully' })
      });
    });

    await page.route('**/change-2fa-method', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: '2FA method updated' })
      });
    });

    // Mock delete account
    await page.route('**/delete-account', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Account deleted successfully' })
      });
    });
  });

  test.describe('Complete Signup Flow', () => {
    test('should complete signup journey with all UI states', async ({ page }) => {
      await page.goto('index.html');
      
      // Initial state verification
      await expect(page.locator('#app-title')).toHaveText('Live Bootcamp Rehearsal');
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#signup-link')).toBeVisible();
      
      // Navigate to signup
      await page.click('#signup-link');
      await expect(page.locator('#signup-section')).toBeVisible();
      await expect(page.locator('#login-section')).toBeHidden();
      
      // Test form validation
      await page.click('#signup-submit');
      await expect(page.locator('#signup-email')).toHaveClass(/error/);
      await expect(page.locator('#signup-password')).toHaveClass(/error/);
      
      // Fill form progressively and watch validation clear
      await page.fill('#signup-email', 'test@example.com');
      await expect(page.locator('#signup-email')).not.toHaveClass(/error/);
      
      await page.fill('#signup-password', 'SecurePassword123!');
      await expect(page.locator('#signup-password')).not.toHaveClass(/error/);
      
      // Test 2FA option
      await page.check('#signup-2fa');
      await expect(page.locator('#signup-2fa')).toBeChecked();
      
      // Submit form and verify loading state
      const submitPromise = page.click('#signup-submit');
      await expect(page.locator('#signup-submit')).toHaveClass(/btn-loading/);
      await expect(page.locator('#signup-submit')).toBeDisabled();
      
      await submitPromise;
      
      // Verify success state
      await expect(page.locator('.toast-success')).toBeVisible();
      await expect(page.locator('.toast-success')).toContainText('User created successfully');
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#signup-section')).toBeHidden();
    });

    test('should handle signup errors gracefully', async ({ page }) => {
      // Mock error response
      await page.route('**/signup', async route => {
        await route.fulfill({
          status: 409,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Email already exists' })
        });
      });

      await page.goto('index.html');
      await page.click('#signup-link');
      
      await page.fill('#signup-email', 'existing@example.com');
      await page.fill('#signup-password', 'Password123!');
      await page.click('#signup-submit');
      
      await expect(page.locator('#signup-err-alert')).toBeVisible();
      await expect(page.locator('#signup-err-alert')).toContainText('Email already exists');
      await expect(page.locator('#signup-submit')).not.toHaveClass(/btn-loading/);
      await expect(page.locator('#signup-submit')).toBeEnabled();
    });
  });

  test.describe('Login Flow', () => {
    test('should complete login with success states', async ({ page }) => {
      await page.goto('index.html');
      
      // Test validation
      await page.click('#login-submit');
      await expect(page.locator('#login-email')).toHaveClass(/error/);
      await expect(page.locator('#login-password')).toHaveClass(/error/);
      
      // Fill form
      await page.fill('#login-email', 'test@example.com');
      await page.fill('#login-password', 'password123');
      
      // Submit and verify loading
      const submitPromise = page.click('#login-submit');
      await expect(page.locator('#login-submit')).toHaveClass(/btn-loading/);
      await submitPromise;
      
      // Verify authenticated state
      await expect(page.locator('#app-header')).toBeVisible();
      await expect(page.locator('#user-email')).toContainText('test@example.com');
      await expect(page.locator('#account-settings-section')).toBeVisible();
      await expect(page.locator('#login-section')).toBeHidden();
    });

    test('should handle login errors', async ({ page }) => {
      // Mock error response
      await page.route('**/login', async route => {
        await route.fulfill({
          status: 401,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Invalid credentials' })
        });
      });

      await page.goto('index.html');
      
      await page.fill('#login-email', 'wrong@example.com');
      await page.fill('#login-password', 'wrongpassword');
      await page.click('#login-submit');
      
      await expect(page.locator('#login-err-alert')).toBeVisible();
      await expect(page.locator('#login-err-alert')).toContainText('Invalid credentials');
    });

    test('should handle 2FA flow', async ({ page }) => {
      // Mock 2FA required response
      await page.route('**/login', async route => {
        await route.fulfill({
          status: 206,
          contentType: 'application/json',
          body: JSON.stringify({ 
            message: '2FA required',
            loginAttemptId: 'attempt-123'
          })
        });
      });

      await page.goto('index.html');
      
      await page.fill('#login-email', 'user2fa@example.com');
      await page.fill('#login-password', 'password123');
      await page.click('#login-submit');
      
      // Should show 2FA section
      await expect(page.locator('#login-section')).toBeHidden();
      await expect(page.locator('#verify-2fa-section')).toBeVisible();
    });
  });

  test.describe('Password Recovery', () => {
    test('should complete forgot password flow', async ({ page }) => {
      await page.goto('index.html');
      
      // Navigate to forgot password
      await page.click('#forgot-password-link');
      await expect(page.locator('#forgot-password-section')).toBeVisible();
      await expect(page.locator('#login-section')).toBeHidden();
      
      // Step 1: Request reset code
      await page.fill('#forgot-email', 'forgot@example.com');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('.toast-success')).toBeVisible();
      await expect(page.locator('#forgot-password-step2')).toBeVisible();
      await expect(page.locator('#forgot-password-step1')).toBeHidden();
      
      // Step 2: Reset password
      await page.fill('#reset-code', 'RESET123');
      await page.fill('#new-password', 'NewPassword123!');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('.toast-success')).toBeVisible();
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#forgot-password-section')).toBeHidden();
    });

    test('should handle back navigation in forgot password', async ({ page }) => {
      await page.goto('index.html');
      
      await page.click('#forgot-password-link');
      await page.fill('#forgot-email', 'test@example.com');
      await page.click('button[type="submit"]');
      
      // Now in step 2
      await expect(page.locator('#forgot-password-step2')).toBeVisible();
      
      // Click back to login
      await page.click('#back-to-login-link');
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#forgot-password-section')).toBeHidden();
    });
  });

  test.describe('Account Settings', () => {
    test('should manage 2FA settings', async ({ page }) => {
      await page.goto('index.html');
      
      // Login first
      await page.fill('#login-email', 'user@example.com');
      await page.fill('#login-password', 'password123');
      await page.click('#login-submit');
      
      await expect(page.locator('#account-settings-section')).toBeVisible();
      
      // Enable 2FA
      const enablePromise = page.click('#2fa-toggle');
      
      // Handle confirmation dialog
      page.on('dialog', dialog => {
        expect(dialog.message()).toContain('enable 2FA');
        dialog.accept();
      });
      
      await enablePromise;
      await expect(page.locator('.toast-success')).toBeVisible();
      await expect(page.locator('#2fa-toggle')).toBeChecked();
      
      // Change 2FA method
      await page.selectOption('#2fa-method-select', 'SMS');
      await expect(page.locator('.toast-success')).toBeVisible();
      
      // Disable 2FA
      const disablePromise = page.click('#2fa-toggle');
      
      page.on('dialog', dialog => {
        expect(dialog.message()).toContain('disable 2FA');
        dialog.accept();
      });
      
      await disablePromise;
      await expect(page.locator('.toast-success')).toBeVisible();
      await expect(page.locator('#2fa-toggle')).not.toBeChecked();
    });

    test('should handle account deletion', async ({ page }) => {
      await page.goto('index.html');
      
      // Login first
      await page.fill('#login-email', 'delete@example.com');
      await page.fill('#login-password', 'password123');
      await page.click('#login-submit');
      
      await expect(page.locator('#account-settings-section')).toBeVisible();
      
      // Delete account
      const deletePromise = page.click('#delete-account-btn');
      
      // Handle confirmation dialog
      page.on('dialog', dialog => {
        expect(dialog.message()).toContain('DELETE');
        dialog.accept('DELETE');
      });
      
      await deletePromise;
      
      // Should logout and return to login
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#app-header')).toBeHidden();
      await expect(page.locator('.toast-success')).toBeVisible();
    });

    test('should handle logout', async ({ page }) => {
      await page.goto('index.html');
      
      // Login first
      await page.fill('#login-email', 'logout@example.com');
      await page.fill('#login-password', 'password123');
      await page.click('#login-submit');
      
      await expect(page.locator('#app-header')).toBeVisible();
      
      // Logout
      await page.click('#settings-login-link');
      
      // Should return to login state
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#app-header')).toBeHidden();
      await expect(page.locator('#account-settings-section')).toBeHidden();
    });
  });

  test.describe('Navigation and UI States', () => {
    test('should navigate between all sections', async ({ page }) => {
      await page.goto('index.html');
      
      // Initial: Login section
      await expect(page.locator('#login-section')).toBeVisible();
      
      // To signup
      await page.click('#signup-link');
      await expect(page.locator('#signup-section')).toBeVisible();
      await expect(page.locator('#login-section')).toBeHidden();
      
      // Back to login
      await page.click('#login-link');
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#signup-section')).toBeHidden();
      
      // To forgot password
      await page.click('#forgot-password-link');
      await expect(page.locator('#forgot-password-section')).toBeVisible();
      await expect(page.locator('#login-section')).toBeHidden();
      
      // Back to login
      await page.click('#back-to-login-link');
      await expect(page.locator('#login-section')).toBeVisible();
      await expect(page.locator('#forgot-password-section')).toBeHidden();
    });

    test('should show loading states correctly', async ({ page }) => {
      // Mock slow response
      await page.route('**/login', async route => {
        await new Promise(resolve => setTimeout(resolve, 1000));
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ message: 'Login successful' })
        });
      });

      await page.goto('index.html');
      
      await page.fill('#login-email', 'slow@example.com');
      await page.fill('#login-password', 'password123');
      
      const submitPromise = page.click('#login-submit');
      
      // Verify loading state
      await expect(page.locator('#login-submit')).toHaveClass(/btn-loading/);
      await expect(page.locator('#login-submit')).toBeDisabled();
      await expect(page.locator('#login-submit')).toContainText('Signing in...');
      
      await submitPromise;
      
      // Verify loading state cleared
      await expect(page.locator('#login-submit')).not.toHaveClass(/btn-loading/);
    });

    test('should display toast notifications correctly', async ({ page }) => {
      await page.goto('index.html');
      
      // Test success toast
      await page.click('#signup-link');
      await page.fill('#signup-email', 'toast@example.com');
      await page.fill('#signup-password', 'password123');
      await page.click('#signup-submit');
      
      const successToast = page.locator('.toast-success');
      await expect(successToast).toBeVisible();
      await expect(successToast).toContainText('User created successfully');
      
      // Toast should auto-hide after 5 seconds
      await page.waitForTimeout(5500);
      await expect(successToast).toBeHidden();
    });

    test('should handle form validation feedback', async ({ page }) => {
      await page.goto('index.html');
      
      // Test invalid email
      await page.fill('#login-email', 'invalid-email');
      await page.click('#login-password'); // Trigger blur
      await expect(page.locator('#login-email')).toHaveClass(/error/);
      
      // Fix email
      await page.fill('#login-email', 'valid@example.com');
      await page.click('#login-password'); // Trigger blur
      await expect(page.locator('#login-email')).not.toHaveClass(/error/);
      
      // Test short password
      await page.fill('#login-password', '123');
      await page.click('#login-email'); // Trigger blur
      await expect(page.locator('#login-password')).toHaveClass(/error/);
      
      // Fix password
      await page.fill('#login-password', 'validpassword123');
      await page.click('#login-email'); // Trigger blur
      await expect(page.locator('#login-password')).not.toHaveClass(/error/);
    });
  });

  test.describe('Responsive Design', () => {
    test('should work on mobile viewport', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('index.html');
      
      // Verify mobile layout
      await expect(page.locator('.auth-container')).toBeVisible();
      
      // All form elements should be accessible
      await expect(page.locator('#login-email')).toBeVisible();
      await expect(page.locator('#login-password')).toBeVisible();
      await expect(page.locator('#login-submit')).toBeVisible();
      
      // Test signup on mobile
      await page.click('#signup-link');
      await expect(page.locator('#signup-section')).toBeVisible();
      
      // Fill and submit should work
      await page.fill('#signup-email', 'mobile@example.com');
      await page.fill('#signup-password', 'password123');
      await page.click('#signup-submit');
      
      await expect(page.locator('.toast-success')).toBeVisible();
    });

    test('should work on tablet viewport', async ({ page }) => {
      await page.setViewportSize({ width: 768, height: 1024 });
      await page.goto('index.html');
      
      // Verify tablet layout
      await expect(page.locator('.auth-container')).toBeVisible();
      
      // Login and verify settings page
      await page.fill('#login-email', 'tablet@example.com');
      await page.fill('#login-password', 'password123');
      await page.click('#login-submit');
      
      await expect(page.locator('#account-settings-section')).toBeVisible();
      await expect(page.locator('#2fa-toggle')).toBeVisible();
    });
  });

  test.describe('Accessibility', () => {
    test('should have proper focus management', async ({ page }) => {
      await page.goto('index.html');
      
      // Tab through login form
      await page.keyboard.press('Tab');
      await expect(page.locator('#login-email')).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(page.locator('#login-password')).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(page.locator('#login-submit')).toBeFocused();
      
      // Test navigation with keyboard
      await page.keyboard.press('Tab');
      await expect(page.locator('#signup-link')).toBeFocused();
      
      await page.keyboard.press('Enter');
      await expect(page.locator('#signup-section')).toBeVisible();
    });

    test('should have proper ARIA attributes', async ({ page }) => {
      await page.goto('index.html');
      
      // Test form ARIA labels
      await expect(page.locator('#login-email')).toHaveAttribute('aria-label', 'Email address');
      await expect(page.locator('#login-password')).toHaveAttribute('aria-label', 'Password');
      
      // Test error states
      await page.click('#login-submit');
      await expect(page.locator('#login-email')).toHaveAttribute('aria-invalid', 'true');
    });

    test('should work with screen reader patterns', async ({ page }) => {
      await page.goto('index.html');
      
      // Navigate to signup
      await page.click('#signup-link');
      
      // Test form labels and descriptions
      await expect(page.locator('label[for="signup-email"]')).toBeVisible();
      await expect(page.locator('label[for="signup-password"]')).toBeVisible();
      await expect(page.locator('label[for="signup-2fa"]')).toBeVisible();
      
      // Test headings structure
      await expect(page.locator('h1')).toBeVisible();
      await expect(page.locator('h2')).toBeVisible();
    });
  });
});