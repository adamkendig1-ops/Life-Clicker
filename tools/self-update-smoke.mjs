// Test the real helper in an isolated installation. Never opens a browser.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import net from 'node:net';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
const source = path.resolve(process.argv[2] || 'target/release/LifeClicker.exe');
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'LifeClicker-selftest-'));
const helper = path.join(os.tmpdir(), `LifeClicker-helper-selftest-${crypto.randomUUID()}.exe`);
const current = path.join(dir, 'LifeClicker.exe');
const original = fs.readFileSync(source);
const updated = Buffer.concat([original, Buffer.from('\nself-update test overlay\n')]);
fs.writeFileSync(current, original); fs.writeFileSync(helper, original);
fs.writeFileSync(path.join(dir, 'LifeClicker.next.exe'), updated);
fs.writeFileSync(path.join(dir, 'LifeClicker.next.exe.sha256'), crypto.createHash('sha256').update(updated).digest('hex'));
// Hold a fixture listener when free, so restarted candidate never serves real origin.
// If an existing launcher occupies it, restarted candidate only performs GET /api/version.
const blocker = net.createServer(socket => socket.once('data', () => socket.end('HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}')));
const holding = await new Promise((resolve, reject) => {
  blocker.once('error', e => e.code === 'EADDRINUSE' ? resolve(false) : reject(e));
  blocker.listen(8765, '127.0.0.1', () => resolve(true));
});
try {
  const child = spawn(helper, ['--apply-self-update', dir, current, '--no-browser'], { windowsHide: true });
  let log = ''; child.stdout.on('data', b => { log += b; }); child.stderr.on('data', b => { log += b; });
  const timer = setTimeout(() => child.kill(), 15000);
  const [code] = await once(child, 'exit'); clearTimeout(timer);
  assert.equal(code, 0, log);
  for (let i = 0; i < 100 && fs.existsSync(helper); i++) await new Promise(r => setTimeout(r, 100));
  assert.deepEqual(fs.readFileSync(current), updated);
  assert.deepEqual(fs.readFileSync(path.join(dir, 'LifeClicker.previous.exe')), original);
  assert.equal(fs.existsSync(path.join(dir, 'LifeClicker.next.exe')), false);
  // Only the restarted executable performs helper cleanup, so this verifies restart too.
  assert.equal(fs.existsSync(helper), false, 'Restarted executable failed to remove helper');
  console.log(`PASS real helper replacement, retained previous executable, restart, helper cleanup; no browser. Fixture: ${dir}`);
} finally { if (holding) await new Promise(r => blocker.close(r)); }
