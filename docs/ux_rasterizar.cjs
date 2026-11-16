// Local fallback for ux_rasterizar.py when Python cannot start on Windows.
// Same triangle coverage, premultiplied colors, texture filtering and PNG output.
// Usage: node docs/ux_rasterizar.cjs <json file or directory>
const fs = require('node:fs');
const path = require('node:path');
const zlib = require('node:zlib');
function chunk(type, data) {
  const payload = Buffer.concat([Buffer.from(type), data]);
  let crc = 0xffffffff;
  for (const byte of payload) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
  }
  const length = Buffer.alloc(4); length.writeUInt32BE(data.length);
  const sum = Buffer.alloc(4); sum.writeUInt32BE((crc ^ 0xffffffff) >>> 0);
  return Buffer.concat([length, payload, sum]);
}
const clamp = (n, lo, hi) => Math.max(lo, Math.min(hi, n));
function render(file) {
  const data = JSON.parse(fs.readFileSync(file, 'utf8'));
  const [width, height] = data.size, ppp = data.pixels_per_point;
  const canvas = new Float64Array(width * height * 4);
  for (let i = 3; i < canvas.length; i += 4) canvas[i] = 1;
  const textures = new Map(data.textures.map(t => [t.id,t]));
  for (const mesh of data.meshes) {
    const tex = textures.get(mesh.texture), [tw,th] = tex.size;
    const vertices = mesh.vertices.map(v => [v[0]*ppp, v[1]*ppp,...v.slice(2)]);
    const clip = mesh.clip.map(v => v*ppp);
    for (let i=0; i<mesh.indices.length; i+=3) {
      let [a,b,c] = mesh.indices.slice(i,i+3).map(n=>vertices[n]);
      let area=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]);
      if (Math.abs(area)<1e-9) continue;
      if (area<0) { [b,c]=[c,b]; area=-area; }
      const x0=Math.max(0,Math.ceil(clip[0]-.5),Math.floor(Math.min(a[0],b[0],c[0])));
      const y0=Math.max(0,Math.ceil(clip[1]-.5),Math.floor(Math.min(a[1],b[1],c[1])));
      const x1=Math.min(width,Math.ceil(clip[2]-.5),Math.ceil(Math.max(a[0],b[0],c[0])));
      const y1=Math.min(height,Math.ceil(clip[3]-.5),Math.ceil(Math.max(a[1],b[1],c[1])));
      const edges=[[b,c],[c,a],[a,b]].map(([u,v])=>[v[0]-u[0],v[1]-u[1],u[0],u[1],v[1]<u[1]||(v[1]===u[1]&&v[0]>u[0])]);
      for (let y=y0;y<y1;y++) for(let x=x0;x<x1;x++) {
        const values=edges.map(([dx,dy,ux,uy])=>dx*(y+.5-uy)-dy*(x+.5-ux));
        if(values.some((v,k)=>!(v>1e-8||(Math.abs(v)<=1e-8&&edges[k][4])))) continue;
        const w=values.map(v=>v/area);
        const u=w[0]*a[2]+w[1]*b[2]+w[2]*c[2], v=w[0]*a[3]+w[1]*b[3]+w[2]*c[3];
        let color;
        if(tex.linear) {
          const tx=u*tw-.5, ty=v*th-.5, lx=Math.floor(tx), ly=Math.floor(ty), fx=tx-lx, fy=ty-ly;
          const at=(xx,yy)=>tex.pixels[clamp(yy,0,th-1)*tw+clamp(xx,0,tw-1)];
          const p=at(lx,ly),q=at(lx+1,ly),r=at(lx,ly+1),s=at(lx+1,ly+1);
          color=p.map((n,k)=>((n*(1-fx)+q[k]*fx)*(1-fy)+(r[k]*(1-fx)+s[k]*fx)*fy)/255);
        } else color=tex.pixels[clamp(Math.floor(v*th),0,th-1)*tw+clamp(Math.floor(u*tw),0,tw-1)].map(n=>n/255);
        const src=color.map((n,k)=>n*(w[0]*a[k+4]+w[1]*b[k+4]+w[2]*c[k+4])/255);
        const dest=(y*width+x)*4;
        for(let k=0;k<4;k++) canvas[dest+k]=src[k]+canvas[dest+k]*(1-src[3]);
      }
    }
  }
  const scanlines=Buffer.alloc(height*(width*3+1));
  for(let y=0;y<height;y++) for(let x=0;x<width;x++) for(let k=0;k<3;k++) scanlines[y*(width*3+1)+1+x*3+k]=Math.round(clamp(canvas[(y*width+x)*4+k],0,1)*255);
  const ihdr=Buffer.alloc(13); ihdr.writeUInt32BE(width,0); ihdr.writeUInt32BE(height,4); ihdr[8]=8;ihdr[9]=2;
  const out=file.replace(/\.json$/,'.png');
  fs.writeFileSync(out,Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',ihdr),chunk('IDAT',zlib.deflateSync(scanlines)),chunk('IEND',Buffer.alloc(0))]));
  console.log(out);
}
for(const arg of process.argv.slice(2)) {
  if(fs.statSync(arg).isDirectory()) for(const f of fs.readdirSync(arg).filter(f=>f.endsWith('.json'))) render(path.join(arg,f));
  else render(arg);
}
