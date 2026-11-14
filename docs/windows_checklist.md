# CPU Express 0.3.0-rc.7 — checklist de aceitação no Windows

Atualizado em 2026-11-14. Este documento separa **evidência já registrada da rc7** de **aceitação manual ainda pendente**. Não significa que todos os testes abaixo foram executados no hardware de Pedro. Para a expansão, consultar [PLANO_EXECUCAO.md](PLANO_EXECUCAO.md).

## 1. O que a evidência atual permite afirmar

| Fonte | Resultado registrado | Limite |
| --- | --- | --- |
| [BUILD-INFO](../BUILD-INFO.json) e [checks](../evidence/current/checks.json) | Release Windows GNU rc7; format, Clippy e testes com saída zero; 203 aprovados e dois ignorados | Não é nova execução nesta revisão; testes ignorados não são aprovados |
| [Smoke nativo](../evidence/current/native-smoke-result.json) | Saída zero, sem timeout, shutdown registrado, faixa alinhada e painel independente detectados | Captura da faixa; `panel_capture=false`. Não comprova toda interação do painel ou composição final no desktop |
| [Transições rc7](../evidence/current/transitions-rc7/result.json) | Cinco estágios; faixa 274 × 60 e retângulo flutuante externo 800 × 240 recuperados no cenário ensaiado | Dimensões são deste ensaio; não são defaults universais nem matriz multimonitor |
| [Métricas reais](../evidence/current/real-metrics-result.json) | Quatro amostras reais válidas no ensaio curto | Não é benchmark de consumo do aplicativo |
| [Clima](../evidence/current/weather/result.json) | Consulta online com 25 registros ready e encerramento normal | Não são 25 consultas independentes; não certifica todos os climas/falhas |
| [UI rc7](../evidence/current/game-ui-result.json) | Dez painéis exportados pelo egui, com hashes de fontes/imagens | São renders headless, não screenshots do compositor |
| [Cadência](../evidence/current/visual-refresh/comparison.json) | Comparação curta local a 60 FPS: aproximadamente 52,87 → 59,81 frames renderizados/s | Não mede FPS apresentado, latência individual, energia nem custo de CPU/GPU |

Referência: SHA-256 `a1febb5706989ab5578496261df37cd8e817e8f133fb1aeac8772706b1e842f6`. O pacote registrado não é assinado. A matriz Windows/DPI/monitores, Explorer, autohide, conflitos com widgets e estabilidade prolongada continuam pendentes. Reexecutar casos afetados quando o binário mudar; não atribuir estes resultados à expansão em andamento.

A faixa é uma janela **independente na área da barra horizontal principal**, não filha do Explorer e não uma reserva de espaço. O painel de menus é outra janela nativa. Clássico/qualidade reduzida/ciclo artístico manual não são opções atuais do produto; o clima local substituiu esses controles. Adaptadores históricos do renderer não são recursos da interface.

## 2. Preparação e registro

Use o pacote completo, uma instância por perfil e arquivos isolados. Para testar retomada real, execute normalmente: DEMO e `--smoke-seconds` não carregam/gravam a jornada normal. Não use o perfil cotidiano para corrupção, migração ou falhas de escrita. Antes de reiniciar Explorer ou desconectar monitores, salve o trabalho aberto.

| Campo | Valor a preencher |
| --- | --- |
| Responsável/data/hora | PENDENTE |
| Versão, caminho do pacote e SHA-256 | PENDENTE — referência rc7 acima |
| Windows: edição/build (`winver`), GPU/driver | PENDENTE |
| CPU/RAM; energia AC/bateria; plano de energia | PENDENTE |
| Monitores: resolução, escala, posição e principal | PENDENTE |
| Barra: borda, altura, auto-hide, widgets | PENDENTE |
| Perfil isolado; real/manual/DEMO; rede | PENDENTE |
| Diretório de logs e capturas | PENDENTE |

No PowerShell, na pasta do executável:

