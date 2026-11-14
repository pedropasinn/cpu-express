# CPU Express 0.1 — relatório de validação

**Data: 13/11/2026.**

**Resultado: o executável Windows x64 foi gerado; o fluxo completo foi executado em uma janela nativa no laboratório Linux. A execução em Windows 10/11 real permanece pendente.** Compilação, testes de lógica, inspeção visual no host e integração efetiva com o desktop Windows estão separados neste relatório.

O código de execução validado corresponde ao commit **`48cb9d9`**, que introduziu a textura nativa com saída visual idêntica. O commit `475c78c` acrescentou scripts de distribuição e avisos de terceiros; alterações posteriores de documentação/empacotamento não representam novos testes de runtime.

## 1. Ambiente e artefato

| Item | Evidência |
|---|---|
| Ambiente de desenvolvimento e execução gráfica | Linux x86-64; Xorg com tela virtual; Mesa por software; Openbox no ensaio de minimização |
| Rust | 1.100.0 estável; projeto em edição 2024 |
| Bibliotecas principais | eframe/egui 0.37.0; sysinfo 0.40.3; versões resolvidas preservadas em Cargo.lock |
| Alvo Windows gerado | `x86_64-pc-windows-gnu` |
| Cross-toolchain | GCC MinGW 14, binutils 2.44, mingw-w64 12; ferramentas extraídas em diretório de trabalho, sem instalação global |
| Formato do executável | PE32+, x86-64, subsistema Windows GUI |
| Tamanho do executável release | **5.287.936 bytes**, aproximadamente **5,04 MiB** |
| SHA-256 do executável release | `759322f458eb223f163884b4d1c14450dd6bdc60a035edd4d906752a3aafa24f` |
| Windows mínimo declarado | Windows 10 versão 1607 ou posterior; Windows 11; arquitetura x64 |

O tamanho e o hash acima foram lidos do arquivo final, não estimados. A inspeção da tabela de imports encontrou DLLs do Windows e OpenGL, sem dependência externa de libgcc/libwinpthread nem runtime WebView2. Isso descreve os imports do arquivo, sem comprovar que o driver gráfico de um computador Windows específico conseguirá iniciá-lo.

O executável final não é o build debug da janela mínima. O release Windows foi compilado após a integração e a otimização da cena; seu registro está em [windows-release-final.log](../validation/windows-release-final.log).

## 2. Comandos e resultados

| Verificação | Onde aconteceu | Resultado |
|---|---|---|
| `cargo fmt --check` | Host Linux | **Passou** |
| `cargo clippy --all-targets -- -D warnings` | Host Linux | **Passou** |
| `cargo test` | Host Linux | **27/27 testes passaram** |
| `cargo build --release --features host-validation` | Host Linux | **Passou**; usado nos ensaios gráficos do laboratório |
| `cargo build --release --target x86_64-pc-windows-gnu` | Cross-compilação no host Linux | **Passou**; produziu o `.exe` identificado acima |
| `cargo clippy --target x86_64-pc-windows-gnu --all-targets -- -D warnings` | Análise do alvo Windows no host Linux | **Passou** |
| `cargo test --target x86_64-pc-windows-gnu --no-run` | Cross-compilação no host Linux | **Passou**; gerou seis executáveis de teste, sem executá-los |
| Executar `.exe` e testes em Windows real | Não disponível neste ambiente | **Pendente** |
| Build e execução com MSVC | Não executados nesta entrega | **Pendente** |

Os seis executáveis de teste Windows correspondem aos alvos unitários de `src/lib.rs` e `src/main.rs`, mais as quatro suítes de integração. O comando `--no-run` comprova compilação e link desses alvos. **Não significa que os 27 testes foram executados no Windows.**

Registros do alvo Windows: [windows-final-clippy.log](../validation/windows-final-clippy.log) e [windows-tests-compiled.log](../validation/windows-tests-compiled.log). A feature `host-validation` permite inspeção gráfica durante o desenvolvimento; não fornece a integração de desktop Linux reservada para a versão 0.5.

## 3. O que os 27 testes verificam

| Suíte | Quantidade | Comportamentos verificados |
|---|---:|---|
| Configuração | 8 | Defaults sem escrita implícita; campos parciais; normalização; salvar/substituir arquivo; preservar configuração inválida; erro de diretório; limpeza somente do temporário próprio quando a troca final falha |
| Métricas | 5 | Provedor manual sem RAM inventada; primeira leitura real de CPU não marcada como válida; intervalo inicial; desligamento do worker adormecido; fila cheia sem bloqueio; publicação de indisponibilidade |
| Simulação | 7 | Mapeamento 0/50/100%; monotonicidade; solução analítica da velocidade/distância; independência de FPS; entradas não finitas; normalização; cenário e fumaça limitados; suspensão sem criação ilimitada de partículas |
| Posicionamento | 7 | Área reservada nas quatro bordas; origens negativas; DPI misto em pixels físicos; janela maior que área útil; fallback sem reserva; entradas inválidas; coordenadas extremas sem overflow |
| **Total** | **27** | **Todos passaram no host Linux** |

