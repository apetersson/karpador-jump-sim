const { chromium } = require('@playwright/test');

const runOnce = async () => {
  const URL = 'https://apetersson.github.io/karpador-jump-sim/';
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  const failed = [];
  const errors = [];

  page.on('response', (resp) => {
    const url = resp.url();
    if (!resp.ok() && (url.includes('master_data') || url.includes('karpador_sim'))) {
      failed.push({ url, status: resp.status() });
    }
  });
  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      errors.push(msg.text());
    }
  });

  try {
    const res = await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 120000 });
    if (!res || !res.ok()) throw new Error(`HTTP ${res ? res.status() : 'no-response'}`);

    await page.getByRole('heading', { name: /Simulator-Startkonfiguration|Simulator/i }).waitFor({ timeout: 30000 });
    const runButton = page.getByRole('button', { name: /In Browser ausf.*hren|Run in Browser/i }).first();
    await runButton.waitFor({ timeout: 30000 });

    for (let n = 0; n < 120 && await runButton.isDisabled(); n++) {
      await page.waitForTimeout(250);
    }

    if (await runButton.isDisabled()) {
      throw new Error('Run button remained disabled');
    }

    await runButton.click();

    const output = page.locator('.runtime-output textarea.runtime-json');
    await output.waitFor({ timeout: 120000 });
    await page.waitForFunction(() => {
      const el = document.querySelector('.runtime-output textarea.runtime-json');
      return !!(el && el.value && el.value.includes('"wall_days"'));
    }, undefined, { timeout: 120000 });

    const raw = await output.inputValue();
    JSON.parse(raw);

    await browser.close();

    if (failed.length) {
      console.log('FAILED_FETCHES', JSON.stringify(failed));
      process.exit(2);
    }
    console.log('OK', { errorCount: errors.length });
    process.exit(0);
  } catch (err) {
    await browser.close();
    console.log('FAILED', String(err));
    if (failed.length) {
      console.log('FAILED_FETCHES', JSON.stringify(failed));
    }
    if (errors.length) {
      console.log('CONSOLE_ERRORS', JSON.stringify(errors.slice(0, 20)));
    }
    process.exit(1);
  }
};

runOnce();
