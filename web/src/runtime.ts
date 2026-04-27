export interface RuntimeApi {
  runWallTimeSimulation: (
    startConfigJson: string,
    seed: bigint,
    maxActions: u32,
    maxDays: u32,
    sessionsPerDay: u8,
    targetLeague: u32,
  ) => string;
  runWallTimeSimulationSummary?: (
    startConfigJson: string,
    seed: bigint,
    maxActions: number,
    maxDays: number,
    sessionsPerDay: number,
    targetLeague: number,
  ) => string;
}

export interface RuntimeSummary {
  plan: string;
  outcome: string;
  wall_days: number;
  sessions: number;
  warnings: string[];
  league: number;
  diamonds: number;
  days_to_master_league: number | null;
  diamonds_spent_total: number;
  diamond_spending_by_kind: {
    kind: string;
    amount: number;
  }[];
  diamond_spending_by_item: {
    kind: string;
    id: string;
    amount: number;
  }[];
}

export interface RuntimeResult {
  payload: string;
  summary: RuntimeSummary | null;
}

const isTypeErrorCannotConvert = (error: unknown): boolean => {
  return error instanceof TypeError && String(error.message).includes('Cannot convert');
};

type WallTimeCall = (
  startConfigJson: string,
  seed: bigint | number,
  maxActions: number,
  maxDays: number,
  sessionsPerDay: number,
  targetLeague: number,
) => string;

const callWithSeedFallback = (
  fn: WallTimeCall,
  options: {
    startConfigJson: string;
    seed: number;
    maxActions: number;
    maxDays: number;
    sessionsPerDay: number;
    targetLeague: number;
  },
): string => {
  try {
    return fn(
      options.startConfigJson,
      BigInt(options.seed),
      options.maxActions,
      options.maxDays,
      options.sessionsPerDay,
      options.targetLeague,
    );
  } catch (error) {
    if (!isTypeErrorCannotConvert(error)) {
      throw error;
    }
    return fn(
      options.startConfigJson,
      options.seed,
      options.maxActions,
      options.maxDays,
      options.sessionsPerDay,
      options.targetLeague,
    );
  }
};

const pickString = (value: unknown): string | null => {
  return typeof value === 'string' ? value : null;
};

const pickNumber = (value: unknown): number | null => {
  return typeof value === 'number' ? value : null;
};

const parseDiamondSpendingByKind = (
  value: unknown,
): { kind: string; amount: number }[] => {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.flatMap((entry) => {
    if (entry == null || typeof entry !== 'object') {
      return [];
    }
    const raw = entry as Record<string, unknown>;
    const kind = pickString(raw.kind);
    const amount = pickNumber(raw.amount);
    if (kind == null || amount == null) {
      return [];
    }
    return [{ kind, amount }];
  });
};

const parseDiamondSpendingByItem = (
  value: unknown,
): { kind: string; id: string; amount: number }[] => {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.flatMap((entry) => {
    if (entry == null || typeof entry !== 'object') {
      return [];
    }
    const raw = entry as Record<string, unknown>;
    const kind = pickString(raw.kind);
    const id = pickString(raw.id);
    const amount = pickNumber(raw.amount);
    if (kind == null || id == null || amount == null) {
      return [];
    }
    return [{ kind, id, amount }];
  });
};

const deriveDiamondSpendingFromPurchases = (source: Record<string, unknown>): {
  diamonds_spent_total: number;
  diamond_spending_by_kind: { kind: string; amount: number }[];
  diamond_spending_by_item: { kind: string; id: string; amount: number }[];
} => {
  const purchases = Array.isArray(source['purchases']) ? source['purchases'] : [];
  const byKind = new Map<string, number>();
  const byItem = new Map<string, number>();
  let total = 0;

  for (const purchase of purchases) {
    if (purchase == null || typeof purchase !== 'object') {
      continue;
    }
    const entry = purchase as Record<string, unknown>;
    const kind = pickString(entry['kind']);
    const id = pickString(entry['id']);
    const amount = pickNumber(entry['price_diamonds']);
    if (kind == null || id == null || amount == null) {
      continue;
    }
    total += amount;
    byKind.set(kind, (byKind.get(kind) ?? 0) + amount);
    byItem.set(`${kind}::${id}`, (byItem.get(`${kind}::${id}`) ?? 0) + amount);
  }

  return {
    diamonds_spent_total: total,
    diamond_spending_by_kind: [...byKind.entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([kind, amount]) => ({ kind, amount })),
    diamond_spending_by_item: [...byItem.entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([key, amount]) => {
        const [kind, id] = key.split('::', 2);
        return { kind: kind ?? '', id: id ?? '', amount };
      }),
  };
};