A verificação de uma hora de simulação avança um relógio simulado para conferir limites do estado; não representa uma hora de aplicação gráfica em execução. Os testes de geometria exercitam a função pura de posicionamento. As chamadas Win32 foram compiladas e analisadas pelo Clippy, mas não executadas neste host.

## 4. Janela mínima antes do jogo

A Etapa 1 criou uma janela eframe com tamanho inicial de 640 × 140, fundo, título “CPU Express”, botão de saída e fechamento de teste após aproximadamente três segundos. A janela foi aberta no laboratório gráfico, teve imagem capturada e encerrou com **código 0**.

A mesma etapa foi compilada para Windows GNU e identificada como PE32+ x86-64 com subsistema GUI. Isso estabeleceu a base de janela e de cross-compilação antes da implementação do trem. A execução observada foi no host Linux; a imagem está em [stage1-window.png](../validation/stage1-window.png).

## 5. Prova do fluxo CPU real → velocidade → cena

A prova de conceito completa permaneceu aberta por aproximadamente seis segundos, com sysinfo real e sem carga artificial de CPU. O diagnóstico [poc-real/metrics.jsonl](../validation/poc-real/metrics.jsonl) registrou:

- Primeira amostra com `cpu_percent: null` e estado `warming_up`.
- Depois do aquecimento, **seis registros com CPU válida**, entre aproximadamente **6,21% e 18,44%**, além de RAM real em percentual e bytes.
- Todos os registros compatíveis com a fórmula `target_kmh = 12 + effective_cpu_percent × 0.88`.
- Velocidade atual distinta do alvo durante aceleração/frenagem, conforme a suavização.
- **161 frames renderizados**, cenário em movimento e fumaça limitada.
- Encerramento normal com **código 0**.

O registro final contém CPU real de aproximadamente 6,98%, alvo de 18,14 km/h e velocidade atual de 18,74 km/h: o trem ainda estava desacelerando em direção ao alvo. Essa diferença é comportamento esperado da suavização.

O arquivo [poc-real/result.json](../validation/poc-real/result.json) e as capturas [first.png](../validation/poc-real/first.png) e [last.png](../validation/poc-real/last.png) documentam a execução. Esse ensaio comprova o encadeamento da coleta real ao rendering no host. A implementação sysinfo para Windows foi compilada, mas suas leituras naquele sistema ainda precisam ser observadas.

## 6. Controles e persistência na aplicação em execução

O ensaio de interface usou eventos de teclado e mouse sobre a janela nativa, com captura visual e diagnóstico. Foram observados:

| Ação | Resultado observado |
|---|---|
| Atalho `M` | Métricas ocultadas; `show_metrics: false` persistido |
| Atalho `P` | Opção de sempre no topo desativada; `always_on_top: false` persistido |
| Abrir controles | Painel visível, sem recorte observado na janela de teste |
| Marcar modo manual e mover slider até 100% | Diagnóstico mudou para `mode: manual`, CPU efetiva 100% e alvo 100 km/h |
| `Escape` com o painel aberto | Painel fechado |
| Botão `×` | Aplicação encerrou normalmente, com código 0 |

A configuração gravada está em [controls-actions/config.json](../validation/controls-actions/config.json), e os valores de modo/velocidade em [controls-actions/metrics.jsonl](../validation/controls-actions/metrics.jsonl). No encerramento desse ensaio, a velocidade era aproximadamente 95,33 km/h, aproximando-se do alvo de 100 km/h.

O slider altera somente a entrada da simulação: **não aumenta a utilização real do processador**. O toggle de sempre no topo e sua persistência foram exercitados; o empilhamento efetivo e a preservação do foco em Windows real continuam no checklist.

## 7. Minimização, coleta independente e retomada

O ensaio release de doze segundos foi realizado com Xorg e **Openbox**, para que minimizar/restaurar fossem operações reais de um gerenciador de janelas. Os registros de [release-minimized/metrics.jsonl](../validation/release-minimized/metrics.jsonl) mostram:

| Momento aproximado | Estado | Frames acumulados | Amostras recebidas |
|---:|---|---:|---:|
| 2,95 s | Oculta/minimizada | 54 | 3 |
| 3,95 s | Oculta/minimizada | 54 | 4 |
| 4,96 s | Oculta/minimizada | 54 | 5 |
| 5,96 s | Oculta/minimizada | 54 | 6 |
| 6,96 s | Oculta/minimizada | 54 | 7 |
| 8,00 s | Restaurada | 82 | 8 |
| 12,03 s | Encerramento | 191 | 13 |

Durante o intervalo oculto, o contador de frames permaneceu em **54**, enquanto a coleta continuou avançando de **3 para 7 amostras**. Depois da restauração, o rendering voltou a avançar, chegando a 191 frames. O processo encerrou com **código 0**. O resultado completo está em [release-minimized/result.json](../validation/release-minimized/result.json).

