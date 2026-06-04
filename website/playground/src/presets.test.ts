import { describe, it, expect } from 'vitest';
import { PRESETS, PRESET_ORDER } from './presets';

describe('presets', () => {
  it('PRESET_ORDER entries all exist in PRESETS', () => {
    for (const id of PRESET_ORDER) {
      expect(PRESETS[id]).toBeDefined();
    }
  });

  it('every PRESETS key is in PRESET_ORDER', () => {
    const orderSet = new Set<string>(PRESET_ORDER);
    for (const key of Object.keys(PRESETS)) {
      expect(orderSet.has(key)).toBe(true);
    }
  });

  it('each preset has at least one edge', () => {
    for (const [, preset] of Object.entries(PRESETS)) {
      expect(preset.edges.length).toBeGreaterThan(0);
    }
  });

  it('each preset id matches its key', () => {
    for (const [key, preset] of Object.entries(PRESETS)) {
      expect(preset.id).toBe(key);
    }
  });

  it('edge vertex indices are non-negative integers', () => {
    for (const [, preset] of Object.entries(PRESETS)) {
      for (const [u, v] of preset.edges) {
        expect(Number.isInteger(u)).toBe(true);
        expect(Number.isInteger(v)).toBe(true);
        expect(u).toBeGreaterThanOrEqual(0);
        expect(v).toBeGreaterThanOrEqual(0);
      }
    }
  });
});