```powershell
Get-FileHash -LiteralPath .\cpu-express.exe -Algorithm SHA256
$perfilChecklist = Join-Path $env:TEMP ('cpu-express-checklist-' + [guid]::NewGuid().ToString('N') + '.json')
& .\cpu-express.exe --config $perfilChecklist
```

Preencher cada caso como **APROVADO**, **FALHOU**, **NÃO EXECUTADO** ou **N/A com justificativa**, incluindo ambiente e evidência. Os estados PENDENTE abaixo são da execução manual a registrar, mesmo quando há cobertura automatizada parcial. Não marcar uma combinação de hardware indisponível como aprovada.

## 3. Primeiro uso, central e teclado

| ID | Procedimento | Critério de aceite | Resultado |
| --- | --- | --- | --- |
| A01 | Abrir perfil novo, aguardar coleta, abrir/fechar central clicando na faixa | Trem visível, CPU válida ou estado de indisponibilidade; central separada sem deslocar a faixa | PENDENTE |
| A02 | Observar CPU real e alternar programas sem gerar carga artificial | Valor e velocidade respondem suavemente; valor ausente/antigo não aparece como atual | PENDENTE |
| A03 | Conferir CPU/RAM na cabine e HUD | Unidades coerentes, CPU simulada identificada, RAM ausente distinta de zero | PENDENTE |
| A04 | Navegar Trem, Viagem (Mapa/Objetivos/Diário), Visual e Janela | Conteúdo corresponde à página; controles e mensagens são alcançáveis | PENDENTE |
| A05 | Tab/Shift+Tab até controles abaixo da dobra; editar CPU; usar Esc/F1 | Foco visível e rolagem até o controle; atalhos não alteram valor durante edição; fechar painel não encerra processo | PENDENTE |
| A06 | Redimensionar central para mínimo admitido (480 × 480 pontos na rc7), mudar DPI e rolar | Navegação e rodapé acessíveis, texto não impede operação, sem rolagem horizontal involuntária | PENDENTE |
| A07 | Alternar HUD, retirar mouse/foco e retornar | Modo persiste, controles reaparecem, DEMO/pausa/simulação reconhecíveis | PENDENTE |
| A08 | Arrastar área livre no modo flutuante; usar botões, cartões e scroll | Apenas arraste deliberado move janela; posição estabilizada não salta ao canto | PENDENTE |
| A09 | Abrir menus/seletores e cancelar; alternar páginas repetidamente | Nenhuma seleção acidental, popup preso, perda de foco ou scroll de outra página | PENDENTE |
| A10 | Reabrir painel após fechar; testar foco no editor durante aparecimento | Reabrir funciona; aparição automática não rouba digitação; clique deliberado permite foco | PENDENTE |

## 4. Tempo ativo, pausa e jornada

Para DEMO, usar outro perfil isolado: `--demo --cpu 50 --config <perfil-demo>`. Não confundir sua retomada com a de um perfil normal.

| ID | Procedimento | Critério de aceite | Resultado |
| --- | --- | --- | --- |
| D01 | Abrir DEMO a 50% | DEMO explícito, RAM indisponível; não há carga artificial para mover o trem | PENDENTE |
| D02 | Alternar CPU manual 0/50/100 na cabine | Com padrões, alvos 12/56/100 km/h e transição suave; metas da jornada não aceleram | PENDENTE |
| D03 | Pausar dez segundos e retomar; repetir com painel aberto | Rodas, fumaça, viagem e serviço param; não há avanço acumulado na retomada | PENDENTE |
| D04 | Habilitar pausa sem foco, alternar para editor e entre faixa/central | Foco da central conta como uso do app; pausa manual não é desfeita por retorno de foco | PENDENTE |
| D05 | Acompanhar circuito e uma parada completa | Ordem das quatro estações, aproximação, parada de serviço de oito segundos ativos e partida coerentes | PENDENTE |
| D06 | Comparar segmentos em 30 e 60 FPS e CPU manual baixa/alta | Mesmo ritmo lógico por tempo ativo; velocidade visual pode diferir | PENDENTE |
| D07 | Abrir/fechar menus e alterar ajustes rapidamente durante movimento | Animação não fica aguardando deadlines acumulados depois da interação; sem rajadas de compensação | PENDENTE |
| D08 | Reiniciar perfil normal após Salvar viagem agora, inclusive durante serviço | Retoma progresso/serviço válido e desbloqueios, sem visitas ou prêmios duplicados | PENDENTE |
| D09 | Reiniciar DEMO e executar smoke com perfil separado | Jornada de teste não substitui a jornada normal; evidência declara modo utilizado | PENDENTE |
| D10 | Bloquear sessão/suspender Windows e retornar | Sem salto offline, perda de acesso à janela ou violação do estado de pausa | PENDENTE |

