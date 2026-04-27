import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('Wasm build artifacts', () => {
  it('includes compiled module and binary in public/wasm', () => {
    const artifactDir = join(process.cwd(), 'public', 'wasm');
    const wasmJs = join(artifactDir, 'karpador_sim.js');
    const wasmWasm = join(artifactDir, 'karpador_sim_bg.wasm');
    expect(existsSync(artifactDir)).toBe(true);
    expect(existsSync(wasmJs)).toBe(true);
    expect(existsSync(wasmWasm)).toBe(true);
  });
});
