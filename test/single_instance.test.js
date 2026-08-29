import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { writePidFile, readPidFile, clearPidFile, isProcessAlive } from '../src/desktop/instance_manager.js';

describe('Single-instance pid helpers', () => {
  it('stores and clears a pid file for the current process', () => {
    const dir = mkdtempSync(join(tmpdir(), 'hiphop-radio-pid-'));
    const pidFile = join(dir, 'hiphop-radio.pid');

    writePidFile(pidFile, process.pid);
    assert.equal(readPidFile(pidFile), process.pid);
    assert.equal(isProcessAlive(process.pid), true);

    clearPidFile(pidFile);
    assert.equal(readPidFile(pidFile), null);

    rmSync(dir, { recursive: true, force: true });
  });
});
