// Prepare a reviewable patch; never modify the protected app sources.
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const cp = require('node:child_process');
const root = path.resolve(__dirname, '..');
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'cpu-content-adapters-'));
const files = new Map();
function read(file) {
  return fs.readFileSync(path.join(root, file), 'utf8').replaceAll('\r\n', '\n');
}
function replace(text, pattern, next) {
  if (!pattern.test(text)) throw new Error(`Adaptador não encontrado: ${pattern}`);
  return text.replace(pattern, next);
}
let app = read('src/app.rs');
app = replace(app, /fn service_wagon\(style: CoachStyle\)[\s\S]*?\n}\n/,
  'fn service_wagon(style: CoachStyle) -> crate::simulation::ServiceWagon {\n    style.into()\n}\n');
app = replace(app, /fn coach_style\(style: crate::simulation::ServiceWagon\)[\s\S]*?\n}\n/,
  'fn coach_style(style: crate::simulation::ServiceWagon) -> CoachStyle {\n    style.into()\n}\n');
app = replace(app, /fn coach_name\(style: CoachStyle\)[\s\S]*?\n}\n/,
  "fn coach_name(style: CoachStyle) -> &'static str {\n    style.name()\n}\n");
app = replace(app, /use crate::\{rendering::EngineStyle, simulation::service::ServiceReward};\n            let reward = match config.visual.engine \{[\s\S]*?\n            };/,
  'use crate::rendering::EngineStyle;\n            let reward = config.visual.engine.required_reward();');
files.set('src/app.rs', app);
let storage = read('src/app/storage.rs');
storage = replace(storage, /use crate::\{rendering::EngineStyle, simulation::service::ServiceReward};\n                        let requirement = match self.config.visual.engine \{[\s\S]*?\n                        };/,
  'use crate::rendering::EngineStyle;\n                        let requirement = self.config.visual.engine.required_reward();');
files.set('src/app/storage.rs', storage);
let workshop = read('src/app/workshop.rs');
workshop = replace(workshop, /let category = match contract.kind \{[\s\S]*?\n                };/,
  'let category = contract.kind.name();');
workshop = replace(workshop, /let choices = \[\n                    CoachStyle::Passenger,[\s\S]*?\n                ];/,
  'let choices = CoachStyle::ALL;');
workshop = replace(workshop, /use crate::\{rendering::EngineStyle, simulation::service::ServiceReward};\n                    let requirement = match engine \{[\s\S]*?\n                    };/,
  'let requirement = engine.required_reward();');
workshop = replace(workshop, /let description = match engine \{[\s\S]*?\n                    };/,
  'let description = engine.unlock_description();');
files.set('src/app/workshop.rs', workshop);
let services = read('src/app/services.rs');
services = replace(services, /let \(cargo_art, wagon_hint\) = match c.kind \{[\s\S]*?\n                };/,
  'let cargo_art = match c.kind {\n                    CargoKind::Parcels => Sprite::Encomenda,\n                    CargoKind::Liquid => Sprite::Tanque,\n                    _ => Sprite::Caixote,\n                };\n                let wagon_hint = c.kind.wagon_hint();');
files.set('src/app/services.rs', services);
let patch = '';
let index = 0;
for (const [file, content] of files) {
  const before = path.join(temporary, `${index}-before.rs`);
  const after = path.join(temporary, `${index++}-after.rs`);
  fs.writeFileSync(before, read(file));
  fs.writeFileSync(after, content);
  let diff;
  try { diff = cp.execFileSync('git', ['diff', '--no-index', '--no-ext-diff', '--', before, after], { encoding: 'utf8' }); }
  catch (error) {
    if (error.status !== 1) throw error;
    diff = error.stdout;
  }
  diff = diff.replace(/^diff --git .*$/m, `diff --git a/${file} b/${file}`)
    .replace(/^--- .*$/m, `--- a/${file}`).replace(/^\+\+\+ .*$/m, `+++ b/${file}`);
  patch += diff;
}
fs.mkdirSync(path.join(root, 'docs/conteudo'), { recursive: true });
fs.writeFileSync(path.join(root, 'docs/conteudo/adaptadores-agente1.patch'), patch, { flag: 'wx' });
console.log('Patch preparado, sem aplicar alterações a src/app.rs ou src/app/*.rs.');
