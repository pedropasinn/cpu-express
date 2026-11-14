# CPU Express — guia de uso da frente de UX

O CPU Express mantém uma pequena paisagem com um trem perto da barra de tarefas. A janela continua com 640 × 140 pontos; os controles, os painéis e as informações sobre a viagem cabem nessa área. A aplicação fica em uma janela própria, e não dentro da barra de tarefas.

Este guia documenta os controles desta entrega do Agente 05. O registro do que foi efetivamente compilado, testado e executado está em [TESTES.md](../TESTES.md). Os comandos e o roteiro de conferência abaixo são instruções de uso; não constituem evidência de execução em Windows.

## 1. Começar no Windows

Se você recebeu o código-fonte e tem a cadeia Rust indicada pelo projeto, abra o PowerShell na pasta que contém `Cargo.toml`:

```powershell
cargo build --release --locked
.\target\release\cpu-express.exe
```

Se estiver usando um pacote com o executável já compilado, abra `cpu-express.exe` na pasta desse pacote. Esta frente preserva o alvo de produto Windows 10/11 x64 e as dependências do projeto original; ela não representa, por si só, a certificação de um novo release Windows.

A primeira leitura real de CPU precisa do intervalo inicial de coleta. Enquanto ela não chega, a interface indica que está aguardando. Arraste uma área livre da paisagem para mover a janela; os botões e os painéis têm suas próprias interações.

Para experimentar apenas o comportamento do trem:

```powershell
.\target\release\cpu-express.exe --demo
.\target\release\cpu-express.exe --demo --cpu 0
.\target\release\cpu-express.exe --demo --cpu 100
```

Em um pacote binário, substitua `.\target\release\cpu-express.exe` por `.\cpu-express.exe` nos exemplos.

## 2. CPU real, CPU manual e DEMO

| Modo | Como iniciar | O que movimenta o trem | O que acontece com as métricas reais |
| --- | --- | --- | --- |
| Real | Sem opções de simulação | Utilização real da CPU global | CPU e RAM são coletadas em segundo plano |
| CPU manual | `--cpu 50`, ou simulação pelo painel | Porcentagem escolhida por você | A coleta real continua; o número manual é identificado como simulado |
| DEMO | `--demo`, com `--cpu` opcional | Porcentagem simulada; começa em 50% | Não inicializa o coletor de CPU/RAM real; RAM permanece indisponível |

**DEMO e CPU manual não geram carga artificial no processador.** Eles mudam a entrada da simulação. Em DEMO, o painel permite escolher outra porcentagem, mas a sessão continua isolada das métricas reais. Para usar CPU real depois, encerre a aplicação e abra novamente sem `--demo`.

Com os parâmetros originais do projeto, os alvos de velocidade são:

| CPU escolhida ou medida | Velocidade-alvo |
| --- | --- |
| 0% | 12 km/h |
| 50% | 56 km/h |
| 100% | 100 km/h |

A velocidade se aproxima do alvo suavemente. O valor apresentado durante a transição não precisa coincidir imediatamente com o alvo. Esses números são uma representação da viagem; não medem a velocidade física do computador.

## 3. Controles e informações sobre a paisagem

Pressione **M** para alternar na ordem **compacto → completo → limpo → compacto**. HUD é o nome dado às informações desenhadas sobre o cenário.

- **Compacto:** conserva as informações essenciais com uma área menor sobre a paisagem.
- **Completo (detalhado):** apresenta mais informações sobre CPU, RAM e viagem.
- **Limpo:** reduz as informações sobre a paisagem. Identificações necessárias, como DEMO, CPU simulada, pausa e ausência de métricas, continuam disponíveis; ocultar o HUD não transforma dados simulados em reais.

Os controles podem se recolher quando o ponteiro está fora da janela e ela não tem foco de teclado. Voltar com o ponteiro ou dar foco à janela torna os controles acessíveis novamente. A opção de recolhimento automático pode ser desligada nas preferências.

Os rótulos usam superfícies próprias para permanecerem legíveis sobre a paisagem. Passe o ponteiro sobre um botão para consultar sua descrição. A navegação por teclado permite alcançar controles sem arrastar a janela.

## 4. Painéis de opções e ajuda

**O** abre ou fecha as opções. O painel tem três seções, com rolagem para alcançar os controles dentro da janela compacta:

