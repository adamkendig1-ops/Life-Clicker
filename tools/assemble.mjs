import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { root, validate } from './validate.mjs';
validate();
const binary = path.resolve(process.argv[2] || 'target/release/LifeClicker.exe');
const destination = path.resolve(process.argv[3] || 'dist/LifeClicker-3.0.0');
if (fs.existsSync(destination)) throw new Error('Output directory already exists; use a new directory');
if (!fs.readFileSync(binary).subarray(0, 2).equals(Buffer.from('MZ'))) throw new Error('Not a Windows executable');
fs.mkdirSync(destination, { recursive: true });
fs.copyFileSync(binary, path.join(destination, 'LifeClicker.exe'));
for (const name of ['game/index.html', 'game/version.json', 'launcher-ui/launcher.html', 'launcher-ui/launcher.js', 'launcher-config.json', 'README_FIRST.txt']) {
  fs.mkdirSync(path.dirname(path.join(destination, name)), { recursive: true });
  fs.copyFileSync(path.join(root, name), path.join(destination, name));
}
const hash = crypto.createHash('sha256').update(fs.readFileSync(binary)).digest('hex');
fs.writeFileSync(path.join(destination, 'LifeClicker.exe.sha256'), hash + '\n');
console.log(JSON.stringify({ destination, executable: 'LifeClicker.exe', sha256: hash }, null, 2));