A mudança de clima externo não é avanço da jornada: verificar separadamente atualização meteorológica e animações baseadas em tempo ativo.

## 5. Oficina, apresentação e clima

| ID | Procedimento | Critério de aceite | Resultado |
| --- | --- | --- | --- |
| V01 | Escolher cada uma das três posições e os cinco cartões de vagão | Posição equipada corresponde à seleção e à miniatura; ordem cauda → locomotiva; rodas/engates alinhados | PENDENTE |
| V02 | Visitar Pinturas em perfil novo e com recompensas liberadas | Requisitos e equipado claros; bloqueadas não equipam; desbloqueio normal em 1/4/12 chegadas, sem duplicação | PENDENTE |
| V03 | Trocar composição e pintura durante pausa | Uma atualização visual sem mover viagem; não recria texturas de catálogo continuamente | PENDENTE |
| V04 | Examinar trem na faixa real de 60 px e no flutuante | Pixels nearest, vagões identificáveis; recorte permitido do céu não oculta rodas/trilho ou ações essenciais | PENDENTE |
| V05 | Mudar FPS/HUD/equipamento/topo, sair e reabrir normalmente | Preferências persistem; perfil antigo conserva FPS; novo perfil usa 60; CPU manual não vira leitura real | PENDENTE |
| V06 | Iniciar pausado: ativar, reiniciar, desativar e reiniciar | Preferência respeitada sem restabelecer pausa manual indevidamente | PENDENTE |
| V07 | Abrir Clima online, aguardar ou solicitar atualização | São Paulo e origem/horário claros; consulta fora da UI, botão respeita intervalo mínimo | PENDENTE |
| V08 | Abrir sem rede; depois perder rede após resposta válida | Distingue ilustrativo, antigo e falha; viagem/UI continuam; dado velho não é chamado de atual | PENDENTE |
| V09 | Recuperar rede e repetir atualização rapidamente | Recuperação sem travar nem acumular consultas; não exige reiniciar o jogo | PENDENTE |
| V10 | Inspecionar condições e dia/noite com fixtures determinísticas | Limpo/nublado/neblina/chuva/tempestade/neve legíveis; log distingue fixture de consulta real | PENDENTE |
| V11 | Abrir cópia de preferências anteriores com clássico/reduzido | Migração para apresentação completa, sem restaurar controles removidos ou perder equipamento válido | PENDENTE |

Fixtures meteorológicas provam apresentação, não ocorrência real de neve/tempestade na cidade. Em DEMO/smoke a rede normalmente fica desativada; `CPU_EXPRESS_LIVE_WEATHER=1` é opt-in de validação e deve ser retirado após o ensaio. Não publicar logs com dados pessoais desnecessários.

## 6. Bandeja e recuperação

| ID | Procedimento | Critério de aceite | Resultado |
| --- | --- | --- | --- |
| T01 | Localizar bandeja e abrir menu | Ações acessíveis; falha detectada não deixa Ocultar como caminho sem volta | PENDENTE |
| T02 | Minimizar/restaurar pela barra | Mesma sessão, sem progresso minimizado | PENDENTE |
| T03 | Ocultar/mostrar dez vezes | Janela recuperável, sem duplicação de janelas/ícones ativos | PENDENTE |
| T04 | Minimizar e escolher Mostrar trem na bandeja | Restaura sem salto temporal | PENDENTE |
| T05 | Oculto, alternar topo e mostrar | Preferência coerente, sessão acessível | PENDENTE |
| T06 | Abrir menu da bandeja, teclado e Esc | Fecha sem ação acidental ou menu preso | PENDENTE |
| T07 | Reiniciar Explorer com trem oculto | Bandeja reaparece ou janela recuperada com aviso; existe caminho de saída | PENDENTE |
| T08 | Reiniciar Explorer com trem visível | Faixa recuperável sem duplicação; painel utilizável | PENDENTE |
| T09 | Sair visível, minimizado e oculto, em execuções separadas | Sair encerra; não converte em ocultação | PENDENTE |
| T10 | Verificar processo após saída e contadores após ciclos | Sem processo preso; crescimento de handles/threads investigado com série, não observação isolada | PENDENTE |

