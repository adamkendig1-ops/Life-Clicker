// Real Windows process/HTTP tests. Never launches a browser or reads browser saves.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import net from 'node:net';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { root } from './validate.mjs';
const exe = path.resolve(process.argv[2] || 'target/release/LifeClicker.exe');
const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'LifeClicker-smoke-'));
const children = new Set();
const delay = ms => new Promise(r => setTimeout(r, ms));
function launch(dir, args = []) {
  const child = spawn(path.join(dir, 'LifeClicker.exe'), ['--root', dir, '--no-browser', ...args], { windowsHide: true });
  child.output = ''; child.stdout.on('data', b => { child.output += b; }); child.stderr.on('data', b => { child.output += b; });
  children.add(child); child.on('exit', () => children.delete(child)); return child;
}
async function exited(child) { if (child.exitCode !== null) return child.exitCode; return (await Promise.race([once(child, 'exit'), delay(15000).then(() => { throw new Error(`Process timeout: ${child.output}`); })]))[0]; }
async function free(port) {
  const server = net.createServer();
  await new Promise((resolve, reject) => { server.once('error', error => reject(new Error(`Cannot reserve port ${port} (${error.code}): stop test without touching existing process`))); server.listen(port, '127.0.0.1', resolve); });
  await new Promise(r => server.close(r));
}
async function request(port, route, body) {
  const response = await fetch(`http://127.0.0.1:${port}${route}`, { method: body === undefined ? 'GET' : 'POST', headers: { 'Content-Type': 'application/json' }, body: body === undefined ? undefined : typeof body === 'string' ? body : JSON.stringify(body), signal: AbortSignal.timeout(5000) });
  return { status: response.status, headers: response.headers, text: await response.text() };
}
async function ready(child, port) {
  for (let i = 0; i < 100; i++) {
    if (child.exitCode !== null) throw new Error(child.output);
    try { const r = await request(port, '/api/version'); if (r.status === 200) return; } catch {}
    await delay(100);
  }
  throw new Error(`Server startup timeout: ${child.output}`);
}
function fixture(name) {
  const dir = path.join(temp, name); fs.mkdirSync(dir);
  fs.copyFileSync(exe, path.join(dir, 'LifeClicker.exe'));
  for (const p of ['game', 'launcher-ui']) fs.cpSync(path.join(root, p), path.join(dir, p), { recursive: true });
  fs.writeFileSync(path.join(dir, 'launcher-config.json'), JSON.stringify({ channel: 'test', manifestUrl: '' }));
  return dir;
}
try {
  await free(8765); await free(8766);
  const prod = fixture('production'); const dev = fixture('development');
  const p = launch(prod); const d = launch(dev, ['--dev']);
  await ready(p, 8765); await ready(d, 8766);
  for (const [port, isDev] of [[8765, false], [8766, true]]) {
    const result = await request(port, '/api/version');
    assert.equal(result.headers.get('cache-control'), 'no-store, no-cache, must-revalidate, max-age=0');
    const version = JSON.parse(result.text); assert.equal(version.version, '7.0.7'); assert.equal(version.saveVersion, 12); assert.equal(version.launcherVersion, '3.0.0'); assert.equal(version.devMode, isDev);
    for (const route of ['/launcher.html', '/launcher.js', '/game/index.html', '/game/version.json', '/api/integrity']) assert.equal((await request(port, route)).status, 200, route);
    assert.equal((await request(port, '/game/index.html')).text, fs.readFileSync(path.join(root, 'game/index.html'), 'utf8'));
    const duplicate = launch(isDev ? dev : prod, isDev ? ['--dev'] : []);
    assert.equal(await exited(duplicate), 0); assert.match(duplicate.output, /Existing Life Clicker/);
  }
  const snap = { data: { LC4_PROFILES: '[]', LC4_SAVE_fixture: '{"saveVersion":12}' } };
  assert.equal((await request(8765, '/api/apply-update', '{')).status, 400);
  const before = fs.readFileSync(path.join(prod, 'game/index.html'));
  assert.equal((await request(8765, '/api/apply-update', { version: '7.0.8', saveVersion: 12, appHtml: 'bad', saveSnapshot: snap })).status, 400);
  assert.deepEqual(fs.readFileSync(path.join(prod, 'game/index.html')), before);
  for (let i = 0; i < 27; i++) assert.equal((await request(8765, '/api/backup-saves', snap)).status, 200);
  assert.equal(JSON.parse((await request(8765, '/api/backups')).text).backups.length, 25);
  const pkg = JSON.parse(fs.readFileSync(path.join(root, 'stable/Life_Clicker_Update_7.0.7_Auto_Update_Validation.lcupdate')));
  assert.equal((await request(8765, '/api/apply-update', { ...pkg, saveSnapshot: snap })).status, 200);
  assert.deepEqual(fs.readFileSync(path.join(prod, 'game/index.html')), before);
  assert.equal(fs.readdirSync(path.join(prod, 'userdata/game-backups')).length, 1);
  const backupList = JSON.parse((await request(8765, '/api/backups')).text).backups;
  assert.deepEqual(JSON.parse((await request(8765, `/api/backup?name=${backupList[0].name}`)).text), snap);
  assert.equal((await request(8766, '/api/apply-update', pkg)).status, 400);
  assert.equal((await request(8766, '/api/install-online-update', { saveSnapshot: snap })).status, 400);
  assert.equal((await request(8766, '/api/backup-saves', snap)).status, 200);
  assert.equal(fs.existsSync(path.join(dev, 'userdata')), false);
  assert.equal(fs.existsSync(path.join(dev, 'dev-userdata/backups')), true);
  assert.equal((await request(8765, '/api/dev')).status, 404);
  assert.equal(JSON.parse((await request(8766, '/api/dev')).text).devMode, true);
  for (const port of [8765, 8766]) assert.equal((await request(port, '/api/shutdown', {})).status, 200);
  assert.equal(await exited(p), 0); assert.equal(await exited(d), 0);
  const unrelated = net.createServer(socket => { socket.once('data', () => socket.end('HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}')); });
  await new Promise(r => unrelated.listen(8765, '127.0.0.1', r));
  try { const blocked = launch(prod); assert.equal(await exited(blocked), 1); assert.match(blocked.output, /Another application occupies/); }
  finally { await new Promise(r => unrelated.close(r)); }
  console.log(`PASS production 8765, dev 8766, APIs/assets, isolated saves, invalid updates, backups/retention, occupied ports, clean shutdown. Fixtures: ${temp}`);
} finally {
  for (const child of children) child.kill();
  // Retain fixtures as reviewable evidence. No recursive removal and no real saves accessed.
}
