import { describe, it, beforeEach, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { mergeConfig, DEFAULT_CONFIG, SETTINGS_SECTIONS } from '../src/state/config.js';
import { Store } from '../src/state/store.js';

describe('Settings & Configuration System', () => {
  it('mergeConfig merges defaults with user overrides cleanly', () => {
    const userConfig = {
      visualizer: {
        colorTheme: 'AMBER_GOLD',
        peakHoldMs: 2000
      },
      dsp: {
        stereoMode: '3D WIDE',
        bassBoost: true
      }
    };

    const merged = mergeConfig(DEFAULT_CONFIG, userConfig);

    assert.equal(merged.visualizer.colorTheme, 'AMBER_GOLD');
    assert.equal(merged.visualizer.peakHoldMs, 2000);
    assert.equal(merged.visualizer.peakDecayRate, 38); // Preserved from default
    assert.equal(merged.dsp.stereoMode, '3D WIDE');
    assert.equal(merged.dsp.bassBoost, true);
    assert.equal(merged.dsp.dolbyMode, 'DOLBY-B'); // Preserved from default
  });

  it('Store initializes with settings schema and toggle/navigate works', () => {
    const store = new Store();

    assert.equal(store.state.settingsVisible, false);
    assert.equal(store.state.settingsSelectedIndex, 0);
    assert.equal(store.state.settingsSections.length, SETTINGS_SECTIONS.length);

    store.toggleSettings();
    assert.equal(store.state.settingsVisible, true);

    store.moveSettingsSelection(1);
    assert.equal(store.state.settingsSelectedIndex, 1);

    store.moveSettingsSelection(-1);
    assert.equal(store.state.settingsSelectedIndex, 0);

    store.toggleSettings(false);
    assert.equal(store.state.settingsVisible, false);
  });

  it('cycleSettingValue updates config value and mirrors state for DSP', () => {
    const store = new Store();

    // Select Visualizer Color Theme
    const themeIdx = store.state.settingsSections.findIndex((s) => s.id === 'visualizer.colorTheme');
    store.state.settingsSelectedIndex = themeIdx;
    const initialTheme = store.config.visualizer.colorTheme;
    assert.equal(initialTheme, 'SYSTEM_AUTO');

    const res = store.cycleSettingValue(1);
    assert.equal(res.value, 'RGB_CHROMA');
    assert.equal(store.config.visualizer.colorTheme, 'RGB_CHROMA');

    // Find and select Stereo Mode
    const dspIdx = store.state.settingsSections.findIndex((s) => s.id === 'dsp.stereoMode');
    store.state.settingsSelectedIndex = dspIdx;
    assert.equal(store.state.stereoMode, 'STEREO');

    const dspRes = store.cycleSettingValue(1);
    assert.equal(dspRes.value, 'MONO');
    assert.equal(store.state.stereoMode, 'MONO');
    assert.equal(store.config.dsp.stereoMode, 'MONO');
  });

  it('cycleStereoMode cleanly cycles across STEREO, MONO, 3D WIDE with alias support', () => {
    const store = new Store();
    assert.equal(store.state.stereoMode, 'STEREO');

    store.cycleStereoMode(1);
    assert.equal(store.state.stereoMode, 'MONO');

    store.cycleStereoMode(1);
    assert.equal(store.state.stereoMode, '3D WIDE');

    store.cycleStereoMode(1);
    assert.equal(store.state.stereoMode, 'STEREO');

    // Test alias handling
    store.state.stereoMode = 'STEREO-3D';
    store.cycleStereoMode(1);
    assert.equal(store.state.stereoMode, 'STEREO');
  });
});

