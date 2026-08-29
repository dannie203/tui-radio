#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const localScript = join(homedir(), '.local', 'bin', 'hiphop-radio-toggle');
const repoScript = join(__dirname, '..', 'omarchy-plugin', 'hiphop-radio-toggle');

const targetScript = existsSync(localScript) ? localScript : repoScript;

const child = spawn(targetScript, process.argv.slice(2), {
  stdio: 'inherit'
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
