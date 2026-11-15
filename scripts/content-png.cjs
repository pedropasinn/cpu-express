// Lossless local P6 -> PNG export, optionally enlarged with integer nearest pixels.
// Usage: node scripts/content-png.cjs source.ppm new-output.png [integer-scale]
const fs = require('node:fs');
const zlib = require('node:zlib');
const [source, destination, rawScale = '1'] = process.argv.slice(2);
const scale = Number(rawScale);
if (!source || !destination || !Number.isInteger(scale) || scale < 1 || scale > 8) {
  throw new Error('Informe origem PPM, novo destino PNG e escala inteira de 1 a 8.');
}
const input = fs.readFileSync(source);
const header = /^P6\n(\d+) (\d+)\n255\n/.exec(input.subarray(0, 100).toString('ascii'));
if (!header) throw new Error('Esperado P6 RGB de 8 bits exportado pelas galerias.');
const width = Number(header[1]);
const height = Number(header[2]);
const pixels = input.subarray(header[0].length);
if (pixels.length !== width * height * 3) throw new Error('Tamanho PPM incoerente.');
const outWidth = width * scale;
const outHeight = height * scale;
const scanlines = Buffer.alloc(outHeight * (outWidth * 3 + 1));
for (let y = 0; y < outHeight; y++) {
  for (let x = 0; x < outWidth; x++) {
    const src = (Math.floor(y / scale) * width + Math.floor(x / scale)) * 3;
    const dst = y * (outWidth * 3 + 1) + 1 + x * 3;
    pixels.copy(scanlines, dst, src, src + 3);
  }
}
function chunk(type, data) {
  const payload = Buffer.concat([Buffer.from(type), data]);
  let crc = 0xffffffff;
  for (const byte of payload) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
  }
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const checksum = Buffer.alloc(4);
  checksum.writeUInt32BE((crc ^ 0xffffffff) >>> 0);
  return Buffer.concat([length, payload, checksum]);
}
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(outWidth, 0);
ihdr.writeUInt32BE(outHeight, 4);
ihdr[8] = 8;
ihdr[9] = 2;
fs.writeFileSync(destination, Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  chunk('IHDR', ihdr), chunk('IDAT', zlib.deflateSync(scanlines)), chunk('IEND', Buffer.alloc(0)),
]), { flag: 'wx' });
console.log(`${destination}: ${outWidth} × ${outHeight}, escala nearest ${scale}×`);
