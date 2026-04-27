const { test, expect } = require('@playwright/test');

test('vite app starts and sim can run in browser', async ({ page }) => {
  await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

  const header = page.getByRole('heading', { name: /Simulator-Startkonfiguration|Simulator/i });
  await expect(header).toBeVisible({ timeout: 30000 });

  const runButton = page.getByRole('button', { name: /In Browser ausf\.?hren|Run in Browser|Runtime/i }).first();
  await expect(runButton).toBeEnabled({ timeout: 20000 });
  await runButton.click();

  const output = page.locator('.runtime-output textarea.runtime-json');
  await expect(output).toBeVisible({ timeout: 30000 });

  await expect(output).not.toHaveValue('', { timeout: 120000 });
  const raw = await output.inputValue();
  expect(raw).toContain('"wall_days"');
  const parsed = JSON.parse(raw);
  expect(parsed.outcome).toBe('TargetReached');
  expect(typeof parsed.wall_days).toBe('number');
  expect(parsed.wall_days).toBeGreaterThan(20);
  expect(parsed.wall_days).toBeLessThan(40);

  const summary = page.locator('.runtime-output pre');
  if (await summary.count()) {
    const summaryText = await summary.first().innerText();
    if (summaryText.includes('wall_days')) {
      expect(summaryText.length).toBeGreaterThan(10);
    }
  }

  console.log('PLAYWRIGHT_OK wall_days=', parsed.wall_days, 'sessions=', parsed.sessions);
});