Após a otimização final, o ensaio foi repetido durante **90 segundos**, com aproximadamente 60 segundos de minimização. No trecho estável entre 20 e 70 segundos, o contador de frames permaneceu em **435**, enquanto as amostras recebidas passaram de **21 para 70**. A restauração retomou a animação e o processo encerrou normalmente com **880 frames e 90 amostras**. A execução visível final de **300 segundos** também encerrou com código 0, **8.747 frames e 300 amostras**. Os dados e as medições de consumo estão em [BENCHMARKS.md](BENCHMARKS.md).

Uma tentativa anterior com `XUnmapWindow` sem gerenciador de janelas não representava uma minimização convencional gerenciada e não foi aceita como evidência desse requisito. Outra tentativa com xfwm4 não produziu um ambiente de teste utilizável. O procedimento foi corrigido usando Openbox. São limitações/ajustes do ensaio Linux, não testes Windows nem demonstrações de falha do produto no Windows.

## 8. Otimização sem alteração da imagem de referência

A textura da cena passou a usar a grade original de **320 × 70 pixels**, ampliada com filtragem nearest para os **640 × 140** exibidos. Um buffer RGBA dessa grade ocupa **89.600 bytes**; esse número descreve o buffer de pixels, não a memória total da aplicação ou do driver.

O PPM exportado antes e depois da alteração, com o mesmo estado de simulação, foi comparado byte a byte e permaneceu idêntico. Ambos os arquivos de 640 × 140 têm SHA-256:

```text
bc63c0452f0a3e61c43485f2fc4a7a3ccc308885571dc5e6e2ad8f6107698da8
```

Essa verificação comprova a preservação da imagem de referência. As medições de consumo da versão otimizada são tratadas separadamente em [BENCHMARKS.md](BENCHMARKS.md); a redução da quantidade de pixels não é apresentada como um benchmark de CPU ou RAM.

## 9. Limites da evidência disponível

**Não houve execução em Windows real.** Wine 10 foi obtido no laboratório, mas abortou durante sua própria inicialização, antes de executar o programa mínimo, com `free(): invalid pointer`. A causa desse abort não foi estabelecida. Wine não foi usado como evidência positiva de funcionamento.

Permanecem para validação no Windows: abertura no driver OpenGL real; leitura de CPU/RAM pelo provedor Windows; arraste entre monitores; escalas diferentes; área útil e posição da taskbar; always-on-top sem interrupção de foco; comportamento de minimizar/restaurar; e consumo do release naquele sistema.

Os scripts [build-windows.ps1](../scripts/build-windows.ps1) e [measure-windows.ps1](../scripts/measure-windows.ps1) foram revisados por leitura do código. **Não foram executados em PowerShell/Windows nesta entrega.** O primeiro compila e testa no Windows com dependências travadas; o segundo mede o processo sem gerar carga artificial. A existência desses scripts não substitui seus resultados de execução.

Os ensaios de consumo e estabilidade estão em [BENCHMARKS.md](BENCHMARKS.md), com ambiente, duração e método próprios. Testes curtos de UI, uma hora de relógio simulado e um executável compilado não demonstram, por si, ausência de vazamento em execução prolongada nem baixo consumo em qualquer computador.

## 10. Checklist curto para Windows 10/11

Registrar versão/build do Windows, CPU, GPU/driver, quantidade de monitores e escala de cada um. Os passos abaixo usam a carga normal do computador, sem testes artificiais de stress.

1. **Abrir e fechar.** Abrir `cpu-express.exe`, verificar a janela compacta e o trem animado, fechar por `×`, reabrir e testar `Alt+F4`. Conferir se não ficou processo residual.
2. **Observar CPU/RAM reais.** Aguardar o aquecimento e verificar atualização dos indicadores durante o uso normal do computador. Confirmar que a animação continua respondendo enquanto os valores mudam.
3. **Testar os três alvos.** No modo manual, selecionar 0%, 50% e 100%; aguardar alguns segundos em cada valor e observar aproximação suave de 12, 56 e 100 km/h. Voltar ao modo real.
4. **Conferir controles e foco.** Alternar métricas e sempre no topo; fechar/reabrir para verificar persistência. Digitar em outro aplicativo para confirmar que o trem não toma o foco periodicamente.
5. **Arrastar e reposicionar.** Mover a janela e usar a opção de reposicionar. Se houver vários monitores, repetir com escalas diferentes e monitor à esquerda do principal. Confirmar a janela inteira na área útil, respeitando a posição da barra disponível nessa versão/configuração do Windows.
6. **Minimizar e restaurar.** Deixar minimizada por cerca de 30 segundos, restaurar e observar a retomada sem grande salto. Para registrar consumo, executar separadamente o script de medição com a aplicação visível e outro ensaio minimizado, seguindo o README.

Somente depois desses ensaios é possível registrar aprovação da integração no computador Windows utilizado. O pacote entregue permanece uma versão inicial com cross-compilação e testes de host documentados.
