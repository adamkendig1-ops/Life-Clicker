// Attach only to the developer server that the tester started, never production.
import assert from 'node:assert/strict';
const base = 'http://127.0.0.1:8766';
const request = async (route, body) => {
  const r = await fetch(base + route, { method: body === undefined ? 'GET' : 'POST', headers: { 'Content-Type': 'application/json' }, body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(5000) });
  return { status: r.status, data: await r.text(), headers: r.headers };
};
const version = JSON.parse((await request('/api/version')).data);
assert.equal(version.devMode, true); assert.equal(version.launcherVersion, '3.0.0'); assert.equal(version.saveVersion, 12);
for (const route of ['/launcher.html', '/launcher.js', '/game/index.html', '/game/version.json', '/api/integrity', '/api/dev']) {
  const r = await request(route); assert.equal(r.status, 200, route); assert.match(r.headers.get('cache-control'), /no-store/);
}
assert.equal((await request('/api/apply-update', {})).status, 400);
assert.equal((await request('/api/install-online-update', {})).status, 400);
assert.equal(JSON.parse((await request('/api/online-update')).data).configured, false);
console.log('PASS actual dev 127.0.0.1:8766: version, assets, integrity, dev info, cache headers, disabled updates');