const deriveDaysToMasterLeague = (source: Record<string, unknown>): number | null => {
  const finalState = source['final_state'];
  if (finalState == null || typeof finalState !== 'object') {
    return null;
  }
  const finalStateRecord = finalState as Record<string, unknown>;
  const finalLeague = pickNumber(finalStateRecord['league']);
  const wallDays = pickNumber(source['wall_days']);
  if (finalLeague != null && finalLeague >= 10 && wallDays != null) {
    return wallDays;
  }

  if (!Array.isArray(source['action_log'])) {
    return null;
  }

  for (const entry of source['action_log']) {
    if (entry == null || typeof entry !== 'object') {
      continue;
    }
    const log = entry as Record<string, unknown>;
    if (pickString(log['event']) !== 'league_fight') {
      continue;
    }
    const detail = pickString(log['detail']);
    if (detail == null) {
      continue;
    }
    const match = detail.match(/->\s*(\d+)-\d+/);
    if (match == null) {
      continue;
    }
    const toLeague = Number.parseInt(match[1], 10);
    if (Number.isNaN(toLeague) || toLeague < 11) {
      continue;
    }
    const minute = pickNumber(log['minute']);
    if (minute == null) {
      continue;
    }
    return minute / 1440;
  }

  return null;
};

const pickStringArray = (value: unknown): string[] => {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((entry) => (typeof entry === 'string' ? entry : null))
    .filter((entry): entry is string => entry !== null);
};

const parseRuntimeSummaryFromValue = (
  value: Record<string, unknown>,
): RuntimeSummary | null => {
  const compactSummary = pickSummaryObject(value, value);
  if (compactSummary != null) {
    return compactSummary;
  }

  const summary = value.summary;
  if (summary != null && typeof summary === 'object') {
    return pickSummaryObject(summary as Record<string, unknown>, value);
  }

  if (typeof value.final_state !== 'object' || value.final_state === null) {
    return null;
  }
  const finalState = value.final_state as Record<string, unknown>;
  const outcome = pickString(value['outcome']);
  const plan = pickString(value['plan']);
  const wallDays = pickNumber(value['wall_days']);
  const sessions = pickNumber(value['sessions']);
  const league = pickNumber(finalState['league']);
  const diamonds = pickNumber(finalState['diamonds']);
  if (
    outcome == null ||
    plan == null ||
    wallDays == null ||
    sessions == null ||
    league == null ||
    diamonds == null
  ) {
    return null;
  }

  const diamondSpending = mergeDiamondSpending(value);
  return {
    plan,
    outcome,
    wall_days: wallDays,
    sessions,
    warnings: pickStringArray(value['warnings']),
    league,
    diamonds,
    days_to_master_league: deriveDaysToMasterLeague(value),
    ...diamondSpending,
  };
};

const mergeDiamondSpending = (
  source: Record<string, unknown>,
): {
  diamonds_spent_total: number;
  diamond_spending_by_kind: { kind: string; amount: number }[];
  diamond_spending_by_item: { kind: string; id: string; amount: number }[];
} => {
  const fallback = deriveDiamondSpendingFromPurchases(source);
  const byKind = parseDiamondSpendingByKind(source['diamond_spending_by_kind']);
  const byItem = parseDiamondSpendingByItem(source['diamond_spending_by_item']);
  const diamondsSpentTotal = pickNumber(source['diamonds_spent_total']);
  return {
    diamonds_spent_total:
      diamondsSpentTotal ??
      (fallback.diamonds_spent_total > 0 ? fallback.diamonds_spent_total : 0),
    diamond_spending_by_kind:
      byKind.length > 0 ? byKind : fallback.diamond_spending_by_kind,
    diamond_spending_by_item:
      byItem.length > 0 ? byItem : fallback.diamond_spending_by_item,
  };
};