```powershell
Get-Process -Name cpu-express -ErrorAction SilentlyContinue |
  Select-Object Id, Handles, @{Name='Threads'; Expression={$_.Threads.Count}}, CPU, WorkingSet64
```

`CPU` acima é tempo acumulado, não percentual. Não confundir CPU global exibida no trem com consumo do processo.

## 7. Geometria rc4, DPI e monitores

| ID | Procedimento | Critério de aceite | Resultado |
| --- | --- | --- | --- |
| G01 | Alternar barra/flutuante cinco vezes com painel aberto | Aplicação das bordas antes de restaurar geometria; sem retângulo intermediário salvo ou superfície extrapolada | PENDENTE |
| G02 | Ajustar offset/largura/altura da faixa; reabrir | Limites respeitam barra; altura física válida; não cria reserva de espaço no Explorer | PENDENTE |
| G03 | Arrastar/redimensionar flutuante, aguardar persistência e reiniciar | Retângulo independente da faixa recuperado, incluindo coordenadas negativas válidas | PENDENTE |
| G04 | Usar central e depois voltar à faixa de 60 px | Tamanho mínimo do painel/UI não amplia a faixa; render fica limitado ao viewport real | PENDENTE |
| W01 | Executar A04–A09/G01–G04 em 100/125/150/200% | Legibilidade e pixels nítidos; registrar cada escala separadamente | PENDENTE |
| W02 | Mover flutuante/central entre monitores de escalas diferentes | Escala e área útil corretas; não desloca faixa arbitrariamente | PENDENTE |
| W03 | Segundo monitor à esquerda/acima (coordenadas negativas) | Janela acessível e posição válida preservada | PENDENTE |
| W04 | Desconectar monitor que contém janela | Recuperação em monitor disponível, automática ou via ação documentada | PENDENTE |
| W05 | Trocar principal/resolução e reposicionar | Recupera retângulo inválido sem reenquadrar posição válida a cada frame | PENDENTE |
| W06 | Auto-hide: revelar/ocultar barra e mostrar trem | Barra, bandeja e recuperação acessíveis; registrar conflitos de sobreposição | PENDENTE |
| W07 | Barra vertical/outra borda, se suportada pelo Windows | Variante ainda não oferecida: fallback flutuante e aviso quando aplicável, não encaixe prometido | PENDENTE |
| W08 | Widgets/Start/ícones próximos à faixa; variar offset | Conflitos registrados, recuperação e ajuste acessíveis; não alegar reserva automática de espaço | PENDENTE |

Matriz a preencher separadamente: Windows 10/11 × escalas disponíveis × um/dois monitores × barra normal/autohide. Não instalar modificadores de shell para declarar cobertura. Shell alternativo, barra secundária e barra vertical não são variantes certificadas da rc7.

## 8. Saves, migração e falhas recuperáveis

Os casos abaixo usam **cópias isoladas** e execução normal. Registre hash antes/depois de arquivos que devem ser preservados. Não apague lock files para contornar escritor ativo.

