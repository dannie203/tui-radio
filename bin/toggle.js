#!/usr/bin/env node
import { isProcessAlive, readPidFile } from '../src/desktop/instance_manager.js';
import { spawn } from 'node:child_process';

const PID_FILE = '/tmp/hiphop-tui.pid';

const pid = readPidFile(PID_FILE);
if (pid && isProcessAlive(pid)) {
  try {
    process.kill(pid, 'SIGUSR1');
  } catch {}
  try {
    spawn('hyprctl', ['dispatch', 'focuswindow', 'title:NEON//WAVE CYBERPUNK AUDIO TERMINAL'], { stdio: 'ignore' });
  } catch {}
  process.exit(0);
}

try {
  spawn(process.execPath, ['bin/index.js'], { stdio: 'inherit', detached: true });
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
