// Execute the actual launcher JavaScript with a small browser API fixture.
import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const script = fs.readFileSync('launcher-ui/launcher.js', 'utf8');
async function run(fail = false, dev = false) {
  const elements = new Map(); const calls = [];
  function element(id) {
    if (!elements.has(id)) elements.set(id, { textContent: '', src: 'about:blank', classList: { add() {}, remove() {} }, addEventListener() {}, replaceChildren() {}, append() {}, files: [] });
    return elements.get(id);
  }
  const context = {
    document: { getElementById: element, querySelector: element, querySelectorAll: () => [], body: element('body') },
    window: { addEventListener() {} }, location: { origin: `http://127.0.0.1:${dev ? 8766 : 8765}` },
    localStorage: { length: 0 }, AbortController, setTimeout, clearTimeout,
    fetch: async url => { calls.push(url); if (fail) throw new Error('fixture API failure'); return { ok: true, json: async () => url === '/api/version' ? { version: '7.0.7', saveVersion: 12, launcherVersion: '3.0.0', devMode: dev } : { configured: true, available: false } }; }
  };
  vm.runInNewContext(script, context);
  await new Promise(r => setTimeout(r, 20));
  if (fail) { assert.match(element('versionLine').textContent, /initialization failed: fixture API failure/); assert.equal(calls.length, 1); }
  else {
    assert.match(element('versionLine').textContent, /Game 7.0.7 · Launcher 3.0.0 · Save Version 12/);
    assert.equal(calls.filter(c => c === '/api/online-update').length, 1);
    assert.equal(calls.some(c => c.includes('install')), false);
    if (dev) { assert.match(element('versionLine').textContent, /DEV MODE/); assert.equal(element('installUpdate').disabled, true); }
    await element('checkOnlineSide').onclick(); assert.equal(calls.filter(c => c === '/api/online-update?force=1').length, 1);
  }
}
await run(); await run(true); await run(false, true);
console.log('PASS UI startup, visible API failures, exactly one background check, manual check, no automatic install, DEV MODE');