| ID | Procedimento | Critério de aceite | Resultado |
| --- | --- | --- | --- |
| C01 | Abrir config inválida isolada | Defaults seguros e aviso; original não substituído | PENDENTE |
| C02 | Alterar preferência e sair com config inválida | Original preservado byte a byte; restrição de gravação comunicada | PENDENTE |
| C03 | Criar perfil válido, salvar e reabrir | Preferências e jornada correspondem ao perfil correto | PENDENTE |
| C04 | Jornada truncada/incompatível/futura em cópia | Aviso recuperável; arquivo não sobrescrito silenciosamente | PENDENTE |
| C05 | Duas instâncias no mesmo perfil | Um escritor; segunda não corrompe nem rouba lock | PENDENTE |
| C06 | Simular falha de escrita em diretório de teste e recuperar permissão | Erro visível, retry limitado, dados anteriores preservados | PENDENTE |
| C07 | Encerrar abruptamente instância isolada e reabrir | Recupera último checkpoint válido; perda limitada ao intervalo não salvo, sem inventar progresso | PENDENTE |
| C08 | Testar migração nova com fixture rc7, quando implementada | Preserva seed/rota/progresso/serviço/desbloqueios; migração repetida não duplica dados | PENDENTE — expansão |
| C09 | Executar smoke curto com diagnóstico em arquivo novo | Encerramento normal e logs associados ao hash; não é teste de retomada de jornada | PENDENTE |

```powershell
$configInvalida = Join-Path $env:TEMP ('cpu-express-invalida-' + [guid]::NewGuid().ToString('N') + '.json')
[IO.File]::WriteAllText($configInvalida, '{ arquivo invalido')
$hashAntes = (Get-FileHash -LiteralPath $configInvalida -Algorithm SHA256).Hash
& .\cpu-express.exe --config $configInvalida
(Get-FileHash -LiteralPath $configInvalida -Algorithm SHA256).Hash -eq $hashAntes
```

O retorno `True` demonstra preservação dos bytes deste arquivo, não valida sozinho toda a recuperação. Manter evidências e fixtures; qualquer descarte segue as regras da pasta de trabalho.

## 9. Desempenho e conclusão

| ID | Procedimento | Critério de aceite | Resultado |
| --- | --- | --- | --- |
| P01 | Medir 30/60 FPS no mesmo hardware/perfil, painel fechado/aberto | Registrar duração, aquecimento, CPU do processo, memória e frames renderizados; separar custo de compositor | PENDENTE |
| P02 | Medir ativo/pausado/minimizado/oculto e retomada | Não renderizar continuamente quando oculto; sem avanço offline ou acúmulo de recursos | PENDENTE |
| P03 | Sessão prolongada proposta: duas horas, ciclos de menus/bandeja e clima | Sem crash, perda de perfil, janela inacessível ou crescimento persistente não explicado de memória/handles | PENDENTE |
| P04 | Repetir após expansão com mesma instrumentação | Regressões quantificadas e justificadas; meta de orçamento definida antes de aceitar novo conteúdo | PENDENTE |

Usar o [protocolo de medição](performance_windows.md) como método, sem herdar afirmações históricas de execução dele. As duas horas são um caso planejado, não resultado obtido.

Bloqueiam aceite: corrupção/perda indevida de dados, janela sem recuperação, crash, saída presa, avanço durante suspensão, foco roubado ou controles essenciais inacessíveis. Registrar ID, reprodução, ambiente, esperado/obtido, recuperação e hash. Classificar limitações de ambiente como NÃO EXECUTADO, sem escondê-las em aprovação agregada.

Entrega do relatório: tabela preenchida por ambiente, logs, capturas com escopo explícito, falhas abertas e recomendações. Assinatura, instalador, nuvem e aprovação completa da matriz de hardware **não estão concluídos por este checklist**.

## Evidência 0.4 em 2026-11-14

Smoke, painel independente, quatro alternâncias e dois cenários de 120 s passaram no Windows local. Migração de cópia do perfil e clima online também passaram. Detalhes e hash em [VALIDACAO_04.md](VALIDACAO_04.md). Isso não preenche automaticamente a matriz de DPI, outros monitores, reinício do Explorer ou dia completo.

Na rc.2, os mesmos cenários nativos foram repetidos e passaram. Migração agora cobre v2 → v3. O conector de interação nativa estava indisponível; cliques e compositor da central não foram certificados. Detalhes atuais em VALIDACAO_04.md.