| Seção | O que ajustar |
| --- | --- |
| **Viagem** | Pausa, CPU manual, presets de 0%, 50% e 100%, ajuste da porcentagem e reposicionamento |
| **Visual** | Modo do HUD, FPS, resposta de velocidade e recolhimento automático dos controles |
| **Janela** | Sempre no topo, minimização e preferências de pausa |

O controle manual permite passos de **0,1 ponto percentual**. O limite de animação fica entre **10 e 60 FPS**. A resposta de velocidade aceita valores de **0,05 a 20**: valores maiores fazem o trem se aproximar mais rapidamente da nova velocidade-alvo. O padrão é 1,8.

**F1** abre ou fecha a ajuda. Use a rolagem para ler as instruções que não cabem de uma vez. **Esc** fecha o painel aberto. Se não houver painel, Esc não encerra a aplicação.

## 5. Pausar sem fazer o trem saltar

**Espaço** alterna pausa e retomada. Durante a pausa, a simulação da viagem e o deslocamento da paisagem ficam congelados. A coleta de métricas pode continuar, portanto os indicadores de CPU/RAM podem mudar enquanto o trem permanece parado.

O tempo passado em pausa não se acumula como distância a percorrer depois. Ao retomar, a viagem continua do ponto em que parou.

Há duas preferências independentes:

- **Iniciar pausado:** abre a próxima sessão com a viagem parada. O padrão é desligado.
- **Pausar sem foco:** interrompe a viagem quando você trabalha em outra janela. O padrão é desligado, para o trem poder acompanhar o trabalho em segundo plano.

A pausa manual e a pausa automática por falta de foco são situações distintas. Voltar à janela não deve desfazer uma pausa escolhida manualmente.

## 6. Atalhos

Os atalhos pertencem à janela do CPU Express; não há captura global de teclas. Comandos de aplicação não devem interferir na edição de um campo ou no uso de um controle que já possui foco.

| Tecla | Ação |
| --- | --- |
| **Espaço** | Pausar ou retomar |
| **M** | Alternar HUD compacto, completo e limpo |
| **P** | Alternar sempre no topo |
| **O** | Abrir ou fechar as opções |
| **F1** | Abrir ou fechar a ajuda |
| **R** | Reposicionar a janela perto da barra de tarefas |
| **Esc** | Fechar o painel aberto; nunca sair sozinho |
| **Ctrl+Q** | Encerrar deliberadamente, fora da edição de um campo |
| **Tab / Shift+Tab** | Percorrer os controles disponíveis |

O botão **Sair** também encerra a aplicação. Para minimizar, use o controle da janela; a restauração ocorre pela barra de tarefas do Windows. O comando de bandeja permanece desabilitado nesta frente, aguardando a interface real do Agente 01. Não há opção ativa que esconda a janela sem oferecer um caminho de volta.

## 7. Preferências e arquivos antigos

No Windows, o caminho padrão continua sendo:

```text
%LOCALAPPDATA%\CPU Express\config.json
```

A variável `CPU_EXPRESS_CONFIG` pode definir outro caminho. A opção `--config` tem prioridade sobre ela. Para uma experiência isolada:

```powershell
.\target\release\cpu-express.exe --demo --config ".\preferencias-demo.json"
```

Arquivos JSON antigos ou parciais continuam válidos: os campos ausentes recebem os padrões. Exemplo de arquivo parcial com preferências desta frente:

```json
{
  "fps": 30,
  "smoothing_k": 1.8,
  "hud_mode": "compact",
  "show_metrics": true,
  "always_on_top": true,
  "auto_hide_controls": true,
  "pause_when_unfocused": false,
  "start_paused": false
}
```

`hud_mode` aceita `compact`, `detailed` ou `clean`. O campo antigo `show_metrics` continua sendo respeitado: `false` mantém o HUD limpo mesmo se `hud_mode` tiver outro valor. Escolher um modo pela interface atualiza os dois campos de maneira consistente.

Ao carregar, números fora dos limites são ajustados em memória e a aplicação mostra um aviso. Ler a configuração não regrava automaticamente o arquivo. Ao salvar uma alteração explícita, os campos desconhecidos de um documento válido são preservados, inclusive valores aninhados; isso evita apagar preferências pertencentes a outras frentes.

