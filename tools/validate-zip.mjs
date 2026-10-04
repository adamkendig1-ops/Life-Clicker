// Independently inflate every entry and verify ZIP CRC-32, sizes and file list.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import { inflateRawSync } from 'node:zlib';
const bytes = fs.readFileSync(process.argv[2]);
const table = Array.from({ length: 256 }, (_, n) => {
  for (let i = 0; i < 8; i++) n = n & 1 ? 0xedb88320 ^ (n >>> 1) : n >>> 1;
  return n >>> 0;
});
function crc32(data) {
  let crc = 0xffffffff;
  for (const byte of data) crc = table[(crc ^ byte) & 255] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}
let end = -1;
for (let i = bytes.length - 22; i >= Math.max(0, bytes.length - 65557); i--) {
  if (bytes.readUInt32LE(i) === 0x06054b50 && i + 22 + bytes.readUInt16LE(i + 20) === bytes.length) { end = i; break; }
}
assert.ok(end >= 0, 'Missing ZIP central directory');
assert.equal(bytes.readUInt16LE(end + 4), 0, 'Multi-disk ZIP unsupported');
const count = bytes.readUInt16LE(end + 10);
assert.equal(count, 8, 'Unexpected release file count');
let offset = bytes.readUInt32LE(end + 16);
const names = [];
for (let i = 0; i < count; i++) {
  assert.equal(bytes.readUInt32LE(offset), 0x02014b50);
  const flags = bytes.readUInt16LE(offset + 8), method = bytes.readUInt16LE(offset + 10);
  assert.equal(flags & 1, 0, 'Encrypted ZIP unsupported');
  const expectedCRC = bytes.readUInt32LE(offset + 16), compressedSize = bytes.readUInt32LE(offset + 20), size = bytes.readUInt32LE(offset + 24);
  const nameLength = bytes.readUInt16LE(offset + 28), extraLength = bytes.readUInt16LE(offset + 30), commentLength = bytes.readUInt16LE(offset + 32);
  const name = bytes.subarray(offset + 46, offset + 46 + nameLength).toString('utf8');
  const local = bytes.readUInt32LE(offset + 42);
  assert.equal(bytes.readUInt32LE(local), 0x04034b50);
  const dataStart = local + 30 + bytes.readUInt16LE(local + 26) + bytes.readUInt16LE(local + 28);
  const compressed = bytes.subarray(dataStart, dataStart + compressedSize);
  assert.ok(method === 0 || method === 8, `Unsupported ZIP method ${method}`);
  const data = method === 8 ? inflateRawSync(compressed) : compressed;
  assert.equal(data.length, size, `Size mismatch: ${name}`);
  assert.equal(crc32(data), expectedCRC, `CRC mismatch: ${name}`);
  names.push(name); console.log(`PASS ZIP CRC-32: ${name} (${size} bytes)`);
  offset += 46 + nameLength + extraLength + commentLength;
}
assert.deepEqual(names.sort(), ['LifeClicker.exe', 'LifeClicker.exe.sha256', 'README_FIRST.txt', 'game/index.html', 'game/version.json', 'launcher-config.json', 'launcher-ui/launcher.html', 'launcher-ui/launcher.js'].sort());
console.log('PASS all 8 ZIP entries: CRC-32, decompression, size and exact file list');
