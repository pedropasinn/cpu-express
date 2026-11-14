# CPU Express — contrato comum para cinco agentes independentes

**Fonte congelada:** linha A, originalmente `CPU-Express-0.1.0-source(1)(1).zip`. Não trocar por outro fork. Rust edição 2024, MSRV declarada 1.95; manter `Cargo.lock`. Windows 10/11 x64 é o único alvo de produto nesta rodada. Se executar visualmente em Linux para investigação, isso não é validar um release Windows.

## Invariantes obrigatórias

1. O trem responde à utilização **real** da CPU global, com opção manual **claramente marcada como simulada**. Nunca fazer benchmark sintético para movimentar o trem. A coleta não pode travar a UI.
2. Velocidade de referência: 0% CPU → 12 km/h, 50% → 56, 100% → 100, com suavização exponencial e distância integrada no tempo, independentemente de FPS. Manter testes de precisão existentes.
3. Janela compacta na área útil perto da taskbar; **não embutida** na taskbar. Não roubar foco, não instalar hooks globais ou injetar DLL no Explorer. Fechar deve liberar threads e recursos.
4. Preservar renderer pixel art original da linha A (canvas de 320×70 ampliado com vizinho mais próximo) como baseline e opção de fallback. Não copiar arte de Tiny Rails nem qualquer outro jogo.
5. Nenhuma rede, conta, coleta remota, atualização automática, anúncios, instaladores, permissões elevadas ou persistência de métricas sem consentimento explícito. Configuração local pode continuar.
6. Preservar interfaces já usadas pelo `src/app.rs`. Mudanças de assinatura que exijam editar arquivos de outro agente devem ir em `INTEGRACAO.md` como patch proposto, e não ser feitas no arquivo alheio.
7. Preservar arquivos originais de licenciamento e atribuição no release final. O pacote de desenvolvimento foi reduzido para conter o necessário; consultar o original para distribuir binários.
8. Documentar ações Win32 `unsafe` e minimizar sua superfície. Reutilizar crates existentes; sugerir novas dependências com justificativa no retorno.

## Arquivos cuja edição é exclusiva

- Agente 1: `src/platform/**`, novos testes específicos do Windows; propor mudanças no manifest em `INTEGRACAO.md`.
- Agente 2: `src/rendering/**` e `assets/**` e testes/exemplos de arte.
- Agente 3: `src/simulation/**` e testes/exemplos de domínio.
- Agente 4: `src/metrics/**`, `src/diagnostics.rs` e material de benchmark, com eventual proposta de mudanças em scripts.
- Agente 5: `src/app.rs`, `src/config.rs`, `src/main.rs` e testes de UX/configuração.

`src/lib.rs`, `Cargo.toml`, `Cargo.lock`, `scripts/build-windows.ps1` e `README.md` são arquivos compartilhados: **não os alterar sem necessidade**. Se a modificação for inevitável, produza um patch restrito e registre o potencial conflito. Não reformatar/reescrever arquivos fora da sua frente.

## Contratos do baseline a preservar

- Plataforma: `platform::dock_window(&eframe::Frame) -> Result<(), String>` e `platform::show_error(&str)`.
- Renderização: `SceneRenderer::new()` / `SceneRenderer::render(&Train, &World) -> &[Color32]` ou adaptador com mesma assinatura.
- Simulação: `Train::new(&Config)`, `Train::update(cpu: f32, dt: f64, &Config) -> f64`, `World::default()`, `World::update(dt: f64, distance: f64, speed: f64)` e getters existentes.
- Métricas: `MetricsWorker::spawn(Duration) -> io::Result<Self>`, `poll() -> Option<MetricsSample>`, `shutdown()`, `MetricStatus` e campos de `MetricsSample`.
- UX: `Config` compatível com JSON antigo via `#[serde(default)]`, `LaunchOptions` e argumentos CLI anteriores continuam funcionando.

## Definição de pronto

Entregar um ZIP com código compilável e **funcionalidade realizada**, não somente um plano. Juntar, na raiz, `PATCH_NOTES.md` (o que mudou), `INTEGRACAO.md` (como conectar aos outros), `TESTES.md` (comandos/resultados/limitações), além de testes automatizados e, quando houver uma execução visual verdadeira, prova visual. Não incluir `target/`, caches ou binários pesados no source ZIP.

Checks preferenciais (na plataforma disponível):

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

Se estiver no Linux, usar `cargo build --release --features host-validation` para a interface de desenvolvimento somente quando pertinente. **Não alegar Windows validado por compilação cruzada ou Wine.** Registrar explicitamente o que não foi executado. Informar arquivos alterados e qualquer quebra de compatibilidade.

## Protocolo de entrega

A entrega precisa ter: resumo de 10 linhas, lista de arquivos alterados, API pública nova, demonstração (comando para testar), 3 riscos reais e como fazer rollback. Se uma tarefa for impossível sem tocar outra frente, implemente a parte isolada e detalhe o adaptador mínimo a ser aplicado pelo coordenador. Não solucionar o conflito copiando ou refatorando a árvore de outro agente.
