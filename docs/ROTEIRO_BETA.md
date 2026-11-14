# Beta CPU Express 0.4.0-rc.3 — teste de 10 minutos

Obrigado por testar! Queremos saber se o trem aparece, se os controles são claros e se sua viagem volta depois de fechar. Não precisa entender de programação.

## Abrir

1. Receba do Pedro o arquivo **CPU-Express-0.4.0-rc.3.zip**. Clique com o botão direito e escolha **Extrair tudo**. Use uma pasta onde você possa salvar arquivos, como Documentos. Não abra o jogo de dentro do ZIP nem separe o executável dos outros arquivos.
2. Para este beta, abra **Iniciar_DIAGNOSTICO.cmd** na pasta extraída. Ele inicia o jogo e grava um diagnóstico local. A viagem deste teste fica em `perfil-beta`, nessa mesma pasta; as próximas aberturas pelo iniciador retomam esse perfil. Deixe a janela de texto aberta até sair do jogo. Abra apenas uma sessão do beta por vez.
3. O uso normal também permite abrir **cpu-express.exe** diretamente. Nesse caso, o perfil fica em `%LOCALAPPDATA%\CPU Express`, separado do perfil deste roteiro, e o diagnóstico não é gravado automaticamente.

O executável não tem assinatura digital. O Windows pode exibir um aviso do SmartScreen de aplicativo ou editor desconhecido. Confira se o arquivo veio do Pedro. Se decidir continuar e as opções estiverem disponíveis, use **Mais informações → Executar assim mesmo**. Não desative o antivírus ou o SmartScreen. Se o computador da empresa bloquear, registre o aviso e interrompa o teste. Uma futura assinatura identifica o editor, mas não garante que o aviso desapareça. Mais detalhes em [Primeira abertura](windows_primeira_abertura.md).

## O que experimentar

| Tempo | Faça isto | Observe |
| --- | --- | --- |
| 0–2 min | Na acolhida, mantenha São Paulo ou informe a cidade e suas coordenadas; escolha faixa na barra e clique **Começar viagem**. | A cidade não é buscada pelo nome; não há GPS nem localização por IP. O trem deve aparecer. Se surgir aviso e janela flutuante, anote isso. |
| 2–4 min | Segure o meio da faixa e arraste para os lados. Depois arraste cada borda lateral para mudar a largura. | A faixa deve ficar dentro da barra; mais largura mostra mais paisagem. Um clique curto continua abrindo a central. |
| 4–6 min | Clique no trem, visite Trem, Viagem, Visual e Janela. Feche a central e abra novamente. Em Janela, experimente faixa e flutuante. | A central é separada do trem. Fechá-la não encerra o jogo. Conte se algo ficar cortado ou inacessível. |
| 6–7 min | Clique com o botão direito no trem para pausar; espere alguns segundos e clique novamente. | Trem e viagem devem parar e retomar sem salto. |
| 7–9 min | Use **Sair** na central ou na bandeja. Abra novamente **Iniciar_DIAGNOSTICO.cmd**. | A acolhida já confirmada não deve voltar; posição, tamanho e viagem devem ser preservados. O tempo com o jogo fechado não rende progresso. |
| 9–10 min | Anote o resultado no formulário abaixo e saia normalmente. | O diagnóstico fica pronto na pasta `diagnosticos`. |

Não existe botão do CPU Express na barra de tarefas. Para recuperar o trem oculto ou minimizado, procure seu ícone na bandeja, perto do relógio (talvez dentro da seta de ícones ocultos), e escolha **Mostrar trem**. Se não conseguir recuperá-lo, anote o que aconteceu; não precisa alterar o Windows.

Se sobrar tempo, teste **Cantos arredondados** e desative **Encaixar na altura da barra** para ajustar a borda superior. No Windows 10, os cantos podem permanecer retos. Altura maior cobre parte da área acima da barra. A linha Curitiba–Paranaguá ainda não está disponível para jogar; o percurso desta edição é o Circuito Aurora.

## Diagnóstico e envio

Ao sair, o iniciador mostra o nome do arquivo `diagnostico-….jsonl`. Abra a pasta **diagnosticos** e escolha o arquivo com data/hora do problema; se o problema ocorreu ao reabrir, inclua também o registro da sessão anterior. Envie os arquivos **manualmente ao Pedro**, pelo mesmo canal em que recebeu o beta, junto com o formulário. Não é preciso enviar a pasta inteira ou seus arquivos de viagem.

O diagnóstico é texto e inclui versão do jogo, métricas, estado da janela/viagem e a cidade e coordenadas configuradas para o clima. Você pode abri-lo no Bloco de Notas antes de compartilhar. O iniciador não envia o arquivo automaticamente. O jogo normal pode consultar o serviço de clima usando as coordenadas escolhidas.

Se nada abrir, envie o texto ou uma foto do aviso e diga se chegou a existir algum arquivo em `diagnosticos`. Capturas são opcionais: evite mostrar outros aplicativos ou informações pessoais.

## Formulário curto

Copie e preencha:

```text
CPU Express: 0.4.0-rc.3
Windows (10/11 e versão mostrada em Win+R → winver):
Escala da tela (Configurações → Sistema → Tela → Escala):
Número de monitores e em qual o jogo apareceu:
Posição da barra (baixo/cima/lado), ocultação automática ligada?:
Faixa apareceu ou houve fallback para janela flutuante?:
Arrastar e redimensionar funcionaram?:
Central e pausa funcionaram?:
Fechar e reabrir preservou a viagem e os ajustes?:
O que deu errado? O que você fez logo antes? O que esperava?:
Horário aproximado e nome do diagnóstico enviado:
```

Este beta coleta relatos; não significa que todas as combinações de Windows, escalas e monitores já foram testadas.