**Um arquivo corrompido, incompatível ou ilegível não é sobrescrito com os padrões.** A sessão pode continuar com valores seguros, mas a interface informa se não conseguiu salvar. Corrija o arquivo original ou escolha um novo caminho com `--config`. A gravação usa arquivo temporário e substituição atômica; uma falha de gravação mantém o arquivo anterior.

O modo DEMO, a porcentagem manual da sessão e as métricas coletadas não se tornam preferências persistentes por esse mecanismo. O registro de métricas exige a opção explícita de diagnóstico abaixo.

## 8. Comandos para uma verificação curta

```powershell
# DEMO por dez segundos, sem coletar CPU/RAM reais.
.\target\release\cpu-express.exe --demo --cpu 50 --smoke-seconds 10

# CPU real com diagnóstico local solicitado explicitamente.
.\target\release\cpu-express.exe --smoke-seconds 10 --diagnostics ".\diagnostico.jsonl"

# Forma curta de encerramento automático após três segundos.
.\target\release\cpu-express.exe --demo --smoke

# Ajuda sem abrir a janela nem inicializar os arquivos indicados.
.\target\release\cpu-express.exe --help
```

`--smoke-seconds` aceita de 0.1 a 86400 segundos. `--smoke` é um encerramento programado da sessão, e não um teste que aumenta a carga da CPU. `--diagnostics` grava JSONL no caminho solicitado; não envia métricas pela rede. Escolha um arquivo diferente das preferências: o diagnóstico substitui o arquivo de saída solicitado, e a CLI rejeita destinos que coincidam com a configuração pelo caminho resolvido antes de abri-lo.

`--help` e `-h` têm o mesmo efeito. O executável usa o subsistema GUI do Windows; a exibição direta da ajuda no PowerShell depende da forma de execução/captura da saída. A ajuda F1 continua disponível dentro da aplicação.

Se uma métrica estiver ausente ou sem leitura recente, a interface sinaliza essa situação sem bloquear a janela. Um valor indisponível não deve ser interpretado como uma medição de 0%. DEMO nunca apresenta RAM fictícia como uma leitura real.

## 9. Investigação visual em Linux

A funcionalidade `host-validation` serve para investigar a interface no ambiente de desenvolvimento. Ela não implementa o produto Linux nem comprova a integração da janela com o Windows:

```bash
cargo run --locked --features host-validation -- --demo --cpu 50 --smoke-seconds 10 --config ./preferencias-investigacao.json
```

Esse comando precisa de um ambiente gráfico compatível. Resultados, screenshots e eventuais restrições do ambiente devem ser registrados com a plataforma em que ocorreram. O posicionamento perto da barra de tarefas, a restauração, o DPI e os vários monitores continuam exigindo conferência em Windows real.

## 10. Conferência manual no Windows

Antes de considerar a experiência nativa validada, registre os resultados de uma sessão real:

1. Abra no tamanho padrão e confira os três modos do HUD, incluindo os marcadores de simulação no modo limpo.
2. Abra opções e ajuda; percorra as seções e a rolagem. Confirme que Esc fecha o painel e mantém a aplicação aberta.
3. Use os presets 0/50/100 e ajuste uma casa decimal. Confirme que a indicação de CPU manual permanece visível.
4. Pause, espere alguns segundos e retome. Confira que cenário, distância e animações não avançam para compensar a espera.
5. Teste os controles com Tab e confirme que Espaço e letras não disparam comandos indevidos durante a edição de um campo.
6. Minimize e restaure pela barra de tarefas. Teste sempre no topo, reposicionamento e arraste em área livre.
7. Altere uma preferência, feche e abra novamente. Confirme a persistência e a preservação de campos extras no JSON.
8. Execute DEMO e confirme a ausência de RAM real e a impossibilidade de trocar a sessão para o coletor real. Registre separadamente qualquer verificação técnica de ausência de acesso aos contadores.

Nenhum item desta lista está marcado como executado neste guia. Consulte [TESTES.md](../TESTES.md) para os resultados e [INTEGRACAO.md](../INTEGRACAO.md) para a conexão com as outras frentes. A interface `LaunchOptions` e as assinaturas existentes dos módulos compartilhados são preservadas.
