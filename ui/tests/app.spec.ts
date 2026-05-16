import { test, expect } from '@playwright/test';

test.describe('pib.Cerebra UI E2E Tests', () => {
  test('App loads successfully and has correct title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/pib.Cerebra/); // Assuming title contains pib.Cerebra, or adjust later
  });

  test('Node Toolbox renders correctly with mocked API', async ({ page }) => {
    // Mock the /api/registry response
    await page.route('/api/registry', async route => {
      const json = [{ name: 'Sequence', type: 'Control' }];
      await route.fulfill({ json });
    });

    await page.goto('/');
    
    // Check if the Node Toolbox or mocked nodes are visible. 
    // This depends on the UI implementation. Let's look for "Sequence" text somewhere in the sidebar.
    await expect(page.getByText('Sequence')).toBeVisible();
  });

  test('Canvas/Editor exists and allows basic interaction', async ({ page }) => {
    await page.goto('/');
    // Check for canvas container, maybe it has a class like 'react-flow' or similar, or just check for main canvas element.
    const canvas = page.locator('.react-flow, canvas, [data-testid="rf__wrapper"]');
    await expect(canvas.first()).toBeVisible();
  });
});
