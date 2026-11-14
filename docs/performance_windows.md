# Medição reproduzível do CPU Express no Windows

## Estado desta entrega

Foram implementados o coletor de contexto `benchmarks/run-windows.ps1`, o
analisador `benchmarks/analyze.py` e 19 testes determinísticos do analisador.
Os testes Python passaram no ambiente Linux desta entrega. O PowerShell foi
revisado estaticamente, mas **não foi executado em Windows nesta rodada**.
Não há neste documento resultados de desempenho atribuídos a hardware Windows.

O wrapper usa `scripts/measure-windows.ps1` sem modificá-lo. O coletor já
existente consulta o processo aproximadamente uma vez por segundo e produz
`summary.json` e `process-samples.csv`. O wrapper acrescenta contexto de
hardware, driver, energia e cenário. A análise ocorre **depois** da execução,
sem acrescentar trabalho por frame ao jogo. Nenhum destes comandos gera carga
artificial na CPU para movimentar o trem.

Os números dos projetos anteriores vieram de ensaios diferentes em Linux com
Mesa e renderização por software. **Não constituem uma comparação A/B
controlada**, nem uma estimativa do desempenho desta alteração em Windows.

## O que é medido

| Grandeza | Fonte e cálculo | Unidade e limite da afirmação |
| --- | --- | --- |
| CPU do processo em escala de um processador lógico | `100 × ΔTotalProcessorTime / Δtempo` | `% de um processador lógico`; pode superar 100% quando várias threads trabalham. Não é uma leitura separada de cada núcleo. |
| CPU do processo normalizada pela máquina | Valor anterior dividido pelo número de processadores lógicos registrado | `% da contagem lógica total`; não dividir pelo número de núcleos físicos. Não representa capacidade energética ou desempenho homogêneo de núcleos diferentes. |
| Tempo de CPU no período observado | Soma dos incrementos de CPU dos intervalos completos selecionados | Segundos de CPU; distinto de segundos de relógio e de CPU global do computador. |
| Contador de CPU desde o início do processo | Último `process_cpu_seconds` observado | Limite observado até o último poll. O coletor original não mede o intervalo entre esse poll e o encerramento; `lifetime_cpu_total_complete=false`. |
| Memória residente do processo | `.WorkingSet64` | Bytes, incluindo páginas privadas e compartilhadas residentes; não somar processos para obter RAM física total sem tratar compartilhamento. |
| Memória privada do processo | `.PrivateMemorySize64` | Bytes de memória privada comprometida; não é outra medida de RAM física residente. |
| FPS efetivo de renderização | `Δrendered_frames / Δelapsed_seconds` no JSONL da aplicação | Rasterizações da cena por segundo. Não equivale a frames apresentados pelo compositor/monitor nem a tempo de GPU. |
| Frames em intervalos com ocultação observada | Deltas de frames quando `hidden=true` nos dois registros extremos | Evidência por extremos de intervalo. Transições intermediárias não registradas continuam possíveis. Não é prova de uma contagem exata de frames continuamente ocultos. |
| Condição energética | Plano ativo antes/depois; fonte AC/bateria e modo declarados | Contexto experimental; não mede watts, joules, temperatura nem duração de bateria. |

O percentual de CPU global que move o trem e o consumo **do processo CPU
Express** são medidas diferentes. O CSV externo mede o segundo. A carga do
observador pode participar da CPU global e, portanto, indiretamente do estado
visual; compare execuções com a mesma instrumentação.

### Estatísticas

- Os intervalos que cruzam o fim do aquecimento são excluídos inteiros. Não se
  inventa uma distribuição uniforme do trabalho dentro de um intervalo.
- A média ponderada de CPU usa a soma de CPU dividida pela soma das durações.
  A de FPS usa frames totais divididos por duração total dos intervalos.
- P50/P95 usam interpolação linear no índice `(N - 1) × p`, também conhecida
  como quantil tipo 7. São quantis **das amostras de intervalos**, sem peso de
  duração; o JSON informa separadamente a média ponderada, N e os intervalos.
- P95 de FPS não é P95 de latência de frame. O log de aproximadamente 1 Hz não
  permite reconstruir o tempo individual de cada frame.
- Falta de amostras produz `null`, com contagem de observações igual a zero.
  `null` não significa CPU de 0%, zero frames ocultos ou RAM de zero bytes.
