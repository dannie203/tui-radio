import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { TrayManager } from '../src/desktop/tray_manager.js';
import { Store } from '../src/state/store.js';

describe('Desktop Tray & MPRIS2 Integration', () => {
  test('TrayManager initializes and syncs state without throwing', async () => {
    const store = new Store();
    let actionCalled = null;

    const mockActions = {
      togglePause: () => { actionCalled = 'togglePause'; },
      next: () => { actionCalled = 'next'; },
      prev: () => { actionCalled = 'prev'; },
      stop: () => { actionCalled = 'stop'; },
      cycleStereoMode: (d) => { actionCalled = `cycleStereoMode:${d}`; },
      toggleBassBoost: () => { actionCalled = 'toggleBassBoost'; }
    };

    const tray = new TrayManager(store, mockActions, {});
    tray.start();

    // Verify action dispatch mapping
    tray.handleAction('play_pause');
    assert.equal(actionCalled, 'togglePause');

    tray.handleAction('next');
    assert.equal(actionCalled, 'next');

    tray.handleAction('prev');
    assert.equal(actionCalled, 'prev');

    tray.handleAction('cycle_stereo');
    assert.equal(actionCalled, 'cycleStereoMode:1');

    tray.handleAction('toggle_bass');
    assert.equal(actionCalled, 'toggleBassBoost');

    tray.destroy();
    assert.equal(tray.active, false);
  });
});
