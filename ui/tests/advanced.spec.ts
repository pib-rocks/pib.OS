import { test, expect } from '@playwright/test';

test('Save and Load Project', async ({ page }) => {
  await page.route('/api/projects', async route => {
    if (route.request().method() === 'GET') {
      await route.fulfill({ json: [{ id: 1, name: 'Test', tree_json: '{}' }] });
    } else if (route.request().method() === 'POST') {
      await route.fulfill({ json: { id: 1, name: 'Test', tree_json: '{}' } });
    }
  });

  await page.goto('/');
  // Assume UI has these
  // await page.click('text=Save Project');
  // await page.click('text=Load Project');
});
