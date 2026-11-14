# CPU Express 0.1 — consumo e estabilidade medidos

**Data: 13/11/2026. Ambiente: Linux x86-64 de desenvolvimento, Xorg com tela virtual, Mesa por software, 9 processadores lógicos disponíveis. Não são medições do executável no Windows.**

O build release com a feature de laboratório `host-validation` executou o mesmo núcleo, renderer e coletor sysinfo usados pelo produto. A implementação Win32 de janela não foi executada. O ensaio de minimização usou Openbox. O código de execução é o commit `48cb9d97fb2d0e95dca74f235a11dc07fb187557`.

## Método

Foram feitos dois ensaios sequenciais, com CPU real, configuração padrão de 30 FPS e coleta a aproximadamente 1 Hz. Não foi executado gerador de carga artificial. Durante os dois ensaios não houve captura de imagens nem compilação concorrente iniciada pelo projeto. O host é virtualizado e compartilhado; atividades externas não são controladas.

O observador leu tempo acumulado de CPU e memória residente do processo pelo `/proc`, aproximadamente uma vez por segundo. O tempo de CPU inclui as threads da aplicação; não inclui o servidor Xorg nem o observador. O diagnóstico JSONL da aplicação estava ligado a aproximadamente 1 Hz para verificar frames, amostras, alvo de velocidade e encerramento.

A utilização média foi calculada como:

```text
CPU em % de um processador lógico = 100 × Δtempo_de_CPU / Δtempo_decorrido
CPU em % da capacidade total disponível = valor anterior / 9
```

Assim, “10,25% de um processador lógico” não significa “10,25% da máquina inteira”. A normalização considera os 9 processadores lógicos expostos ao ambiente. RSS representa páginas residentes do processo, incluindo páginas compartilhadas; não mede toda a memória da GPU/servidor gráfico nem equivale ao contador Private Bytes do Windows.

## Resultados

| Medida | Janela visível | Janela minimizada |
|---|---:|---:|
| Duração total do processo | 300,34 s | 90,28 s |
| Janela usada para análise | 15,089–300,037 s | 20,074–69,014 s |
| CPU média, % de um processador lógico | **10,25%** | **0,102%** |
| CPU média, % da capacidade dos 9 processadores lógicos | **1,14%** | **0,0114%** |
| RSS mediana | **73,95 MiB** | **73,91 MiB** |
| RSS mínima–máxima no intervalo | 72,24–74,04 MiB | 73,91–73,91 MiB |
| Frames da cena por segundo no intervalo | **29,17** | **0** |
| Código de saída | **0** | **0** |

No ensaio visível, os primeiros 15 segundos foram excluídos da análise de consumo. O diagnóstico final registrou **8.747 frames e 300 amostras recebidas**. Todas as amostras do intervalo analisado estavam válidas e recentes, os alvos obedeceram à fórmula da CPU e a quantidade de fumaça permaneceu dentro do limite. As diferenças entre alvo e velocidade atual são esperadas durante a suavização.

No segundo ensaio, a janela foi minimizada aproximadamente aos 15 segundos e restaurada aos 75 segundos. A análise de consumo usa somente 20–70 segundos, evitando as transições. Nesse intervalo, os frames acumulados permaneceram em **435** enquanto o número de amostras recebidas passou de **21 para 70**. Depois de restaurar, o contador de frames voltou a avançar, terminando em **880 frames e 90 amostras**. A aplicação permaneceu responsiva e encerrou normalmente.

## Estabilidade e limites da conclusão

No intervalo analisado da execução visível, a RSS começou em 74,04 MiB e terminou em 72,24 MiB. **Não houve tendência contínua de crescimento observada nesses cinco minutos.** A execução minimizada manteve a RSS constante no intervalo analisado. Isso é evidência de estabilidade nesses ensaios, e não uma prova de ausência de vazamento em horas ou dias. Não foi usado um detector de vazamento do Windows.

O custo observado é compatível com continuar o desenvolvimento, mas não estabelece um orçamento universal de CPU/RAM para Windows. O renderer por software, o ambiente virtualizado, os drivers e o número de processadores diferem de um desktop comum. É necessário medir o release no computador Windows de destino para concluir se o custo operacional atende àquele uso. O próprio jogo participa da CPU global que orienta sua velocidade; os números da tabela medem separadamente o custo do processo.

Quando minimizada, a aplicação suspende o desenho e o avanço visual, mantendo a coleta aproximadamente a 1 Hz. No Windows, a versão de winit utilizada não oferece notificações gerais de oclusão: uma janela coberta por outra pode continuar desenhando. Essa limitação é distinta da minimização.

## Executável e otimização de pixels

O executável Windows x64 release possui **5.287.936 bytes**: **5,29 MB** em unidades decimais ou **5,04 MiB** em unidades binárias. A medida é do `.exe`, sem os documentos e as licenças do pacote.

SHA-256:

```text
759322f458eb223f163884b4d1c14450dd6bdc60a035edd4d906752a3aafa24f
```

A textura enviada ao renderer foi reduzida de 640 × 140 para a grade nativa de 320 × 70, ampliada com filtragem nearest. O buffer RGBA passou de 358.400 para 89.600 bytes: **75% menos pixels enviados por atualização**. A imagem de referência exportada permaneceu idêntica byte a byte, conforme [pixel-equality.json](../validation/pixel-equality.json). Essa redução de payload não é apresentada como redução de 75% no consumo total de CPU ou RAM.

Existe um ensaio exploratório anterior em [release-baseline/result.json](../validation/release-baseline/result.json), mas ele usou duração e entrada diferentes. Não é um experimento controlado para estimar ganho de CPU da otimização; os números principais acima são da versão final.

## Repetir a medição no Windows

Na pasta extraída do pacote Windows, com PowerShell 5.1 ou posterior:

```powershell
.\scripts\measure-windows.ps1 -Exe .\cpu-express.exe -Seconds 300 -IncludeDiagnostics
```

O script cria uma pasta de resultados própria, uma configuração isolada de 30 FPS, CSV com amostras do processo e JSON com metadados, hash, CPU, Working Set e Private Bytes. Os primeiros 15 segundos ficam fora da estatística principal. Não altera as preferências normais, não instala ferramentas, não gera carga e não envia dados. O aplicativo fecha ao final da duração solicitada.

Execute um ensaio com a janela visível. Para comparar com minimização, execute outro ensaio, minimize logo após abrir e mantenha assim até terminar; registre a ação ao interpretar os resultados. Para uma observação de maior duração, use `-Seconds 900` ou outro intervalo adequado. **O script foi revisado estaticamente, mas não executado em Windows/PowerShell nesta entrega.** Respeite a política de execução de scripts já adotada no computador.

## Dados e rastreabilidade

- [Resumo estruturado e intervalos exatos](../validation/benchmark-summary.json).
- [Processo visível: amostras brutas](../validation/release-stability/result.json) e [diagnóstico da aplicação](../validation/release-stability/metrics.jsonl).
- [Minimização final: amostras brutas](../validation/release-minimized-final/result.json) e [diagnóstico da aplicação](../validation/release-minimized-final/metrics.jsonl).
- [Relatório de testes e limites de execução](VALIDATION.md).

Os caminhos absolutos presentes nos comandos registrados identificam o laboratório em que os ensaios foram executados; não são caminhos que o usuário precise reproduzir.
