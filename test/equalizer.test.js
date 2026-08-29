import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { EQ_PRESETS, EQ_PRESET_KEYS, getEqPreset, cycleEqPreset } from '../src/audio/equalizer.js';

describe('32-Band Equalizer Presets Engine', () => {
  test('contains all sound profiles with exactly 32 frequency gains', () => {
    assert.ok(EQ_PRESET_KEYS.includes('FLAT'));
    assert.ok(EQ_PRESET_KEYS.includes('MEGA_BASS'));
    assert.ok(EQ_PRESET_KEYS.includes('VOCAL_CLEAR'));
    assert.ok(EQ_PRESET_KEYS.includes('ROCK_PUNCH'));
    assert.ok(EQ_PRESET_KEYS.includes('LOFI_WARMTH'));
    assert.ok(EQ_PRESET_KEYS.includes('CYBER_SYNTH'));
    assert.ok(EQ_PRESET_KEYS.includes('CLUB_EDM'));

    for (const key of EQ_PRESET_KEYS) {
      const preset = EQ_PRESETS[key];
      assert.ok(preset.name);
      assert.equal(preset.gains.length, 32);
      assert.ok(preset.gains.every((g) => typeof g === 'number' && !isNaN(g)));
    }
  });

  test('cycles presets forward and backward seamlessly', () => {
    const start = 'FLAT';
    const next = cycleEqPreset(start, 1);
    assert.equal(next, 'MEGA_BASS');
    const prev = cycleEqPreset(next, -1);
    assert.equal(prev, start);
  });

  test('falls back to FLAT for invalid preset names', () => {
    const res = getEqPreset('NON_EXISTENT');
    assert.equal(res.id, 'FLAT');
  });
});
