# Aceitação manual: Windows 10/11

**Estado inicial: todos os ensaios abaixo estão PENDENTES.** Nenhum teste visual ou de shell Windows foi executado nesta sessão. Compilação cruzada, testes puros, Wine e um job de CI não substituem este registro. Preencha os resultados usando exatamente o executável cujo SHA-256 foi anotado.

## Preparação e identificação

Use um usuário comum, sem executar como administrador. Salve documentos abertos antes dos ensaios de reconexão de monitor ou reinício do Explorer. Não instale ferramentas para alterar a taskbar. A janela deve ficar na área útil do desktop, próxima à barra, sem reservar espaço do sistema.

Registre:

| Campo | Valor a preencher |
|---|---|
| Pessoa e data/hora | PENDENTE |
| Windows (edição, versão e build de `winver`) | PENDENTE |
| Commit/revisão do código | PENDENTE |
| SHA-256 de `cpu-express.exe` | PENDENTE |
| Configuração usada (`--config`) | PENDENTE |
| Monitor A: resolução, escala e posição | PENDENTE |
| Monitor B: resolução, escala e posição | PENDENTE |
| Monitor primário | PENDENTE |
| Borda da taskbar; auto-hide ligado/desligado | PENDENTE |
| Código principal ou exemplo de integração | PENDENTE |
| Resultado geral; evidências/logs | PENDENTE |

Comandos na pasta do executável:

```powershell
Get-FileHash -LiteralPath .\cpu-express.exe -Algorithm SHA256
Start-Process -FilePath .\cpu-express.exe -ArgumentList '--config', '.\windows-acceptance.json'
```

Use somente uma instância durante os ensaios. A configuração de aceitação é um arquivo novo local, para não misturar preferências da sessão cotidiana. O núcleo segue usando CPU real; `--cpu` não é necessário para estes testes.

## Matriz mínima de ambientes

Execute pelo menos um ambiente Windows 10 e um Windows 11. Em cada sistema, cubra dois monitores e pelo menos duas escalas diferentes. Complete as quatro escalas abaixo quando o equipamento permitir. Não marque como aprovado um caso indisponível: use **N/A** e descreva a limitação.

| ID | Ambiente | Configuração | Estado |
|---|---|---|---|
| M01 | Windows 10 | Um monitor, 100%, taskbar inferior visível | PENDENTE |
| M02 | Windows 10 | Um monitor, 125%, taskbar inferior visível | PENDENTE |
| M03 | Windows 10 | Dois monitores, A 100% e B 150%, B à esquerda (x negativo) | PENDENTE |
| M04 | Windows 10 | B acima de A (y negativo), B em 200% | PENDENTE |
| M05 | Windows 11 | Um monitor, 100%, taskbar inferior visível | PENDENTE |
| M06 | Windows 11 | Dois monitores, A 125% e B 200%, B à esquerda | PENDENTE |
| M07 | Ambos | Taskbar em ocultação automática, janela próxima à borda | PENDENTE |
| M08 | Sistema que oferece a opção nativa | Taskbar à esquerda, à direita e no topo | PENDENTE |
| M09 | Ambos | Trocar o monitor primário sem encerrar o app | PENDENTE |
| M10 | Ambos | Desconectar/reconectar o monitor que contém a janela | PENDENTE |

Só mova a taskbar para bordas que o sistema em teste permita nativamente. Cobertura matemática de bordas não prova que uma versão específica do Windows ofereça a opção. Se não houver duas máquinas, registrar a cobertura real e deixar os demais ambientes pendentes.

## W1: geometria, escala e foco

Repita W01–W06 nas configurações relevantes da matriz; registre cada combinação em uma linha de resultado.

