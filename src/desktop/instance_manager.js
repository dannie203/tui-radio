import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';

export function writePidFile(pidFile, pid = process.pid) {
  try {
    writeFileSync(pidFile, String(pid), 'utf8');
    return true;
  } catch {
    return false;
  }
}

export function readPidFile(pidFile) {
  try {
    if (!existsSync(pidFile)) return null;
    const value = Number(readFileSync(pidFile, 'utf8').trim());
    return Number.isFinite(value) ? value : null;
  } catch {
    return null;
  }
}

export function clearPidFile(pidFile) {
  try {
    rmSync(pidFile, { force: true });
    return true;
  } catch {
    return false;
  }
}

export function isProcessAlive(pid) {
  if (!Number.isFinite(pid) || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}
