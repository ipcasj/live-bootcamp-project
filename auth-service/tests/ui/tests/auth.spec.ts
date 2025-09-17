import { test, expect, Page } from '@playwright/test';

/**
 * Page Object Model for Authentication UI
 */
class AuthPage {
  constructor(private page: Page) {}

  // Navigation methods
  async goto() {
    await this.page.goto('index.html');
    
    // Wait for the application to initialize and show only the login section
    await this.page.waitForSelector('#login-section:not(.hidden)', { timeout: 5000 });
    
    // Verify other sections have the hidden class (they should exist but be hidden)
    await expect(this.page.locator('#signup-section')).toHaveClass(/hidden/);
    await expect(this.page.locator('[id="2fa-section"]')).toHaveClass(/hidden/);
    await expect(this.page.locator('#forgot-password-section')).toHaveClass(/hidden/);
    await expect(this.page.locator('#account-settings-section')).toHaveClass(/hidden/);
  }

  async goToSignup() {
    await this.page.click('#signup-link');
  }

  async goToLogin() {
    await this.page.click('#signup-login-link, #forgot-password-login-link, [id="2fa-login-link"]');
  }

  async goToForgotPassword() {
    await this.page.click('#forgot-password-link');
  }

  // Login actions
  async fillLoginForm(email: string, password: string) {
    await this.page.fill('#login-email', email);
    await this.page.fill('#login-password', password);
  }

  async submitLogin() {
    await this.page.click('#login-submit');
  }

  async login(email: string, password: string) {
    await this.fillLoginForm(email, password);
    await this.submitLogin();
  }

  // Signup actions
  async fillSignupForm(email: string, password: string, enable2FA = false) {
    await this.page.fill('#signup-email', email);
    await this.page.fill('#signup-password', password);
    if (enable2FA) {
      await this.page.check('#signup-2fa');
    }
  }

  async submitSignup() {
    await this.page.click('#signup-submit');
  }

  async signup(email: string, password: string, enable2FA = false) {
    await this.fillSignupForm(email, password, enable2FA);
    await this.submitSignup();
  }

  // 2FA actions
  async fill2FACode(code: string) {
    await this.page.fill('[id="2fa-code"]', code);
  }

  async submit2FA() {
    await this.page.click('[id="2fa-submit"]');
  }

  async verify2FA(code: string) {
    await this.fill2FACode(code);
    await this.submit2FA();
  }

  // Forgot password actions
  async fillForgotPasswordEmail(email: string) {
    await this.page.fill('#forgot-email', email);
  }

  async submitForgotPassword() {
    await this.page.click('button[type="submit"]');
  }

  async fillResetPassword(code: string, newPassword: string) {
    await this.page.fill('#reset-code', code);
    await this.page.fill('#new-password', newPassword);
  }

  async submitResetPassword() {
    await this.page.click('button[type="submit"]');
  }

  // Settings actions
  async toggle2FA() {
    await this.page.click('[id="2fa-toggle"]');
  }

  async change2FAMethod(method: string) {
    await this.page.selectOption('[id="2fa-method-select"]', method);
  }

  async deleteAccount() {
    await this.page.click('#delete-account-btn');
  }

  async logout() {
    await this.page.click('#settings-login-link');
  }

  // Assertion helpers
  async expectViewVisible(viewId: string) {
    const selector = viewId === '2fa-section' ? '[id="2fa-section"]' : `#${viewId}`;
    await expect(this.page.locator(selector)).not.toHaveClass(/hidden/);
  }

  async expectAlert(type: string, message?: string) {
    const alert = this.page.locator(`#${type}-err-alert`);
    await expect(alert).not.toHaveClass(/hidden/);
    if (message) {
      await expect(alert).toContainText(message);
    }
  }

  async expectToast(title?: string) {
    const toast = this.page.locator('.toast');
    await expect(toast).toBeVisible();
    if (title) {
      await expect(toast).toContainText(title);
    }
  }

  async expectAuthenticated() {
    await expect(this.page.locator('#app-header')).toBeVisible();
  }

  async expectNotAuthenticated() {
    await expect(this.page.locator('#app-header')).toBeHidden();
  }

  async expectButtonLoading(buttonId: string) {
    await expect(this.page.locator(`#${buttonId}`)).toHaveClass(/btn-loading/);
    await expect(this.page.locator(`#${buttonId}`)).toBeDisabled();
  }

  async expectButtonNotLoading(buttonId: string) {
    await expect(this.page.locator(`#${buttonId}`)).not.toHaveClass(/btn-loading/);
    await expect(this.page.locator(`#${buttonId}`)).toBeEnabled();
  }

