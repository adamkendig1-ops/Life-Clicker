import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import net from 'node:net';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { root } from './validate.mjs';
const executable = path.resolve(process.argv[2] || 'target/release/LifeClicker.exe');
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'LifeClicker-devtest-'));
for (const p of ['game', 'launcher-ui']) fs.cpSync(path.join(root, p), path.join(dir, p), { recursive: true });
const delay = ms => new Promise(r => setTimeout(r, ms));
const request = (route, body) => fetch(`http://127.0.0.1:8766${route}`, { method: body === undefined ? 'GET' : 'POST', headers: { 'Content-Type': 'application/json' }, body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(3000) });
function launch() {
  const child = spawn(executable, ['--dev', '--root', dir, '--no-browser'], { windowsHide: true });
  child.output = ''; child.stdout.on('data', b => { child.output += b; }); child.stderr.on('data', b => { child.output += b; }); return child;
}
async function exit(child) { if (child.exitCode !== null) return child.exitCode; return (await once(child, 'exit'))[0]; }
const unrelated = net.createServer(socket => socket.once('data', () => socket.end('HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}')));
await new Promise((resolve, reject) => { unrelated.once('error', reject); unrelated.listen(8766, '127.0.0.1', resolve); });
try { const blocked = launch(); assert.equal(await exit(blocked), 1); assert.match(blocked.output, /Another application occupies/); }
finally { await new Promise(r => unrelated.close(r)); }
const child = launch();
try {
  let ready = false;
  for (let i = 0; i < 100; i++) { try { if ((await request('/api/version')).ok) { ready = true; break; } } catch {} await delay(100); }
  assert.ok(ready, child.output);
  const version = await (await request('/api/version')).json(); assert.equal(version.devMode, true); assert.equal(version.saveVersion, 12);
  const duplicate = launch(); assert.equal(await exit(duplicate), 0); assert.match(duplicate.output, /Existing Life Clicker/);
  assert.equal((await request('/api/backup-saves', { data: { LC4_PROFILES: '[]' } })).status, 200);
  assert.equal(fs.existsSync(path.join(dir, 'dev-userdata/backups')), true);
  assert.equal(fs.existsSync(path.join(dir, 'userdata')), false);
  for (const endpoint of ['/api/apply-update', '/api/install-online-update']) assert.equal((await request(endpoint, {})).status, 400);
  await request('/api/shutdown', {}); assert.equal(await exit(child), 0);
  console.log(`PASS actual dev 8766 startup, occupied unrelated/recognized port, isolated fixture backup, disabled updates and shutdown. Fixture: ${dir}`);
} finally { if (child.exitCode === null) child.kill(); }
