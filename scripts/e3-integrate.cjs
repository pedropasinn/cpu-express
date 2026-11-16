const fs = require('fs');
// One-time source transformation retained as implementation history; never reapply it.
if (fs.readFileSync('src/simulation/world.rs', 'utf8').includes('active_line: String')) {
  console.log('Integração E3 já aplicada. Nenhum arquivo alterado.');
  process.exit(0);
}
for (const name of ['economy', 'economy_goals', 'objectives', 'passenger_contracts']) {
  const path = `src/simulation/${name}.rs`;
  let text = fs.readFileSync(path, 'utf8');
  text = text.replace(/&ServiceState\b(?!<)/g, '&ServiceState<impl Clone + Eq>');
  fs.writeFileSync(path, text);
}
const linePath = 'data/lines/curitiba-paranagua.json';
let line = fs.readFileSync(linePath, 'utf8');
line = line.replace('"status": "draft"', '"status": "playable"')
  .replace('"open_questions": [', '"open_questions": [],\n  "editorial_notes": [\n    "Os serviços mostrados são do jogo e não representam a operação real.",');
fs.writeFileSync(linePath, line);
