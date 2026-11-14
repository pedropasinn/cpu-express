# Agente 05 — validação executada e limites

**Data: 13/11/2026. Resultado: 63 testes aprovados, zero falhas, mais uma exportação visual opcional executada separadamente.** Os 27 testes originais da base estão incluídos e seus arquivos não foram alterados.

Esta é a validação desta frente de UX. `docs/VALIDATION.md`, `BUILD-INFO.json` e `validation/` recebidos no pacote são históricos da base, não evidência da versão modificada.

## 1. Checks finais sobre os mesmos arquivos de código

| Comando | Resultado observado | O que comprova |
| --- | --- | --- |
| `cargo fmt --check` | Exit 0 | Formatação de todo o projeto. |
| `cargo test --offline --locked` | **63 passaram; 0 falharam; 1 ignorado** | Testes unitários e de integração com as features padrão. O ignorado é somente o exportador visual opcional. |
| `cargo clippy --offline --locked --all-targets --features host-validation -- -D warnings` | Exit 0, sem warnings | Lint de todos os alvos no host de desenvolvimento. |
| `cargo check --offline --locked --all-targets --target x86_64-pc-windows-gnu` | Exit 0 | Compilação de verificação/tipos do código e dos testes para Windows GNU. **Não gera nem executa um `.exe`.** |
| `cargo build --offline --locked --release --features host-validation` | Exit 0 | Build otimizado do executável de investigação Linux com o código final. **Não é release Windows nem port Linux de produto.** |
| Exportador `tests/ux_visual.rs`, executado com `--ignored` e diretório explícito | 1 passou | A aplicação real produziu malhas/texturas do egui para a inspeção visual headless. |
| `git apply --check AGENTE05_UX.patch`, na base intacta | Exit 0 | Patch de produção/testes aplicável à linha A fornecida. |

Os logs finais estão em `docs/ux_evidencias/logs/`. `final-gates.json` registra os comandos, códigos de saída e hashes dos arquivos Rust no início da rodada; a verificação confirmou que esses arquivos não mudaram durante a execução dos checks.

Não foram acrescentadas dependências nem atualizadas versões do lockfile. `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/lib.rs`, exemplos e scripts permanecem iguais aos recebidos.

## 2. Ambiente real usado

