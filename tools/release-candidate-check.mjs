// Validate an assembled/extracted installation with no repository asset dependency.
// The executable's cwd is the installation; no --root argument or default browser.
// UI logic uses synthetic browser APIs and never reads real browser storage.
import fs from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import vm from 'node:vm';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';

const installation = path.resolve(process.argv[2]);
const interactive = process.argv.includes('--serve');
const base = 'http://127.0.0.1:8765';
const files = ['LifeClicker.exe', 'LifeClicker.exe.sha256', 'launcher-ui/launcher.html', 'launcher-ui/launcher.js', 'game/index.html', 'game/version.json', 'launcher-config.json', 'README_FIRST.txt'];
const hash = b => crypto.createHash('sha256').update(b).digest('hex');
const before = Object.fromEntries(files.map(f => [f, hash(fs.readFileSync(path.join(installation, f)))]));
const probe = net.createServer();
await new Promise((resolve, reject) => { probe.once('error', reject); probe.listen(8765, '127.0.0.1', resolve); });
await new Promise(r => probe.close(r));
const launch = () => spawn(path.join(installation, 'LifeClicker.exe'), ['--no-browser'], { cwd: installation, windowsHide: true });
const child = launch();
let output = '';
child.stdout.on('data', b => { output += b; }); child.stderr.on('data', b => { output += b; });
const delay = ms => new Promise(r => setTimeout(r, ms));
async function exited(process) {
  if (process.exitCode !== null) return process.exitCode;
  let timer;
  try { return (await Promise.race([once(process, 'exit'), new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('Process exit timeout')), 15000); })]))[0]; }
  finally { clearTimeout(timer); }
}
async function request(route, options) { return fetch(base + route, { ...options, signal: AbortSignal.timeout(90000) }); }
try {
  let ready = false;
  for (let i = 0; i < 100; i++) {
    if (child.exitCode !== null) throw new Error(output);
    try { if ((await request('/api/version')).ok) { ready = true; break; } } catch {}
    await delay(100);
  }
  assert.ok(ready, output);
  const version = await (await request('/api/version')).json();
  assert.deepEqual(version, { devMode: false, launcherVersion: '3.0.0', product: 'Life Clicker', saveVersion: 12, version: '7.0.7' });
  for (const [route, source] of [['/launcher.html', 'launcher-ui/launcher.html'], ['/launcher.js', 'launcher-ui/launcher.js'], ['/game/index.html', 'game/index.html']]) {
    const r = await request(route); assert.equal(r.status, 200); assert.match(r.headers.get('cache-control'), /no-store/);
    assert.equal(hash(Buffer.from(await r.arrayBuffer())), before[source]);
  }
  const duplicate = launch(); let duplicateOutput = '';
  duplicate.stdout.on('data', b => { duplicateOutput += b; }); duplicate.stderr.on('data', b => { duplicateOutput += b; });
  assert.equal(await exited(duplicate), 0); assert.match(duplicateOutput, /Existing Life Clicker instance at 127.0.0.1:8765/);
  console.log(`PASS assembled installation started from ${installation}; production API, exact assets and second instance. PID ${child.pid}`);
  if (interactive) {
    console.log('READY for browser inspection. Close using POST /api/shutdown or the launcher Exit button.');
    await once(child, 'exit');
  } else {
    const elements = new Map(); const calls = []; const responses = [];
    const element = id => {
      if (!elements.has(id)) elements.set(id, { textContent: '', src: 'about:blank', classList: { add() {}, remove() {} }, addEventListener() {}, replaceChildren() {}, append() {}, files: [] });
      return elements.get(id);
    };
    const context = {
      document: { getElementById: element, querySelector: element, querySelectorAll: () => [], body: element('body') },
      window: { addEventListener() {} }, location: { origin: base }, AbortController, setTimeout, clearTimeout,
      localStorage: new Proxy({}, { get() { throw new Error('Unexpected browser-storage access during startup/update checks'); } }),
      fetch: async (route, options) => {
        calls.push(route);
        assert.ok(!route.includes('install') && !route.includes('apply') && !route.includes('backup'), `Unexpected mutation ${route}`);
        const r = await request(route, options); const data = await r.json(); responses.push({ route, data });
        return { ok: r.ok, status: r.status, json: async () => data };
      }
    };
    vm.runInNewContext(await (await request('/launcher.js')).text(), context);
    for (let i = 0; i < 300 && !responses.some(r => r.route === '/api/online-update'); i++) await delay(100);
    assert.equal(element('versionLine').textContent, 'Game 7.0.7 · Launcher 3.0.0 · Save Version 12');
    assert.equal(calls.filter(r => r === '/api/online-update').length, 1);
    const startup = responses.find(r => r.route === '/api/online-update')?.data;
    assert.ok(startup?.configured, 'Live stable feed must be configured'); assert.equal(startup.error, undefined, startup.error);
    await element('checkOnlineSide').onclick();
    const manual = responses.find(r => r.route === '/api/online-update?force=1')?.data;
    assert.ok(manual?.configured); assert.equal(manual.error, undefined, manual.error); assert.notEqual(manual.checkedAt, startup.checkedAt);
    console.log(`PASS served UI logic: versions, no DEV MODE, exactly one startup check, live HTTPS feed and manual refresh, zero save access or unapproved installation. Status: ${element('onlineStatus').textContent}`);
    await request('/api/shutdown', { method: 'POST' });
  }
  assert.equal(await exited(child), 0, output);
  for (const f of files) assert.equal(hash(fs.readFileSync(path.join(installation, f))), before[f], `Unexpected modification: ${f}`);
  assert.equal(fs.existsSync(path.join(installation, 'userdata')), false, 'No save access/update should create userdata');
  console.log('PASS clean shutdown, unchanged installation and no userdata; no repository files were served.');
} finally { if (child.exitCode === null) child.kill(); }
