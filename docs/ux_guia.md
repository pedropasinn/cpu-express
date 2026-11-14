# CPU Express 0.2.0-rc.1 — guia de uso

O CPU Express reúne um pequeno trem, uma viagem contínua e as métricas do computador em uma janela horizontal de aproximadamente **640 × 140 pontos lógicos**. Ela fica perto da barra de tarefas do Windows, **em uma janela própria, sem ser embutida na barra**. O tamanho físico acompanha a escala do monitor.

Este guia descreve a versão consolidada das cinco frentes. O produto tem como alvo **Windows 10/11 x64**; o recurso de desenvolvimento `host-validation` não constitui um port Linux. **Não houve validação nativa em Windows 10/11 nesta sessão.** Os passos abaixo são instruções de uso; o [checklist Windows](windows_checklist.md) registra o que ainda precisa ser conferido na máquina de destino.

## 1. Abrir e experimentar

Extraia o pacote completo e abra `cpu-express.exe`. A janela deve aparecer perto da barra de tarefas e iniciar a coleta de CPU/RAM. A leitura de CPU precisa de um intervalo inicial; nesse período, a interface mostra que está lendo a CPU.

Arraste uma área livre da paisagem para mover a janela. **R** ou **Opções → Trem → Reposicionar** aproxima a janela da barra no monitor em uso. Para experimentar sem consultar os contadores reais, abra o PowerShell na pasta do executável e execute:

```powershell
.\cpu-express.exe --demo
```

O DEMO começa em 50% de CPU simulada. Abra **Opções → Trem** para experimentar 0%, 50%, 100% ou um valor intermediário. Nenhum desses controles gera carga artificial no processador.

Se você estiver usando a fonte, o script de build e empacotamento está documentado em [CI Windows](windows_ci.md). Este guia usa os comandos do pacote binário.

## 2. CPU real, simulação manual e DEMO

| Modo | Como iniciar | Entrada que move o trem | Coleta real |
|---|---|---|---|
| Real | Abrir normalmente | Utilização global da CPU | CPU e RAM são coletadas em segundo plano. |
| Manual | `--cpu 50` ou **Simular CPU** na aba Trem | Valor escolhido no painel | A coleta real continua, inclusive para a RAM; a entrada manual aparece identificada como simulada. |
| DEMO | `--demo`, com `--cpu` opcional | Valor escolhido; padrão 50% | O coletor `sysinfo` de CPU/RAM não é inicializado; a RAM fica indisponível. |

Para iniciar exemplos específicos:

```powershell
.\cpu-express.exe --demo --cpu 0
.\cpu-express.exe --demo --cpu 100
.\cpu-express.exe --cpu 50
```

Use uma instância por vez para comparar. Para sair de uma sessão DEMO e usar CPU real, encerre o aplicativo e abra novamente sem `--demo`. No modo real, você pode ativar ou desativar a simulação pela aba Trem.

Com os parâmetros padrão, 0%, 50% e 100% correspondem a velocidades-alvo de **12, 56 e 100 km/h**. A velocidade muda suavemente e pode levar alguns instantes para alcançar o novo alvo. Ela é uma medida fictícia do jogo. A CPU mostrada é a utilização global do computador, que também inclui o próprio aplicativo; ela não representa apenas o consumo do CPU Express.

Dados ausentes ou antigos recebem uma indicação de indisponibilidade ou atraso. A interface não deve apresentar uma leitura antiga como atual, nem usar `0 / 0 GiB` como uma leitura de RAM válida.

## 3. As quatro abas de Opções

Pressione **O** ou clique em **Opções**. O painel usa rolagem para acomodar os controles na pequena janela.

| Aba | Conteúdo |
|---|---|
| **Trem** | CPU manual, presets 0/50/100%, ajuste de 0,1 ponto percentual, pausa/retomada e reposicionamento. |
| **Viagem** | Trecho atual, próxima estação, percentual do trecho, distância restante, visitas, circuitos, estações descobertas e conquistas. |
| **Visual** | HUD, recolhimento de controles, cenário Viagem/Clássico, nível de detalhes, ciclo de luz, nuvens, chuva, aparência dos três vagões, FPS e resposta da velocidade. |
| **Janela** | Sempre no topo, pausa ao trabalhar em outra janela, iniciar pausado, minimizar e ocultar na bandeja. |

