import { loadRuntimeApi, runSimulation, type RuntimeResult } from './runtime';

interface RunSimulationMessage {
  type: 'run';
  payload: {
    config: string;
    seed: number;
    maxActions: number;
    maxDays: number;
    sessionsPerDay: number;
    targetLeague: number;
  };
}

interface WorkerSuccessMessage {
  type: 'success';
  result: RuntimeResult;
}

interface WorkerFailureMessage {
  type: 'failure';
  error: string;
}

type WorkerMessage = WorkerSuccessMessage | WorkerFailureMessage;

self.onmessage = async (event: MessageEvent<RunSimulationMessage>) => {
  const { data } = event;
  if (data.type !== 'run') {
    return;
  }

  try {
    const { config, seed, maxActions, maxDays, sessionsPerDay, targetLeague } = data.payload;
    const runtimeApi = await loadRuntimeApi();
    const result = await runSimulation(
      runtimeApi,
      config,
      {
        seed,
        maxActions,
        maxDays,
        sessionsPerDay,
        targetLeague,
      },
    );
    const message: WorkerMessage = {
      type: 'success',
      result,
    };
    self.postMessage(message);
  } catch (error) {
    const message: WorkerMessage = {
      type: 'failure',
      error: error instanceof Error ? error.message : String(error),
    };
    self.postMessage(message);
  }
};
