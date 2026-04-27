import { test, expect } from '@playwright/test';
import { execSync } from 'child_process';
import { writeFileSync } from 'fs';
import { resolve } from 'path';

const START_CONFIG = {
  start_state: {
    player_rank: 25,
    gold: 12345,
    diamonds: 300,
    league: 4,
    competition: 2,
    generation: 18,
    retirements: 17,
    magikarp_level: 31,
    magikarp_kp: 0,
    candy: 4,
    training_sodas: 2,
    skill_herbs: 1,
    league_aids: 0,
    owned_supports: ['charizard', 'pikachu', 'piplup', 'meowth'],
    owned_decors: ['shaymin_planter', 'octillery_pot'],
    berry_levels: {},
    training_levels: {},
  },
  policy: {
    allowed_berry_upgrades: ['food_1', 'food_2', 'food_6'],
    allowed_training_upgrades: ['training_1', 'training_2'],
    purchase_plan: '[]',
    allow_training_sodas: true,
    allow_skill_herbs: true,
    allow_support_upgrades: true,
    training_upgrade_share: 2500,
    karpador_loss_risk_max_level_percent: 60,
    sessions_per_day: 10,
  },
};

function getRustResult() {
  const tmpConfig = '/tmp/karpador_test_start_config.json';
  writeFileSync(tmpConfig, JSON.stringify(START_CONFIG, null, 2));
  const bin = resolve(import.meta.dirname, '..', '..', 'rust', 'target', 'release', 'karpador-sim');
  const output = execSync(
    `${bin} run --start-config ${tmpConfig} --seed 42 --json`,
    { encoding: 'utf-8', timeout: 60_000 },
  );
  return JSON.parse(output);
}

test('browser sim matches rust standalone for default config', async ({ page }) => {
  const rustResult = getRustResult();
  console.log('Rust result:', {
    outcome: rustResult.outcome,
    wall_days: rustResult.wall_days?.toFixed(2),
    sessions: rustResult.sessions,
    league: rustResult.final_state?.league,
  });

  await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

  const header = page.getByRole('heading', { name: /Simulator-Startkonfiguration|Simulator/i });
  await expect(header).toBeVisible({ timeout: 30000 });

  const runButton = page.getByRole('button', { name: /In Browser ausf.*hren|Run in Browser/i }).first();
  await expect(runButton).toBeEnabled({ timeout: 20000 });
  await runButton.click();

  const output = page.locator('.runtime-output textarea.runtime-json');
  await expect(output).toBeVisible({ timeout: 30000 });
  await expect(output).not.toHaveValue('', { timeout: 120000 });
  const raw = await output.inputValue();
  expect(raw).toContain('"wall_days"');
  const browserResult = JSON.parse(raw);

  console.log('Browser result:', {
    outcome: browserResult.outcome,
    wall_days: browserResult.wall_days?.toFixed(2),
    sessions: browserResult.sessions,
    league: browserResult.final_state?.league,
  });

  expect(browserResult.outcome).toBe('TargetReached');
  expect(rustResult.outcome).toBe('TargetReached');
  expect(browserResult.final_state.league).toBe(rustResult.final_state.league);

  const wallDaysDiff = Math.abs(browserResult.wall_days - rustResult.wall_days);
  expect(wallDaysDiff).toBeLessThan(0.01);
  expect(browserResult.sessions).toBe(rustResult.sessions);

  console.log(`OK: browser=${browserResult.wall_days?.toFixed(2)}d rust=${rustResult.wall_days?.toFixed(2)}d diff=${wallDaysDiff.toFixed(4)}`);
});