**F1** abre a ajuda. **Esc** fecha o painel aberto e mantém o programa funcionando. O botão **Sair** encerra a aplicação.

## 4. A viagem e os quatro biomas

O **Circuito Aurora** percorre quatro estações e retorna ao início:

| Trecho | Paisagem principal |
|---|---|
| Vila Aurora → Bosque dos Pinheiros | Vale verde |
| Bosque dos Pinheiros → Serra Clara | Floresta de pinheiros |
| Serra Clara → Porto do Sol | Serra árida |
| Porto do Sol → Vila Aurora | Litoral |

A paisagem acompanha o trecho real da viagem e faz a transição para o próximo bioma ao se aproximar da estação. A chegada não reinicia o deslocamento das camadas do cenário. O trem passa pelas estações; as chegadas atualizam contadores, descobertas e avisos curtos na janela.

A distância usa **unidades fictícias de viagem**. Ela não é uma contagem de quilômetros reais nem uma medida de trabalho realizado no computador. No HUD compacto, a parte inferior mostra a próxima estação e o percentual do trecho; os detalhes ficam na aba Viagem.

Há três conquistas locais:

| Conquista | Condição |
|---|---|
| **Primeira estação** | Chegar à primeira estação após a partida. |
| **Primeiro circuito** | Completar a primeira volta. |
| **Longa jornada** | Acumular 36.000 unidades de viagem. |

**Viagem, distância, descobertas e conquistas pertencem à sessão atual. Ao encerrar e reabrir, o progresso reinicia.** As preferências de aparência e comportamento são salvas separadamente. Esta versão não acumula progresso enquanto está fechada.

## 5. Aparência e HUD

A aba Visual permite escolher o cenário **Viagem**, com os quatro biomas ligados à rota, ou o cenário **Clássico**. No cenário Viagem, você pode escolher detalhes completos ou reduzidos, luz, nuvens, clima limpo ou chuva e a aparência de cada vagão.

Os vagões são numerados **da cauda para a locomotiva**. As opções são Passageiros, Panorâmico, Dormitório, Carga e Tanque. A escolha altera a aparência. O modo de detalhes reduzidos omite efeitos decorativos; algumas opções ficam desabilitadas enquanto ele está ativo.

O ciclo de luz dura **dez minutos de tempo ativo da simulação** e não acompanha a hora real do computador. Pausar a viagem também pausa esse ciclo. O limite de animação vai de 10 a 60 FPS, com padrão de 30 FPS. A opção **Resposta** controla a rapidez com que o trem se aproxima da nova velocidade: valores maiores produzem uma reação mais rápida.

**M** alterna as informações sobre o cenário:

| HUD | Apresentação |
|---|---|
| **Compacto** | CPU, velocidade e, quando há espaço, próxima estação/progresso. |
| **Completo** | Mais espaço para CPU, RAM e velocidade; pode usar apresentação compacta com um painel aberto ou em janela estreita. |
| **Limpo** | Reduz os indicadores fixos; continuam os avisos necessários de DEMO, simulação, pausa e falhas, além dos avisos temporários da viagem. |

Os controles podem se recolher quando o ponteiro está fora da janela e ela não tem foco. Volte com o ponteiro ou dê foco para acessá-los; essa preferência pode ser desativada. Passe o mouse sobre botões e indicadores para ler suas explicações.

## 6. Pausa, minimização, bandeja e saída

| Ação | Efeito | Como continuar |
|---|---|---|
| **Pausar** / Espaço | Congela viagem, paisagem, rodas e fumaça. | **Seguir** / Espaço. |
| **Minimizar** | Recolhe a janela e suspende a animação. | Botão do aplicativo na barra de tarefas ou **Mostrar trem** na bandeja. |
| **Ocultar na bandeja** | Esconde a janela e suspende a animação. | Ícone do CPU Express na área de notificação; clique para mostrar ou use o menu. |
| **Sair** / Ctrl+Q | Encerra a instância e libera os recursos. | Abrir o programa novamente inicia outra sessão. |

