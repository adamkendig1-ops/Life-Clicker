// Explicit API exercises use only a copied installation and synthetic snapshots.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import net from 'node:net';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
const fixture = fs.mkdtempSync(path.join(os.tmpdir(), 'LifeClicker-api-contract-'));
fs.cpSync(path.resolve(process.argv[2]), fixture, { recursive: true });
const probe = net.createServer();
await new Promise((resolve, reject) => { probe.once('error', reject); probe.listen(8765, '127.0.0.1', resolve); });
await new Promise(r => probe.close(r));
const child = spawn(path.join(fixture, 'LifeClicker.exe'), ['--no-browser'], { cwd: fixture, windowsHide: true });
let output = ''; child.stdout.on('data', b => { output += b; }); child.stderr.on('data', b => { output += b; });
const delay = ms => new Promise(r => setTimeout(r, ms));
async function call(route, body) {
  const r = await fetch('http://127.0.0.1:8765' + route, { method: body === undefined ? 'GET' : 'POST', headers: { 'Content-Type': 'application/json' }, body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(90000) });
  assert.equal(r.status, 200, `${route}: ${await r.clone().text()}`);
  assert.match(r.headers.get('cache-control'), /no-store/);
  return r.json();
}
try {
  let ready = false;
  for (let i = 0; i < 100; i++) { try { if ((await call('/api/version')).launcherVersion === '3.0.0') { ready = true; break; } } catch {} await delay(100); }
  assert.ok(ready, output);
  assert.equal((await call('/api/integrity')).ok, true);
  const automatic = await call('/api/online-update'); assert.equal(automatic.error, undefined);
  const manual = await call('/api/online-update?force=1'); assert.equal(manual.error, undefined); assert.notEqual(manual.checkedAt, automatic.checkedAt);
  // Explicit test request, not a UI auto-install. Abort before POST if a release is available.
  assert.equal(manual.available, false, 'A newer release exists: do not install it in this API test');
  const current = await call('/api/install-online-update', {});
  assert.equal(current.gameInstalled, false); assert.equal(current.launcherStaged, false);
  const snapshot = { data: { LC4_PROFILES: '[]', LC4_SAVE_fixture: '{"saveVersion":12}' } };
  const saved = await call('/api/backup-saves', snapshot); assert.equal(saved.ok, true);
  assert.ok((await call('/api/backups')).backups.some(b => b.name === saved.name));
  assert.deepEqual(await call('/api/backup?name=' + encodeURIComponent(saved.name)), snapshot);
  const html = fs.readFileSync(path.join(fixture, 'game/index.html'), 'utf8');
  const result = await call('/api/apply-update', { version: '7.0.7', saveVersion: 12, notes: ['Fixture reinstall'], appHtml: html, saveSnapshot: snapshot });
  assert.equal(result.gameInstalled, true);
  assert.equal(fs.readFileSync(path.join(fixture, 'game/index.html'), 'utf8'), html);
  assert.equal((await call('/api/version')).saveVersion, 12);
  assert.equal((await call('/api/open-folder?target=userdata', {})).ok, true);
  assert.equal((await call('/api/shutdown', {})).ok, true);
  const exitCode = child.exitCode === null ? (await once(child, 'exit'))[0] : child.exitCode;
  assert.equal(exitCode, 0);
  console.log(`PASS every launcher-used API: version, integrity, automatic/manual checks, explicit no-op online install, fixture backup/list/read, transactional local install, open temporary data folder, shutdown. Fixture: ${fixture}`);
} finally { if (child.exitCode === null) child.kill(); }
