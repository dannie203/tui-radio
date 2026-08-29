import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { THEMES, THEME_KEYS, getTheme, cycleTheme } from '../src/ui/theme.js';

describe('Linux Theme Engine', () => {
  test('contains all 7 popular Linux palettes with valid hex colors', () => {
    assert.equal(THEME_KEYS.length, 7);
    assert.ok(THEME_KEYS.includes('CATPPUCCIN_MOCHA'));
    assert.ok(THEME_KEYS.includes('TOKYO_NIGHT'));
    assert.ok(THEME_KEYS.includes('GRUVBOX_RETRO'));
    assert.ok(THEME_KEYS.includes('NORD_FROST'));
    assert.ok(THEME_KEYS.includes('DRACULA'));
    assert.ok(THEME_KEYS.includes('MATRIX_GREEN'));
    assert.ok(THEME_KEYS.includes('AMBER_GOLD'));

    for (const key of THEME_KEYS) {
      const theme = THEMES[key];
      assert.ok(theme.name);
      assert.ok(theme.bgDark.startsWith('#'));
      assert.ok(theme.amber.startsWith('#'));
      assert.ok(theme.cyanDolby.startsWith('#'));
    }
  });

  test('cycles themes forward and backward cleanly with wrap-around', () => {
    const start = 'AMBER_GOLD';
    const next = cycleTheme(start, 1);
    assert.notEqual(next, start);
    const prev = cycleTheme(next, -1);
    assert.equal(prev, start);
  });

  test('falls back to default Amber Gold for unknown theme IDs', () => {
    const fallback = getTheme('UNKNOWN_THEME_XYZ');
    assert.equal(fallback.id, 'AMBER_GOLD');
  });
});
