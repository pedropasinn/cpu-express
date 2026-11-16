// Original pixel geometry. Run locally with Node; no dependencies or image input.
const fs = require('node:fs');
const path = require('node:path');
const colors = { d:'#342016', w:'#794E2D', l:'#B27C48', c:'#FFF6DE', b:'#F6D080', g:'#BF8D37', t:'#1C7064', s:'#104640', a:'#2F647E', r:'#B23A30' };
let pixels;
function rect(x,y,w,h,c) { for(let j=y;j<y+h;j++) for(let i=x;i<x+w;i++) pixels[j][i]=c; }
function coin(x,y) {
  rect(x+2,y,6,10,'d'); rect(x+1,y+1,8,8,'d'); rect(x,y+2,10,6,'d');
  rect(x+2,y+1,6,8,'g');rect(x+1,y+2,8,6,'g');
  rect(x+3,y+1,4,1,'c');rect(x+1,y+3,1,3,'b');
  rect(x+3,y+3,4,4,'d');rect(x+2,y+4,6,2,'d');
  rect(x+4,y+2,2,6,'d');rect(x+4,y+4,2,2,'b');
  rect(x+3,y+3,1,1,'b');rect(x+6,y+6,1,1,'b');
}
function save(name,draw) {
  pixels=Array.from({length:16},()=>Array(16).fill(null)); draw();
  // Keep a one-cell dark silhouette without overlapping final rectangles.
  const before=pixels.map(row=>row.slice());
  for(let y=0;y<16;y++) for(let x=0;x<16;x++) if(before[y][x] && [[0,1],[0,-1],[1,0],[-1,0]].some(([dx,dy])=>!before[y+dy]?.[x+dx])) pixels[y][x]='d';
  function svg(scale=1) {
    const lines=[`<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 ${16*scale} ${16*scale}' shape-rendering='crispEdges'>`];
    for(let y=0;y<16;y++) for(let x=0;x<16;) {
      const color=pixels[y][x]; let end=x+1;
      while(end<16 && pixels[y][end]===color) end++;
      if(color) lines.push(`<rect x='${x*scale}' y='${y*scale}' width='${(end-x)*scale}' height='${scale}' fill='${colors[color]}'/>`);
      x=end;
    }
    return lines.join('\n')+'\n</svg>\n';
  }
  fs.writeFileSync(path.join(__dirname,name+'.svg'),svg());
  if(name==='moeda') {
    fs.mkdirSync(path.join(__dirname,'grandes'),{recursive:true});
    fs.writeFileSync(path.join(__dirname,'grandes','moeda-48.svg'),svg(3));
  }
}
save('moeda',()=>coin(3,3));
save('pilha-moedas',()=>{rect(1,9,9,5,'d');rect(2,10,7,1,'g');rect(2,12,7,1,'b');rect(6,7,9,7,'d');rect(7,8,7,1,'b');rect(7,10,7,1,'g');rect(7,12,7,1,'b');coin(3,1);});
save('saco-moedas',()=>{rect(5,1,7,3,'d');rect(6,2,5,2,'b');rect(6,4,5,2,'w');rect(4,5,9,9,'w');rect(2,8,13,6,'w');rect(4,6,3,6,'l');rect(6,7,5,5,'g');rect(8,7,1,5,'d');rect(6,9,5,1,'d');rect(8,9,1,1,'b');rect(5,4,7,1,'d');});
save('caixa-patio',()=>{rect(1,3,14,11,'w');rect(2,4,12,8,'l');rect(3,5,10,2,'w');rect(3,9,10,2,'w');rect(2,7,12,1,'d');for(let i=0;i<9;i++)rect(3+i,4+i,2,1,'b');rect(2,3,12,1,'g');rect(2,13,12,1,'d');});
save('caixa-oficina',()=>{rect(1,5,14,9,'a');rect(5,2,6,4,'d');rect(6,3,4,2,'c');rect(1,8,14,2,'d');rect(3,6,1,7,'c');rect(12,6,1,7,'c');rect(6,8,4,3,'d');rect(7,9,2,1,'g');});
save('caixa-plataforma',()=>{rect(5,2,6,4,'d');rect(6,3,4,2,'b');rect(1,5,14,9,'t');rect(2,6,12,7,'s');rect(4,5,2,9,'b');rect(10,5,2,9,'b');rect(7,8,2,3,'c');rect(2,6,1,1,'c');});
save('caixa-garagem',()=>{rect(1,6,14,8,'w');rect(2,5,12,2,'d');rect(4,3,8,3,'d');rect(6,2,4,2,'d');rect(4,5,8,1,'r');rect(4,8,8,6,'d');rect(5,9,6,1,'l');rect(5,11,6,1,'l');rect(6,12,1,2,'c');rect(9,12,1,2,'c');});
save('caixa-atelie',()=>{rect(4,2,8,3,'d');rect(3,4,10,10,'a');rect(2,5,1,6,'d');rect(13,5,1,6,'d');rect(4,4,8,2,'c');rect(4,7,8,5,'b');rect(5,8,6,3,'r');rect(4,12,8,1,'c');rect(10,5,2,4,'a');});
console.log('8 sprites originais + moeda 48 x 48 gerados.');
