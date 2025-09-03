// ui-tests/delete-account.spec.ts
import { test, expect } from '@playwright/test';

test('user can sign up, log in, and delete their account', async ({ page }) => {
  const email = `ui-test-${Date.now()}@example.com`;
  const password = 'password123';

  // Go to the app
  await page.goto('/');

  // Sign up
  await page.click('#signup-link');
  await page.fill('#signup-form [name=email]', email);
  await page.fill('#signup-form [name=password]', password);
  await page.click('#signup-form-submit');
  await expect(page.locator('#login-section')).toBeVisible();

  // Log in
  await page.fill('#login-form [name=email]', email);
  await page.fill('#login-form [name=password]', password);
  await page.click('#login-form-submit');
  await expect(page.locator('#delete-account-btn')).toBeVisible();

  // Delete account
  await Promise.all([
    page.waitForEvent('dialog').then(dialog => dialog.accept()),
    page.click('#delete-account-btn'),
  ]);
  await expect(page.locator('#delete-account-btn')).toBeHidden();
  await expect(page.locator('#login-section')).toBeVisible();

  // Try logging in again (should fail)
  await page.fill('#login-form [name=email]', email);
  await page.fill('#login-form [name=password]', password);
  await page.click('#login-form-submit');
});