| ID | Procedimento reproduzível | Resultado esperado | Estado |
|---|---|---|---|
| W01 | Abrir o app. Conferir quatro limites da janela e a barra. | Janela inteira dentro da área útil; não cobre a taskbar reservada. | PENDENTE |
| W02 | Arrastar para cada monitor e usar “Reposicionar na área útil”. | Usa o monitor que contém/está mais próximo da janela; preserva coordenadas negativas válidas. | PENDENTE |
| W03 | Alternar escala de 100% para 150% e 200%, aguardar o Windows aplicar e reposicionar. | Margem proporcional; posição física correta; sem dupla multiplicação por DPI. | PENDENTE |
| W04 | Usar resolução pequena; se não reduzir a área abaixo da janela, reservar espaço com recurso normal do ambiente de laboratório. | Janela cabe na área disponível; tamanhos positivos; sem travar. Layout de controles deve ser avaliado separadamente pela frente UX. | PENDENTE |
| W05 | Deixar o Bloco de Notas recebendo texto. Iniciar o app por um comando com atraso de 5 s e voltar ao editor antes do lançamento. Continuar digitando. | A abertura automática não toma o teclado do editor. Não usar o clique de lançamento como prova de foco. | PENDENTE |
| W06 | Após o início, manter o editor ativo e provocar uma mudança de resolução/monitor pelo mecanismo de laboratório. Observar o app reagir quando a integração de eventos estiver ligada. | Ajustes automáticos não ativam a janela do trem; sem ciclo contínuo de reposicionamento. | PENDENTE |
| W07 | Arrastar a janela livremente e esperar 30 s. | Não é puxada de volta a cada frame. Reencaixe ocorre apenas por pedido/evento documentado. | PENDENTE |
| W08 | Ativar auto-hide e levar o mouse à borda da barra; depois retirar o mouse. | Barra continua acessível e o trem não desaparece. A margem não representa reserva permanente para a barra emergente. | PENDENTE |
| W09 | Trocar primário ou desconectar o monitor do trem. Usar a recuperação documentada. | Janela pode ser recuperada em monitor disponível, sem coordenadas inválidas. | PENDENTE |
| W10 | Minimizar pelo botão atual e restaurar pela barra/Alt+Tab. | Minimização e restauração continuam funcionais; trem retoma sem grande salto de tempo. | PENDENTE |
| W11 | Alternar “No topo” e depois trabalhar no editor. | Política de sobreposição muda; a alternância automática não rouba foco. Ação explícita com clique pode alterar o foco normalmente. | PENDENTE |

Exemplo para W05 (execute, depois retorne ao editor antes de cinco segundos):

```powershell
Start-Sleep -Seconds 5
Start-Process -FilePath .\cpu-express.exe -ArgumentList '--config', '.\windows-acceptance.json'
```

W04 não requer alterar o registro nem instalar um componente de shell. Se o equipamento não permite produzir uma área suficientemente pequena, anote essa limitação; os testes puros cobrem áreas diminutas, mas não aprovam o layout real.

## W2: bandeja e transições de estado

**Pré-condição:** usar o exemplo de integração da plataforma ou aplicar a ligação mínima descrita em `INTEGRACAO.md` em uma cópia de teste. O aplicativo baseline não ganha controles de bandeja apenas por compilar o módulo novo. Sem essa pré-condição, marque os itens como “PENDENTE — integração UX” e não como aprovados.

A demonstração está em `examples/windows_desktop.rs`. Na raiz do código-fonte Windows:

```powershell
cargo run --locked --example windows_desktop
```

O artefato de CI inclui o mesmo exemplo compilado como `windows_desktop.exe`; execute-o diretamente e anote também seu SHA-256. Esse exemplo testa a plataforma; as regressões do jogo devem usar `cpu-express.exe`. Não atribua ao jogo uma funcionalidade observada somente no exemplo.

| ID | Procedimento reproduzível | Resultado esperado | Estado |
|---|---|---|---|
| T01 | Abrir o exemplo/app integrado; localizar o ícone inclusive na área de ícones ocultos do Windows. | Exatamente um ícone pertencente à instância, menu e tooltip acessíveis. | PENDENTE |
| T02 | Selecionar “Ocultar” e depois “Mostrar” pela bandeja, dez vezes. | Ocultar é diferente de minimizar; cada ocultação tem recuperação, sem criar novas janelas/threads. | PENDENTE |
| T03 | Minimizar, depois “Mostrar” pela bandeja. | Recupera a janela; estado interno acompanha o estado observado. | PENDENTE |
| T04 | Usar reposicionamento e alternância de topo pelo menu da bandeja. | Eventos chegam à thread da UI; cena e métricas continuam responsivas. | PENDENTE |
| T05 | Abrir o menu com botão direito; navegar por teclado; pressionar Esc. | Menu fecha; nenhuma ação acidental; não ficam menus órfãos. | PENDENTE |
| T06 | Ocultar a janela, abrir Gerenciador de Tarefas, selecionar “Windows Explorer” e “Reiniciar”. | Ícone é registrado novamente; se isso falhar, evento de indisponibilidade recupera a janela conforme integração. | PENDENTE |
| T07 | Repetir reinício do Explorer com a janela visível. | Sem duplicação permanente de ícones; app continua acessível. | PENDENTE |
| T08 | Em uma sessão de laboratório, simular falha de criação/registro do tray pelo ponto de teste indicado na documentação. Solicitar ocultação. | Falha é apresentada; a janela permanece visível/recuperável. Não encerrar o Explorer da sessão de trabalho apenas para criar esta condição. | PENDENTE |
| T09 | Ativar auto-hide, ocultar o trem, revelar taskbar e mostrar pela bandeja. | Ícone permanece alcançável e janela pode ser recuperada. | PENDENTE |
| T10 | Selecionar “Sair” com janela visível, minimizada e oculta, em execuções separadas. | Cada execução encerra completamente; nenhum caso converte sair em ocultar. | PENDENTE |