O tempo em pausa, minimizado ou oculto não vira uma distância acumulada a percorrer ao retornar. A coleta real continua em segundo plano nos modos real/manual; em DEMO, permanece sem coleta real. A janela oculta não deve reconstruir a cena continuamente. A economia efetiva de recursos precisa ser medida no Windows utilizado.

**Ocultar na bandeja** só fica habilitado quando a bandeja está disponível para recuperar a janela. Se houver falha na criação do ícone, a aplicação informa o problema e mantém a opção de minimizar. O ícone pode estar dentro da área de ícones ocultos do próprio Windows; ele ainda usa um desenho genérico do sistema.

O menu da bandeja oferece **Mostrar trem**, **Ocultar trem**, **Reposicionar na área útil**, **Alternar sempre no topo** e **Sair do CPU Express**. Mudanças de monitor ou de área útil acionam recuperação de posição quando necessário. Se a janela não aparecer após uma mudança, use **Mostrar trem** e **Reposicionar na área útil** pelo menu.

A preferência **Pausar ao trabalhar em outra janela** é desligada por padrão, permitindo que o trem acompanhe o trabalho em segundo plano. **Iniciar pausado na próxima abertura** é uma preferência independente. Uma pausa manual permanece até você solicitar a retomada.

## 7. Atalhos locais

Os atalhos exigem foco na janela e cedem aos campos de edição/controles de entrada. Eles não capturam teclas globalmente.

| Tecla | Ação |
|---|---|
| Espaço | Pausar ou retomar |
| M | Alternar HUD |
| P | Alternar sempre no topo |
| O | Abrir ou fechar Opções |
| F1 | Abrir ou fechar ajuda |
| R | Reposicionar perto da barra de tarefas |
| Esc | Fechar o painel; não encerra o app |
| Ctrl+Q | Sair, fora de um campo em edição |
| Tab / Shift+Tab | Percorrer controles |

## 8. Preferências e conservação dos arquivos

No Windows, o arquivo padrão é `%LOCALAPPDATA%\CPU Express\config.json`. A variável `CPU_EXPRESS_CONFIG` pode definir outro caminho; `--config` tem prioridade sobre ela. Para experimentar com preferências separadas:

```powershell
.\cpu-express.exe --demo --config .\preferencias-demo.json
```

As alterações explícitas de HUD, aparência dos vagões, cenário, efeitos, qualidade, FPS, resposta, topo e preferências de pausa são salvas. A gravação agrupa mudanças sucessivas de controles, e a saída normal tenta concluir a gravação pendente.

Configurações antigas ou parciais recebem valores padrão para os campos ausentes. Campos desconhecidos de documentos válidos são preservados, inclusive em objetos aninhados. Um arquivo corrompido, incompatível ou ilegível permanece intacto: a sessão usa valores seguros e mostra um aviso se não consegue salvar. Corrija esse arquivo ou escolha outro caminho com `--config`.

O modo DEMO, a porcentagem manual, a pausa manual atual e o progresso da viagem não são salvos como preferências de sessão.

## 9. Diagnóstico solicitado explicitamente

```powershell
# Demonstração de dez segundos, sem coleta real.
.\cpu-express.exe --demo --cpu 50 --smoke-seconds 10

# Sessão real de dez segundos com registro local.
.\cpu-express.exe --smoke-seconds 10 --diagnostics .\diagnostico.jsonl
```

`--smoke` equivale a `--smoke-seconds 3`; nenhum dos dois produz estresse de CPU. `--diagnostics` grava um arquivo local no caminho escolhido e substitui o conteúdo de uma saída existente; use um nome novo para cada captura. O diagnóstico e a configuração devem usar arquivos diferentes. A interface F1 oferece ajuda mesmo quando a saída de `--help` não aparece diretamente no terminal por se tratar de um executável GUI.

Para relatar um problema, informe a versão, o que fez, o resultado observado, a escala do Windows e se estava em modo real, manual ou DEMO. O [checklist de aceitação](windows_checklist.md) fornece um roteiro reproduzível.
