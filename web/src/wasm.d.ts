declare module '/wasm/karpador_sim.js' {
  const module: {
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
  };
  export = module;
}