- CSVs com BOM UTF-8 e números com vírgula decimal são aceitos. NaN, infinito,
  contadores regressivos, sobreposição de intervalos e deltas de CPU
  inconsistentes são rejeitados com erro; não são silenciosamente convertidos.
- Se a quantidade de processadores lógicos estiver ausente/`null`, a CPU em
  escala de um lógico continua disponível, mas a normalização total fica
  `null`. Não se usa o computador do analista como substituto do computador
  medido. Vírgula em números do CSV é separador decimal, não agrupador de milhar.

## Como executar

Pré-requisitos: Windows 10/11 x64, PowerShell 5.1 ou posterior e o executável
compilado em release. A análise posterior usa Python 3.10 ou posterior com
biblioteca padrão; o aplicativo não passa a depender de Python.

Na raiz do projeto, compile a versão que será medida:

```powershell
cargo build --release --locked
```

Guarde o hash/revisão da fonte usada e a identidade do artefato. O coletor grava
o SHA-256 do `.exe`; o wrapper registra o hash de `Cargo.lock` e do script de
coleta. Informe a versão do compilador do log de build em `-CompilerVersion`.
O wrapper não executa `rustc`/`rustup`, pois um proxy poderia baixar uma
toolchain ausente. `BuildProfile`, `SourceRevision` e `CompilerVersion` são
declarações do operador, não provas independentes de como um binário foi feito.

### 1. Visível, com diagnóstico de frames

```powershell
.\benchmarks\run-windows.ps1 `
  -Exe .\target\release\cpu-express.exe `
  -BuildProfile release -SourceRevision "revisao-ou-hash-da-fonte" `
  -CompilerVersion "versao-do-log-de-build" `
  -Scenario visible -PowerSource AC `
  -PowerModeNote "modo de energia efetivamente selecionado no Windows" `
  -DisplayNote "monitor, resolucao, escala e taxa de atualizacao" `
  -Seconds 300 -WarmupSeconds 30 -IncludeDiagnostics
