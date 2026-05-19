import { test, expect } from '@playwright/test';

// スモークテスト: dev サーバが立ち上がり、未認証なら /login にリダイレクトされること
test('未認証はログイン画面にリダイレクトされる', async ({ page }) => {
  await page.goto('/');
  await page.waitForURL(/\/login/);
  await expect(page.getByText('WNAV ログイン')).toBeVisible();
});
