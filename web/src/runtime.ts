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
  const compactSummary = pickSummaryObject(value);
  if (compactSummary != null) {
    return compactSummary;
  }

  const summary = value.summary;
  if (summary != null && typeof summary === 'object') {
    return pickSummaryObject(summary as Record<string, unknown>);
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

  return {
    plan,
    outcome,
    wall_days: wallDays,
    sessions,
    warnings: pickStringArray(value['warnings']),
    league,
    diamonds,
  };
};

const pickSummaryObject = (summary: Record<string, unknown>): RuntimeSummary | null => {
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

  return {
    plan,
    outcome,
    wall_days: wallDays,
    sessions,
    warnings: pickStringArray(summary['warnings']),
    league,
    diamonds,
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
