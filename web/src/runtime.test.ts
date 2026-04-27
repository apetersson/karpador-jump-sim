import { describe, expect, it } from 'vitest';
import {
  parseRuntimeSummary,
  runSimulation,
  type RuntimeApi,
} from './runtime';

describe('parseRuntimeSummary', () => {
  it('parses compact summary payload', () => {
    const summaryPayload = JSON.stringify({
      plan: 'balanced',
      outcome: 'Goal reached',
      wall_days: 12.5,
      sessions: 3_200,
      warnings: ['ok'],
      league: 6,
      diamonds: 14,
    });
    expect(parseRuntimeSummary(summaryPayload)).toEqual({
      plan: 'balanced',
      outcome: 'Goal reached',
      wall_days: 12.5,
      sessions: 3_200,
      warnings: ['ok'],
      league: 6,
      diamonds: 14,
    });
  });

  it('derives summary from full runtime result', () => {
    const fullPayload = JSON.stringify({
      plan: 'balanced',
      outcome: 'In progress',
      wall_days: 3,
      sessions: 40,
      warnings: [],
      final_state: {
        league: 4,
        diamonds: 7,
      },
    });
    expect(parseRuntimeSummary(fullPayload)).toEqual({
      plan: 'balanced',
      outcome: 'In progress',
      wall_days: 3,
      sessions: 40,
      warnings: [],
      league: 4,
      diamonds: 7,
    });
  });
});

describe('runSimulation', () => {
  it('prefers runtime summary API when available', async () => {
    const api: RuntimeApi = {
      runWallTimeSimulation: () => JSON.stringify({ plan: 'legacy' }),
      runWallTimeSimulationSummary: () =>
        JSON.stringify({
          plan: 'custom',
          outcome: 'ok',
          wall_days: 4,
          sessions: 99,
          warnings: [],
          league: 1,
          diamonds: 8,
        }),
    };
    const result = await runSimulation(api, '{}', { targetLeague: 1 });
    expect(result.payload).toBe(JSON.stringify({ plan: 'legacy' }));
    expect(result.summary).toEqual({
      plan: 'custom',
      outcome: 'ok',
      wall_days: 4,
      sessions: 99,
      warnings: [],
      league: 1,
      diamonds: 8,
    });
  });
});
