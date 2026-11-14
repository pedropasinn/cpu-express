# Validação — CPU Express 0.2.0-rc.1

**Esta página descreve a consolidação de 13/11/2026.** As evidências antigas foram separadas em `historico/`; seus resultados não valem como medição do executável novo.

## Resultados executados

| Comando/verificação | Ambiente | Resultado |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Host Linux | Passou. |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Host Linux | Passou. |
| `cargo test --offline --locked --all-targets` | Host Linux | **169 aprovados, 0 falhas, 1 ignorado.** |
| Exportador `ux_visual`, solicitado explicitamente | Host Linux, sem janela nativa | Passou; **18 cenários** exportados e revisados. |
| `python -m unittest discover -s benchmarks -p 'test_*.py'` | Host Linux | **19 aprovados.** |
| `cargo clippy --offline --locked --all-targets --target x86_64-pc-windows-gnu -- -D warnings` | Compilação cruzada | Passou. |
| `cargo build --offline --locked --release --target x86_64-pc-windows-gnu --bin cpu-express` | Compilação cruzada, Rust 1.100.0/MinGW | Passou; executável PE32+ GUI x64. |
| `cargo test --no-run --offline --locked --target x86_64-pc-windows-gnu` | Compilação cruzada | **18 executáveis de teste linkados**; não executados. |
| Inventário de licenças | Conferência local | **144 pacotes / 328 textos**, tamanhos e SHA256 conferidos sem divergências. |

O teste ignorado da execução padrão é o exportador visual opt-in. Ele foi executado separadamente com diretório de saída explícito. A contagem de 169 é de casos de teste Rust, não de cenários manuais ou testes executados no Windows.

## O que a suíte cobre

Os testes incluem integração de velocidade/distância, estabilidade e limites do tempo ativo, pausa e retorno, rota e eventos, conservação das conquistas, filas limitadas, integração de biomas por trecho, continuidade do cenário, desenho clássico preservado, ausência de alocação no renderer, frescor e dados parciais de métricas, persistência de preferências e campos futuros, atalhos/edição, ausência de rasterização quando a UI observa minimização e política de posicionamento/ciclo de vida.

O último ajuste de Clippy apenas colapsou duas condições no encerramento Windows em uma cadeia equivalente. Depois dele, formatação, Clippy Windows, release e ligação dos testes Windows foram repetidos. A lógica Linux já aprovada permaneceu inalterada.

## Evidência visual

Os 18 cenários incluem HUD compacto/completo/limpo, pausa, quatro abas, conteúdo rolado, modo sem foco e janelas de 320 × 140. São imagens produzidas a partir das texturas e geometria do **app egui real**, rasterizadas sem janela. Não foram desenhadas como mockups nem geradas por IA.

Método e PNGs: `evidencias_consolidacao/METODO_VISUAL.md` e arquivos vizinhos. A galeria em `../assets/gallery/` também vem do renderer real, incluindo a floresta acrescentada. Essas imagens não comprovam apresentação em GPU, escala do Windows, foco ou funcionamento da bandeja.

## Executável e rastreabilidade

O binário ocupa **5.476.352 bytes (5,22 MiB)**. É um programa Windows x64 sem assinatura Authenticode. A inspeção das importações não identificou dependência externa de `libgcc`, `libstdc++` ou `libwinpthread`; há importações do Windows e OpenGL.

O SHA256 definitivo, o target, o compilador e o manifesto das fontes estão em `../BUILD-INFO.json` e `evidencias_consolidacao/logs/`. Os hashes das fontes de produção, Cargo.toml e Cargo.lock foram conferidos antes/depois do build final. O executável entregue foi produzido pela fonte consolidada.

## O que não foi executado

- Aplicativo ou testes em **Windows 10/11 real**.
- Execução do jogo ou testes em **Wine**. O ambiente bloqueou a criação dos sockets locais necessários ao display/Xvfb/wineserver. Ter Wine instalado não foi tratado como prova de execução.
- Script PowerShell de distribuição ou workflow GitHub Actions. Seus arquivos foram preparados/revisados; o host não ofereceu o caminho de execução necessário.
- Benchmark de consumo do jogo consolidado em Windows. Não reutilizamos números de CPU/RAM/FPS das versões anteriores.

**Aceitação pendente:** seguir `windows_checklist.md` no PC de destino para foco, bandeja, área útil, DPI, monitores, Explorer, restauração e consumo. Por isso a versão permanece `rc.1`.