const pickSummaryObject = (
  summary: Record<string, unknown>,
  fallbackSource: Record<string, unknown>,
): RuntimeSummary | null => {
  const outcome = pickString(summary['outcome']);
  const plan = pickString(summary['plan']);
  const wallDays = pickNumber(summary['wall_days']);
  const sessions = pickNumber(summary['sessions']);
  const league = pickNumber(summary['league']);
  const diamonds = pickNumber(summary['diamonds']);
  if (
    outcome == null ||
    plan == null ||
    wallDays == null ||
    sessions == null ||
    league == null ||
    diamonds == null
  ) {
    return null;
  }

  const diamondSpending = mergeDiamondSpending({
    ...fallbackSource,
    ...summary,
  });

  return {
    plan,
    outcome,
    wall_days: wallDays,
    sessions,
    warnings: pickStringArray(summary['warnings']),
    league,
    diamonds,
    days_to_master_league:
      pickNumber(summary['days_to_master_league']) ?? deriveDaysToMasterLeague(fallbackSource),
    ...diamondSpending,
  };
};

export const parseRuntimeSummary = (raw: string): RuntimeSummary | null => {
  try {
    const parsed = JSON.parse(raw);
    if (parsed == null || typeof parsed !== 'object') {
      return null;
    }
    return parseRuntimeSummaryFromValue(parsed as Record<string, unknown>);
  } catch {
    return null;
  }
};

export const runSimulation = async (
  runtimeApi: RuntimeApi,
  config: string,
  options?: {
    seed?: number;
    maxActions?: number;
    maxDays?: number;
    sessionsPerDay?: number;
    targetLeague?: number;
  },
): Promise<RuntimeResult> => {
  const seed = options?.seed ?? 42;
  const payload = callWithSeedFallback(runtimeApi.runWallTimeSimulation, {
    startConfigJson: config,
    seed,
    maxActions: options?.maxActions ?? 100_000,
    maxDays: options?.maxDays ?? 240,
    sessionsPerDay: options?.sessionsPerDay ?? 10,
    targetLeague: options?.targetLeague ?? 10,
  });
  return {
    payload,
    summary: parseRuntimeSummary(
      runtimeApi.runWallTimeSimulationSummary
        ? callWithSeedFallback(runtimeApi.runWallTimeSimulationSummary, {
            startConfigJson: config,
            seed,
            maxActions: options?.maxActions ?? 100_000,
            maxDays: options?.maxDays ?? 240,
            sessionsPerDay: options?.sessionsPerDay ?? 10,
            targetLeague: options?.targetLeague ?? 10,
          })
        : payload,
    ),
  };
};

const importRuntimeModule = (path: string): Promise<unknown> => {
  return Function('path', 'return import(path)')('' + path);
};

export interface WebAssemblyRuntimeApi {
  default: () => Promise<void>;
  run_wall_time_simulation: (
    startConfigJson: string,
    seed: bigint,
    maxActions: number,
    maxDays: number,
    sessionsPerDay: number,
    targetLeague: number,
  ) => string;
  run_wall_time_simulation_summary?: (
    startConfigJson: string,
    seed: bigint,
    maxActions: number,
    maxDays: number,
    sessionsPerDay: number,
    targetLeague: number,
  ) => string;
}

export const loadRuntimeApi = async (): Promise<RuntimeApi> => {
  const module = (await importRuntimeModule('/wasm/karpador_sim.js')) as unknown as WebAssemblyRuntimeApi;
  if (typeof module.default === 'function') {
    await module.default();
  }
  return {
    runWallTimeSimulation: module.run_wall_time_simulation,
    runWallTimeSimulationSummary: module.run_wall_time_simulation_summary,
  };
};