Uma falha de recuperação em T02/T06/T08/T09 bloqueia a habilitação de “Ocultar” no produto. Retorno de erro de API não prova sozinho que o estado foi recuperado: observar a janela e o processo. T08 é ensaio de desenvolvedor; a forma exata de induzir a falha deve ser registrada, sem fingir que uma falha de `Shell_NotifyIconW` foi executada se houve apenas teste da máquina de estados.

## W3: encerramento, recursos e regressão

| ID | Procedimento reproduzível | Resultado esperado | Estado |
|---|---|---|---|
| E01 | Fechar por ×; repetir com Esc e Alt+F4 em novas execuções. | Processo encerra; `MetricsWorker` e tray concluem limpeza. | PENDENTE |
| E02 | Durante a execução, registrar contagem de handles/threads. Fazer 20 ciclos mostrar/ocultar/menu, aguardar 5 s e registrar novamente. | Sem crescimento monotônico por ciclo; registrar números, sem impor um limite inventado. | PENDENTE |
| E03 | Fechar com menu aberto ou logo após selecionar ação; repetir três vezes. | Sem deadlock, crash ou processo residual. | PENDENTE |
| E04 | Após “Sair”, aguardar 5 s e consultar o processo. Passar o mouse pela região da bandeja. | Sem `cpu-express` em execução e sem ícone ativo órfão. | PENDENTE |
| E05 | Abrir sem opção manual, observar CPU/movimento e indicadores. | CPU real continua usada; UI não fica bloqueada enquanto coleta. | PENDENTE |
| E06 | Executar `--smoke-seconds 10` e aguardar encerramento. | Saída automática continua válida; as APIs novas não quebram opções anteriores. | PENDENTE |

Comandos auxiliares:

```powershell
Get-Process -Name cpu-express -ErrorAction SilentlyContinue |
  Select-Object Id, Handles, @{Name='Threads'; Expression={$_.Threads.Count}}, CPU, WorkingSet64
Start-Process -FilePath .\cpu-express.exe -ArgumentList '--config', '.\windows-acceptance.json', '--smoke-seconds', '10' -Wait
Get-Process -Name cpu-express -ErrorAction SilentlyContinue
```

`CPU` nesse comando é tempo acumulado de CPU do processo, não porcentagem. Nenhum ensaio exige estressar a máquina. Tempos de saída devem ser medidos; se ainda existir processo depois de 5 s, registrar a falha e investigar a consulta de métricas ou chamada de shell em andamento. Encerrar forçadamente pelo Gerenciador de Tarefas é uma recuperação de laboratório, não um resultado aprovado.

## Registro de evidência e regra de aceite

Para cada execução, registre: ID do caso + ambiente Mxx; ação; esperado; observado; aprovado/falhou/N/A; horário; nome do screenshot/log quando existir. Capture somente a área necessária do aplicativo e da barra, evitando documentos pessoais. Uma screenshot comprova apenas o estado mostrado; teste de foco, recuperação e cleanup precisa também de sequência/observação.

Casos bloqueantes: janela fora da área útil; foco roubado ao abrir automaticamente; janela oculta sem recuperação; saída que deixa processo; crash ou thread presa; uso alto e contínuo causado por polling da plataforma. Falhas de layout em área reduzida devem ser enviadas ao Agente 05 com DPI, tamanho físico e imagem. Itens não executados continuam pendentes e devem acompanhar o relatório da versão.

