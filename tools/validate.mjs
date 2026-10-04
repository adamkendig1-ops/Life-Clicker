import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export function text(file) { return new TextDecoder('utf-8', { fatal: true }).decode(fs.readFileSync(file)); }
export function validate() {
  for (const file of ['game/index.html', 'game/version.json', 'launcher-ui/launcher.html', 'launcher-ui/launcher.js', 'launcher-config.json']) {
    const contents = text(path.join(root, file));
    if (file.endsWith('.json')) JSON.parse(contents);
    const scripts = file.endsWith('.html') ? [...contents.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/gi)].filter(m => !/\bsrc=/.test(m[1])).map(m => m[2]) : file.endsWith('.js') ? [contents] : [];
    for (const script of scripts) {
      const result = spawnSync(process.execPath, ['--check'], { input: script, encoding: 'utf8' });
      if (result.status !== 0) throw new Error(`${file}: ${result.stderr}`);
    }
    console.log(`PASS UTF-8 / syntax / required file: ${file}`);
  }
  const meta = JSON.parse(text(path.join(root, 'game/version.json')));
  const html = text(path.join(root, 'game/index.html'));
  if (!html.includes(`VERSION='${meta.version}'`) || !html.includes(`SAVE_VERSION=${meta.saveVersion}`)) throw new Error('Game metadata mismatch');
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) validate();
