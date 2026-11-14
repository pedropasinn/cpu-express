# CPU Express 0.3 — validação e alcance das evidências

Consolidação documental em 13/11/2026 para a base 0.3.0-rc.1. O contrato funcional está em [VERSAO_03.md](VERSAO_03.md). Este relatório descreve os resultados locais disponíveis em `evidence/current/`, relativos às execuções registradas, sem fixar hashes de binários.

O executavel final e identificado em BUILD-INFO.json. O empacotamento exige que os testes nativos e de metricas reais correspondam ao SHA-256 desse executavel. A suite foi repetida apos as correcoes da revisao tecnica.

## Gates automatizados

Fonte: [checks.json](../evidence/current/checks.json) e logs associados. O script `scripts/verify-current.ps1 -Jobs 2` executa os comandos abaixo no Windows.

| Verificação | Comando Cargo | Resultado | Tempo registrado |
| --- | --- | --- | --- |
| Formatação | `cargo fmt --all -- --check` | saída 0 | 0.54 s |
| Clippy | `cargo clippy --locked --all-targets -j 4 -- -D warnings` | saída 0 | 4.61 s |
| Testes | `cargo test --locked --all-targets -j 4` | saída 0 | 51.50 s |

Logs: [format.log](../evidence/current/format.log), [clippy.log](../evidence/current/clippy.log) e [tests.log](../evidence/current/tests.log). A soma dos resultados Rust é **178 aprovados, zero falhas e dois ignorados**. Os ignorados são exportadores opcionais de interface, não testes funcionais que falharam. Os logs registram execução de testes Windows, não apenas linkedição de executáveis. O invólucro `NativeCommandError` do PowerShell em saídas de stderr não substitui os códigos finais de saída registrados em `checks.json`.

A suíte cobre configuração, expedição, invariantes e rotas da jornada, métricas e worker, geometria e ciclo de vida da plataforma, renderização, simulação e interface/CLI. Isso não substitui a aceitação interativa de todos os comportamentos Windows.

Python: **19 + 10 + 23 aprovados, total 52**, conforme resultado de execução informado pelo integrador nesta entrega. Os três gates de `checks.json` são Cargo; não contém esses resultados Python. Este relatório não atribui os números Python aos logs históricos da consolidação anterior.

## Execução nativa da faixa e do painel

Fonte: [native-smoke-result.json](../evidence/current/native-smoke-result.json), com registro detalhado em [native-smoke.jsonl](../evidence/current/native-20261113-213124/native-smoke.jsonl).

- Saída 0, sem timeout, 13 registros, evento final `shutdown` e 350 frames renderizados.
- `taskbar_child_verified=true`: faixa integrada (`embedded=true`), parent `Shell_TrayWnd`, retângulo de 274 × 60 em (0, 1020); a barra medida também tinha 60 pixels de altura.
- `independent_panel_verified=true`: painel nativo visível, retângulo de 593 × 697 em (10, 320), independente da faixa.
- `strip_capture=true`: [faixa.png](../evidence/current/native-20261113-213124/faixa.png) é captura do **framebuffer nativo do aplicativo**, não captura do desktop/compositor.
- `panel_capture=false`: **não houve captura do compositor do painel**. A validação nativa do painel é de HWND/retângulo e visibilidade; a evidência de conteúdo das abas vem da exportação headless descrita a seguir.

Essas dimensões descrevem o ambiente ensaiado, não uma garantia para todas as configurações de DPI, barra ou monitores.

## Conteúdo das quatro abas

Em [evidence/current/panels/](../evidence/current/panels/), há pares JSON/PNG para as quatro abas:

| Aba | Exportação |
| --- | --- |
| Trem | [floating_trem.png](../evidence/current/panels/floating_trem.png) |
| Viagem | [floating_viagem.png](../evidence/current/panels/floating_viagem.png) |
| Visual | [floating_visual.png](../evidence/current/panels/floating_visual.png) |
| Janela | [floating_janela.png](../evidence/current/panels/floating_janela.png) |

São renderizações **headless do egui**, acompanhadas pelos JSON de mesmo nome. Servem à inspeção do conteúdo/layout e complementam a verificação nativa por HWND/retângulo. Não demonstram composição real da janela, foco, interação, oclusão ou DPI do painel no desktop. Os glifos foram corrigidos e a exportacao da aba Viagem foi inspecionada visualmente.

## Métricas reais

Fontes: [real-metrics-result.json](../evidence/current/real-metrics-result.json) e [real-metrics-final.jsonl](../evidence/current/real-metrics-final.jsonl).

A execução terminou com saída 0 e **nove amostras reais válidas**. No registro final, `mode=real`, `metric_status=ready`, `metric_freshness=current` e `metrics_stale=false`. Foram recebidas dez amostras; a amostra final tinha aproximadamente 0,36 s de idade. A execução durou 10,05 s e renderizou 289 frames.

No encerramento, CPU era 13,97% e RAM 60,90%; são leituras daquele instante, não benchmarks de consumo do aplicativo. A viagem acumulou 10,009 s ativos e 240,217 unidades, coerentes com 24 unidades por segundo ativo. Não houve chegada a estação nesse intervalo. Esse smoke curto não comprova sozinho salvamento/reabertura, objetivos de longa duração ou estabilidade prolongada.

## Comportamento e pesquisa da versão

Conforme [VERSAO_03.md](VERSAO_03.md), a jornada e os desbloqueios persistem em checkpoint separado das preferências; o diário de 32 mensagens continua sendo da sessão. Pausa, minimização e ocultação não concedem tempo offline. DEMO e encerramento automático não carregam nem gravam a jornada normal. Portanto, os smokes com encerramento automático não constituem teste de persistência do perfil normal.

A faixa principal é filha da barra horizontal primária, alinhada à esquerda; há modo flutuante alternativo. Ela não reserva espaço no layout do Explorer, podendo disputar a área com widgets/botões. A descrição antiga de janela apenas acima/próxima à barra não representa o modo principal da 0.3.

A pesquisa consolidou **96 achados**, provenientes de A1, A2, A3, A4 e A6. Os documentos e índices estão em `pesquisa/` na raiz do projeto, incluindo `achados-consolidados.json` e `inventario-entradas.json`. São evidências estáticas de pesquisa, não código-fonte recuperado do gameplay nem resultados de execução deste aplicativo. Arte, plataforma, passageiros e pinturas implementados são procedurais e autorais.

## Limites e fechamento do build

Há evidência local de execução real no Windows. Permanecem fora do alcance desse smoke curto: matriz de DPI 100/125/150/200% e DPI misto, autohide, reinício do Explorer, múltiplos monitores, conflitos com widgets e aceitação prolongada. Barras verticais, monitores secundários e substitutos do Explorer não são variantes de integração oferecidas nesta versão.

O empacotador verifica a correspondencia entre o executavel e os hashes registrados nos testes nativos. Os metadados incluem hashes dos fontes de producao e o manifesto identifica os arquivos distribuidos.

Os documentos anteriores de entrada rápida foram copiados integralmente para `docs/historico/LEIA-ME_RC1.txt`, `INTEGRACAO_RC1.md`, `PATCH_NOTES_RC1.md` e `TESTES_RC1.md`, com igualdade de hash conferida antes da atualização. Eles descrevem a consolidação 0.2.0-rc.1. `docs/VALIDATION.md` e `docs/evidencias_consolidacao/` pertencem ao histórico; a validação corrente é este documento e as evidências atuais explicitamente identificadas acima.
