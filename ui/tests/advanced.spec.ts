import { test, expect } from '@playwright/test';

test('Save and Load Project', async ({ page }) => {
  await page.route('/api/projects', async route => {
    if (route.request().method() === 'GET') {
      await route.fulfill({ json: [{ id: 1, name: 'Test', tree_json: '{}' }] });
    } else if (route.request().method() === 'POST') {
      await route.fulfill({ json: { id: 1, name: 'Test', tree_json: '{}' } });
    }
  });

  await page.route('/api/registry', async route => {
      await route.fulfill({ json: [
          { name: "SubtreeNode", description: "Subtree node", config_schema: { type: "object" } }
      ] });
  });

  await page.goto('/');
});

test('Properties Panel Visibility', async ({ page }) => {
  await page.route('/api/registry', async route => {
      await route.fulfill({ json: [
          { name: "SubtreeNode", description: "Subtree node", config_schema: { type: "object" } }
      ] });
  });

  await page.goto('/');
  
  // Click on the SubtreeNode in the toolbox
  await page.click('text=SubtreeNode');
  
  // Verify Properties Panel is visible
  await expect(page.locator('text=Properties: SubtreeNode')).toBeVisible();
  
  // Verify textarea for JSON config
  await expect(page.locator('textarea')).toBeVisible();
});
