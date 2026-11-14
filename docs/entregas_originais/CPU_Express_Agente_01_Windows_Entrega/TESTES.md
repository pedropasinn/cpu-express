# Verificação — Agente 01 / Windows

Data: **13/11/2026**. Ambiente de execução: Linux, Rust/Cargo **1.100.0**, alvo
cruzado **x86_64-pc-windows-gnu**, MinGW-w64. O lockfile foi mantido.

**Os executáveis Windows foram compilados e inspecionados, mas não executados
em Windows real.** O CI MSVC foi escrito, não executado. Nenhuma imagem antiga
do pacote é apresentada como prova visual desta alteração.

## Resultados obtidos

| Verificação | Resultado nesta entrega | Evidência |
| --- | --- | --- |
| `cargo fmt --check` | Aprovado | `validation/windows-agent/fmt.log` (vazio significa sem diferenças). |
| Clippy Linux, todos os alvos, warnings como erro | Aprovado | `clippy-linux.log`. |
| `cargo test --locked --offline` no Linux | **45 passaram; 0 falharam** | `test-linux.log`. |
| Clippy Windows GNU, todos os alvos, warnings como erro | Aprovado, incluindo código Win32 e demonstração | `clippy-windows-gnu.log`. |
| Build release Windows GNU do jogo e da demonstração | Aprovado; dois arquivos PE x64 GUI | `build-windows-gnu.log`, `pe-summary.json` e dumps PE. |
| Aplicabilidade do patch de UX ao baseline | `git apply --check` aprovado | `integration-audit.json`. |
| Clippy Windows do projeto com patch de UX aplicado | Aprovado em cópia separada, sem mudar o app entregue | `clippy-integration-windows-gnu.log`. |
| Comparação com os arquivos originais | Apenas Cargo.toml e os dois arquivos platform existentes foram editados; **42 arquivos originais preservados** | `source-audit.json`. |
| Avisos dos binários GNU | 143 pacotes; 326 textos com hashes conferidos | `LICENSE-INVENTORY.json` no ZIP de binários. |
| Workflow MSVC | YAML e configuração inspecionados; job não executado | `.github/workflows/windows.yml`, `docs/windows_ci.md`. |
| GUI, bandeja, foco, monitores, DPI e cleanup em Windows | **Pendente** | `docs/windows_checklist.md`. |

Os nomes abreviados dos logs acima ficam em `validation/windows-agent/`.
Dependências já compiladas foram reutilizadas do cache local; não se trata de
um ensaio de build limpo nem de medição de tempo de compilação. O código do
CPU Express foi recompilado na árvore desta entrega, e o patch foi conferido
com um diretório de artefatos separado para evitar reutilizar o app sem patch.

## Os 45 testes executados

| Grupo | Testes |
| --- | ---: |
| Configuração original | 8 |
| Métricas originais | 5 |
| Simulação original | 7 |
| Posicionamento original | 7 |
| Nova geometria de plataforma | 12 |
| Política de estados e recuperação | 6 |
| **Total** | **45** |

Os 27 testes originais continuam passando. A matriz nova combina quatro
monitores/origens, quatro escalas de DPI (100/125/150/200%) e quatro bordas.
Inclui área de 1 × 1 pixel, margens excessivas/negativas, origem extrema de
`i32`, DPI extremo, áreas inválidas/disjuntas, múltiplas reservas, auto-hide
sem reserva e recuperação da posição arrastada sem resnap.

O módulo `tray.rs` contém ainda **três testes puros** de protocolo versão 4 e
coordenadas do menu. Como esse módulo está sob `cfg(windows)`, eles foram
checados pelo compilador no alvo Windows, **não executados neste host**. O
workflow Windows os executará junto com a suíte quando for acionado no
repositório.

## Comandos realmente utilizados

Na raiz do projeto, com a toolchain no PATH:

```bash
cargo fmt --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline

# Com o target Rust Windows GNU e o linker MinGW configurados:
cargo clippy --locked --offline --all-targets \
  --target x86_64-pc-windows-gnu -- -D warnings
cargo build --release --locked --offline \
  --target x86_64-pc-windows-gnu \
  --bin cpu-express --example windows_desktop

git apply --check integration/windows-ux.patch

# Em cópia separada da fonte com o patch aplicado:
cargo clippy --manifest-path ../integration-draft/Cargo.toml \
  --locked --offline --all-targets \
  --target x86_64-pc-windows-gnu -- -D warnings
```

O host usou `CARGO_BUILD_JOBS=2` e diretórios de artefatos separados para testes
Linux, alvo Windows e cópia de integração. `--offline` foi possível porque a
toolchain e as dependências já estavam disponíveis. No computador do usuário,
remova essa opção se o cache ainda não estiver preenchido.

A primeira passagem de Clippy Windows encontrou quatro empréstimos mutáveis
desnecessários nas chamadas `Shell_NotifyIconW`. Eles foram corrigidos; o log
`clippy-windows-gnu-initial.log` registra essa passagem histórica, e o log sem
`initial` é o resultado final aprovado. A revisão independente também motivou
o fallback de sinalização de parada da bandeja, a notificação de fechamento
inesperado do helper e a preservação do objeto para tentar cleanup novamente.
Essas correções não são apresentadas como testes de falhas Win32 executados.

## Binários e identidade

Os hashes e tamanhos dos **novos** `cpu-express.exe` e `windows_desktop.exe`
estão em `BUILD-INFO-WINDOWS-AGENT.json`. No ZIP de binários,
`SHA256SUMS.txt` permite conferir os dois arquivos. O `BUILD-INFO.json` original
continua sendo apenas o registro histórico do baseline.

`file` e `objdump -p` confirmaram PE32+ x86-64 e subsistema GUI. Os imports
estáticos inspecionados não incluem DLL separada de runtime MinGW como
`libgcc_s_seh-1.dll` ou `libwinpthread-1.dll`. Isso não prova compatibilidade
gráfica nem funcionamento no Windows de destino; o jogo continua exigindo
as APIs Windows indicadas pelo baseline e um driver OpenGL compatível.

## Como concluir o aceite no Windows

1. Extraia o ZIP de binários em uma pasta, mantendo avisos e arquivos juntos.
2. Abra `cpu-express.exe` para conferir o trem e o posicionamento atual.
3. Abra `windows_desktop.exe` para testar a bandeja e os estados da janela.
4. Execute as matrizes de `docs/windows_checklist.md`: dois monitores, DPI
   distintos, coordenadas negativas, auto-hide e reinício do Explorer.
5. Registre pass/fail, versão do Windows, monitores, escalas e evidência real.

O teste temporizado da demonstração pode ser iniciado no PowerShell assim:

```powershell
$demo = Start-Process .\windows_desktop.exe -ArgumentList '--smoke-seconds', '15' -PassThru
$demo.WaitForExit()
$demo.ExitCode
```

Isso verifica abertura/encerramento somente se for executado de fato. Não gera
carga de CPU artificial. O menu `Sair`, o botão `Sair` e `Esc` devem encerrar
o processo; ocultar pela bandeja é uma ação distinta.

## O que não foi medido ou executado

- Não foram executados os binários no Windows, em Wine ou em PowerShell.
- Não foi acionado o workflow no GitHub; não há URL de execução de CI.
- Não houve benchmark novo de CPU, memória, FPS ou contagem de handles.
- Não houve inspeção visual nova do trem ou da bandeja; a imagem antiga é
  histórica e não valida os novos caminhos Win32.
- Não foi testada em execução uma fila Win32 cheia, falha de shell ou menu
  modal durante shutdown. Os casos estão descritos para validação instrumentada.