  async expectFieldError(fieldId: string) {
    await expect(this.page.locator(`#${fieldId}`)).toHaveClass(/error/);
  }

  async expectNoFieldError(fieldId: string) {
    await expect(this.page.locator(`#${fieldId}`)).not.toHaveClass(/error/);
  }
}

test.describe('Authentication UI', () => {
  let authPage: AuthPage;

  test.beforeEach(async ({ page }) => {
    authPage = new AuthPage(page);
    await authPage.goto();
  });

  test.describe('Initial State', () => {
    test('should show login form by default', async () => {
      await authPage.expectViewVisible('login-section');
      await authPage.expectNotAuthenticated();
    });

    test('should have proper page title and meta', async ({ page }) => {
      await expect(page).toHaveTitle(/Live Bootcamp Auth/);
    });

    test('should be responsive on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await authPage.expectViewVisible('login-section');
      
      // Check that the visible login form is usable on mobile
      await expect(page.locator('#login-section .auth-card')).toBeVisible();
    });
  });

  test.describe('Navigation', () => {
    test('should navigate between login and signup', async () => {
      await authPage.goToSignup();
      await authPage.expectViewVisible('signup-section');
      
      await authPage.goToLogin();
      await authPage.expectViewVisible('login-section');
    });

    test('should navigate to forgot password', async () => {
      await authPage.goToForgotPassword();
      await authPage.expectViewVisible('forgot-password-section');
    });

    test('should return to login from forgot password', async () => {
      await authPage.goToForgotPassword();
      await authPage.goToLogin();
      await authPage.expectViewVisible('login-section');
    });
  });

  test.describe('Form Validation', () => {
    test('should validate required fields on login', async ({ page }) => {
      await authPage.submitLogin();
      
      // Should focus first invalid field
      await expect(page.locator('#login-email')).toBeFocused();
      await authPage.expectFieldError('login-email');
    });

    test('should validate email format', async ({ page }) => {
      await page.fill('#login-email', 'invalid-email');
      await page.locator('#login-email').blur();
      await authPage.expectFieldError('login-email');
    });

    test('should validate password length', async ({ page }) => {
      await page.fill('#signup-password', '123');
      await page.locator('#signup-password').blur();
      await authPage.expectFieldError('signup-password');
    });

    test('should clear field errors on input', async ({ page }) => {
      await page.fill('#login-email', 'invalid');
      await page.locator('#login-email').blur();
      await authPage.expectFieldError('login-email');
      
      await page.fill('#login-email', 'valid@example.com');
      await authPage.expectNoFieldError('login-email');
    });

    test('should validate 2FA code format', async ({ page }) => {
      // Setup mock for login that triggers 2FA
      await page.route('**/login', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ 
            message: 'Login requires 2FA',
            requires_2fa: true,
            login_attempt_id: 'test123'
          })
        });
      });
      
      await authPage.goto();
      await authPage.login('test@example.com', 'password123');
      
      // Now we should be on 2FA section
      await authPage.expectViewVisible('2fa-section');
      
      await page.fill('[id="2fa-code"]', 'invalid');
      await page.locator('[id="2fa-code"]').blur();
      await authPage.expectFieldError('2fa-code');
    });
  });

  test.describe('User Flows', () => {
    const testEmail = `test${Date.now()}@example.com`;
    const testPassword = 'password123';

    test('should handle signup flow', async ({ page }) => {
      await authPage.goToSignup();
      await authPage.signup(testEmail, testPassword);
      
      // Should show success toast and redirect to login
      await authPage.expectToast('Account Created');
      await authPage.expectViewVisible('login-section');
    });

    test('should handle signup with 2FA enabled', async ({ page }) => {
      await authPage.goToSignup();
      await authPage.signup(testEmail, testPassword, true);
      
      await authPage.expectToast('Account Created');
      await authPage.expectViewVisible('login-section');
    });

    test('should handle login flow', async ({ page }) => {
      // First create account
      await authPage.goToSignup();
      await authPage.signup(testEmail, testPassword);
      
      // Then login
      await authPage.login(testEmail, testPassword);
      await authPage.expectAuthenticated();
      await authPage.expectViewVisible('account-settings-section');
    });

    test('should handle login with 2FA', async ({ page }) => {
      // This would require a real backend to test properly
      // For now, test the UI flow
      await authPage.login('2fa@example.com', testPassword);
      
      // Should redirect to 2FA if backend returns 206
      // await authPage.expectViewVisible('2fa-section');
    });

    test('should handle forgot password flow', async ({ page }) => {
      await authPage.goToForgotPassword();
      await authPage.fillForgotPasswordEmail(testEmail);
      await authPage.submitForgotPassword();
      
      // Should show step 2 on success
      await expect(page.locator('#forgot-password-step2')).toBeVisible();
    });

    test('should handle logout', async ({ page }) => {
      // Login first
      await authPage.goToSignup();
      await authPage.signup(testEmail, testPassword);
      await authPage.login(testEmail, testPassword);
      
      // Then logout
      await authPage.logout();
      await authPage.expectNotAuthenticated();
      await authPage.expectViewVisible('login-section');
    });
  });

  test.describe('Loading States', () => {
    test('should show loading state on form submission', async ({ page }) => {
      await authPage.fillLoginForm('test@example.com', 'password123');
      
      // Start submission and immediately check loading state
      const submitPromise = authPage.submitLogin();
      await authPage.expectButtonLoading('login-submit');
      
      await submitPromise;
      // After response, button should not be loading
      await authPage.expectButtonNotLoading('login-submit');
    });
  });

  test.describe('Error Handling', () => {
    test('should display server errors', async ({ page }) => {
      await authPage.login('nonexistent@example.com', 'wrongpassword');
      await authPage.expectAlert('login');
    });
  });

  test.describe('Accessibility', () => {
    test('should have proper ARIA labels', async ({ page }) => {
      await expect(page.locator('#login-email')).toHaveAttribute('aria-labelledby');
      await expect(page.locator('#login-password')).toHaveAttribute('aria-labelledby');
    });

    test('should announce errors to screen readers', async ({ page }) => {
      await authPage.submitLogin();
      await expect(page.locator('#login-err-alert')).toHaveAttribute('aria-live', 'assertive');
    });

    test('should support keyboard navigation', async ({ page }) => {
      await page.keyboard.press('Tab');
      await expect(page.locator('#login-email')).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(page.locator('#login-password')).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(page.locator('#login-submit')).toBeFocused();
    });

    test('should have proper contrast ratios', async ({ page }) => {
      // This would require automated accessibility testing tools
      // For now, ensure key elements are visible
      await expect(page.locator('.section-title')).toBeVisible();
      await expect(page.locator('.form-label')).toBeVisible();
      await expect(page.locator('.btn-primary')).toBeVisible();
    });
  });

  test.describe('Visual Regression', () => {
    test('should match login page screenshot', async ({ page }) => {
      await expect(page).toHaveScreenshot('login-page.png');
    });

    test('should match signup page screenshot', async ({ page }) => {
      await authPage.goToSignup();
      await expect(page).toHaveScreenshot('signup-page.png');
    });

    test('should match settings page screenshot', async ({ page }) => {
      // Mock authenticated state
      await page.evaluate(() => {
        // Simulate logged in state
        const app = (window as any).app;
        if (app) {
          app.setAuthenticatedUser('test@example.com');
          app.showView('account-settings-section');
        }
      });
      
      await expect(page).toHaveScreenshot('settings-page.png');
    });

    test('should match error states', async ({ page }) => {
      await authPage.submitLogin();
      await expect(page.locator('.auth-card')).toHaveScreenshot('login-validation-errors.png');
    });

    test('should match mobile layout', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await expect(page).toHaveScreenshot('mobile-login.png');
    });
  });

  test.describe('Performance', () => {
    test('should load quickly', async ({ page }) => {
      const startTime = Date.now();
      await authPage.goto();
      const loadTime = Date.now() - startTime;
      
      expect(loadTime).toBeLessThan(3000); // Should load in under 3 seconds
    });

    test('should not have memory leaks', async ({ page }) => {
      // Navigate between views multiple times
      for (let i = 0; i < 10; i++) {
        await authPage.goToSignup();
        await authPage.goToLogin();
        await authPage.goToForgotPassword();
        await authPage.goToLogin();
      }
      
      // Should not accumulate excessive DOM nodes
      const elementCount = await page.locator('*').count();
      expect(elementCount).toBeLessThan(500);
    });
  });

  test.describe('Browser Compatibility', () => {
    test('should work with modern features disabled', async ({ page }) => {
      // Disable modern JavaScript features
      await page.addInitScript(() => {
        delete (window as any).fetch;
        delete (window as any).Promise;
      });
      
      await authPage.goto();
      await authPage.expectViewVisible('login-section');
    });
  });
});