```

Mantenha o trem visível. Execute sem outros processos de teste criando carga;
registre o trabalho de fundo habitual. As configurações do ensaio são
isoladas: 30 FPS, coleta de métricas em 1 segundo e os parâmetros originais do
trem. A preferência habitual do usuário não é sobrescrita.

### 2. Minimizado

Repita o mesmo comando com `-Scenario minimized`. Minimize o trem manualmente
**antes dos 30 segundos de aquecimento** e mantenha-o assim até encerrar. A
indicação de cenário é uma declaração: o wrapper não força a minimização nem
verifica `IsIconic`/estado Win32. O JSONL registra o campo `hidden` que a
aplicação observa; se ele continuar falso, isso é uma lacuna de observação,
não uma prova de que o processo renderizou corretamente minimizado.

### 3. Retomada

Em uma execução separada use `-Scenario restore` e registre em `-Notes` a
sequência planejada e realizada, por exemplo: minimizar após aquecimento,
aguardar 60 segundos e restaurar. Confira a idade/frescor das métricas e a
continuidade do trem no Windows. Os testes de worker cobrem o descarte de
amostras antigas; este ensaio verifica também a integração visual e nativa.

### 4. Custo do diagnóstico

Repita os cenários visível e minimizado **sem** `-IncludeDiagnostics`. Assim
continua havendo medição externa de CPU/memória, mas não gravação de métricas
ou contadores de frames pelo jogo. Compare pares com e sem diagnóstico para
estimar seu custo. Sem JSONL, o analisador devolve FPS/ocultação como
indisponíveis. O programa normal não grava esses logs. No caminho opt-in,
`Diagnostics::record` serializa e faz `flush` síncrono no thread da UI,
aproximadamente uma vez por segundo; a leitura não bloqueante do worker não
estende essa garantia ao logger.

### 5. Analisar os arquivos

Cada execução cria uma pasta própria. Use o caminho que o wrapper imprime:

```powershell
py -3 .\benchmarks\analyze.py "C:\caminho\da\execucao\cpu-express-DATA-ID"
```

O resultado `analysis.json` fica junto dos arquivos brutos. Preserve os três
arquivos originais e `benchmark-metadata.json` para auditoria. Também é
possível analisar uma execução feita diretamente pelo script antigo; os
metadados adicionais ausentes ficam explicitamente desconhecidos.

O analisador não junta o relógio externo do processo com o relógio de abertura
da aplicação como se seus zeros fossem idênticos. Os resumos de CPU/memória e
render são calculados nos seus respectivos domínios; o aquecimento é aplicado
separadamente. Não faça correlação de intervalos individuais dos dois logs sem
uma futura âncora temporal comum.

## Como ler os intervalos de renderização

`analysis.json` contém quatro grupos:

| Grupo | Regra |
| --- | --- |
| `all` | Todos os intervalos completos posteriores ao aquecimento. |
| `visible_at_both_endpoints` | Os dois registros informam `hidden=false`. |
| `hidden_at_both_endpoints` | Os dois registros informam `hidden=true`. |
| `transition_or_unknown` | Mudança de estado ou ausência de informação em um extremo. |

Cada grupo traz número de intervalos, duração observada, delta de frames,
média ponderada e distribuição P50/P95 de FPS. Se não houver intervalo do
grupo, a contagem de frames observados e o FPS ficam `null`.

Dois extremos ocultos não excluem uma restauração e nova ocultação entre eles.
Por isso `guaranteed_frames_while_hidden` continua `null` no analisador legado.
A UI consolidada acrescenta contadores `rendered_frames_visible` e
`rendered_frames_hidden` no ponto de rasterização, além de uma guarda que
impede o desenho quando a aplicação observa ocultação/minimização. Esses
campos ficam no JSONL bruto; o analisador conserva sua semântica anterior.

No Windows, o controlador consulta o estado nativo da janela. Ainda é
necessário executar o protocolo no PC de destino, correlacionar as ações
observadas com os registros e medir o consumo. Testes headless da guarda
não substituem essa validação de plataforma.

## Condições para comparar versões

Faça ao menos três repetições por cenário e alterne a ordem entre versões,
sempre no mesmo computador, monitor, resolução, escala, driver, fonte de
energia, plano/modo de energia e perfil release. Registre a carga de fundo e
preserve a mesma instrumentação. O mínimo de 300 segundos com aquecimento de
30 segundos é um protocolo proposto, não uma garantia estatística; se a
variabilidade entre repetições exceder a diferença, o resultado é inconclusivo.

O inventário WMI de adaptadores/driver não prova qual renderer OpenGL foi
selecionado. Um plano ativo também não descreve sozinho todos os controles de
energia do Windows. Campos desconhecidos devem continuar desconhecidos, com
notas manuais quando necessárias. Uma redução de CPU não é automaticamente
uma medição de economia de bateria.

Declare, por comparação, a revisão e hash do executável, máquina/driver,
cenário e sua evidência, diagnóstico ligado/desligado, N, duração útil,
CPU equivalente a um lógico, CPU normalizada total, segundos de CPU,
working set, private bytes, FPS rasterizado e suas limitações. Resultados com
erro, término forçado, período incompleto ou mudança de energia devem ser
identificados. Não una execuções incompletas e completas sem distinção.

## Validação destas ferramentas

```powershell
py -3 -m unittest discover -s benchmarks -p "test_*.py" -v
```

Os 19 testes verificam cálculo ponderado, normalização por processadores
lógicos, percentis, exclusão de aquecimento parcial, ausência de dados,
localização decimal, BOM, NaN/infinito/valores negativos, reset de contadores,
transições de visibilidade, registros duplicados, preservação de execução
incompleta e o caminho completo CSV/JSONL → CLI → JSON. Usam dados de teste;
não executam carga sintética para movimentar o trem nem constituem medição de desempenho do jogo.

## Referências e proveniência

- Código da baseline: `scripts/measure-windows.ps1` para coleta de processo;
  `src/app.rs`, `record_diagnostics` e `step_frame` para significado dos frames.
- [Microsoft — Process.TotalProcessorTime](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.totalprocessortime): tempo de CPU do processo, soma de modo usuário e privilegiado.
- [Microsoft — Process.WorkingSet64](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.workingset64): memória física do processo e páginas residentes.
- [Microsoft — Process.PrivateMemorySize64](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.privatememorysize64): memória privada, não compartilhável entre processos.
- [Microsoft — powercfg](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options): `/getactivescheme` consulta o plano ativo; este wrapper não altera planos.

Documentação oficial consultada em 13/11/2026. As fórmulas e a classificação
conservadora de intervalos são implementadas localmente em `analyze.py`.