- Host: Linux x86_64; `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- Targets instalados utilizados: Linux nativo e `x86_64-pc-windows-gnu` para `cargo check`.
- Execuções Cargo com dependências em cache, `--locked` e `--offline`.
- `CARGO_BUILD_JOBS=1`; debug e testes com `CARGO_PROFILE_DEV_DEBUG=0` e `CARGO_PROFILE_TEST_DEBUG=0`. São ajustes de execução, sem alteração no manifesto. O release utilizou o perfil original, com otimização e LTO.
- O executável de release Linux foi compilado. Não foi incluído no ZIP-fonte.
- Não havia uma sessão Windows real nem o linker/SDK necessário para produzir um novo executável Windows nesta entrega.

A primeira tentativa de compilar a base sem limitar o paralelismo terminou com `SIGKILL` em uma dependência. A execução com um job passou os 27 testes originais; os checks finais adotaram esse limite. Isso foi uma restrição de recursos do ambiente, não um teste marcado como aprovado após falhar.

## 3. Comportamentos cobertos

### Configuração e preservação de dados

Os oito testes de configuração originais continuam passando. Os nove novos verificam campos antigos/parciais, defaults e modos novos, precedência de `show_metrics`, limites, round-trip, preservação de campos desconhecidos e recusa de sobrescrever conteúdo corrompido ou incompatível. A carga inicial não regrava o arquivo.

A CLI testa também a colisão acidental entre o destino de diagnóstico e a configuração: caminhos iguais, relativos/absolutos, componentes `..` e links simbólicos no host. A abertura de diagnóstico é recusada antes que o arquivo de configuração seja truncado. O teste específico de variação de caixa Windows foi compilado na checagem de target, mas só será executado em Windows.

### DEMO e argumentos

O parser cobre os argumentos anteriores, limites numéricos, valores não finitos, caminhos nativos, erros em português, ajuda sem efeitos colaterais e ordem das flags. `--demo` usa 50% quando não há `--cpu` explícito. Um teste da fábrica de métricas faz a função de criação do provider real entrar em `panic!` caso seja chamada pelo demo; a execução passa, recebe a amostra manual e confirma ausência de RAM.

Os testes de UI instanciam `CpuExpressApp::new_demo` e verificam o estado e os textos da aplicação real, inclusive o marcador DEMO no HUD limpo.

### Pausa, teclado, foco e mouse

Os 11 testes de `tests/ux_app.rs` verificam:

- Descarte de uma pausa longa mesmo sem frames intermediários e limite para um frame excepcionalmente longo.
- Congelamento de rodas, paisagem, tempo de mundo e partículas durante pausa.
- Atalhos com modificadores e ausência de comandos indevidos durante edição.
- Escape fechando somente painéis, sem emitir fechamento da aplicação.
- Estado inicial pausado na aplicação real.
- Arraste na paisagem emitindo o comando de movimento; botões e painel não disparam esse movimento.
- Editor numérico de CPU recebendo Espaço e M sem acionar pausa ou HUD.
- Tab começando em um controle visível e trazendo para a área de rolagem um controle inicialmente escondido.

Essas interações usam `egui::Context::run_ui` e `eframe::Frame::_new_kittest`. Elas exercitam os widgets e a lógica reais, mas não equivalem ao gerenciamento de uma janela pelo Windows.

## 4. Evidências visuais

Sete PNGs em `docs/ux_evidencias/` mostram HUD compacto, completo, limpo, pausa, opções, ajuda e modo limpo sem foco. São **renderizações headless do egui, sem janela nativa**. O exportador usa o renderer original, os controles reais, as texturas e a geometria produzida pela aplicação; `docs/ux_rasterizar.py` converte essa saída em imagem por software.

Foram corrigidos após inspeção: velocidade coberta pelo painel, scrollbar invisível sem hover e dois separadores sem glifo disponível. Os quadros finais foram regenerados após as correções. A ajuda, os painéis e o HUD cabem nos 640×140 pixels do cenário de teste a escala 1.

As imagens não comprovam comportamento de driver, DPI, foco nativo ou composição de GPU. A tentativa de abrir uma janela Linux via Xorg foi bloqueada porque o ambiente negou criação de sockets Unix; essa restrição não foi contornada. Não há screenshot atual de Windows nesta entrega.

## 5. Checklist nativo Windows — não executado

Registrar versão do Windows, monitor, escala, driver gráfico e resultado de cada item antes de considerar a integração nativa validada.

| Conferência pendente | Resultado esperado |
| --- | --- |
| Abrir a aplicação em Windows 10/11 x64 | Janela compacta sem roubar o foco de outra aplicação. |
| Testar DPI 100%, 125%, 150% e monitores diferentes | Texto e botões utilizáveis, painel dentro da janela, reposicionamento dentro da área útil. |
| Arrastar, fixar, minimizar e restaurar | Sem conflito com controles; restauração pela taskbar. |
| Testar CPU real e perda/atraso de leitura | Indicadores verdadeiros; aviso recuperável e UI responsiva. |
| Usar DEMO e presets 0/50/100 | CPU identificada como simulada, RAM indisponível e nenhuma inicialização de contadores reais. |
| Pausar, aguardar, retomar e minimizar/restaurar | Sem compensar o tempo ausente por um salto da paisagem. |
| Navegar com Tab, editar CPU, abrir/fechar ajuda | Foco visível, editor sem atalhos indevidos, Escape sem encerrar. |
| Persistir preferências e reabrir | Campos mantidos; arquivo antigo/corrompido preservado quando não puder ser salvo. |
| Fechar por Sair/Ctrl+Q | Processo encerra e libera o worker. |
| Medir CPU/RAM e FPS no computador de destino | Registrar medições reais; esta frente não acrescenta alegações de consumo ou benchmarks. |

Comandos sugeridos para a máquina Windows, ainda a executar nela:

```powershell
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
.\target\release\cpu-express.exe --demo --cpu 50
.\target\release\cpu-express.exe --demo --cpu 0 --smoke-seconds 10
.\target\release\cpu-express.exe --config .\preferencias.json
```

Não existe novo `.exe` Windows no pacote entregue. O `BUILD-INFO.json` herdado referencia o binário da base e não deve ser associado à fonte modificada. Antes de empacotar/distribuir binários, repor as licenças e atribuições da origem conforme `CONTRATO_COMUM.md` e `INTEGRACAO.md`.
