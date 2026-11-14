# CPU Express 0.3 — funcionamento e decisões

## Uma ferrovia tranquila no computador

O CPU Express tem dois ritmos. A velocidade visual da locomotiva responde à CPU real, com aceleração suavizada; o itinerário avança a 24 unidades fictícias por segundo ativo. Assim, trabalhar com programas mais pesados muda a animação, mas não facilita os objetivos. Não é preciso aquecer o computador para jogar.

O Circuito Aurora liga Vila Aurora, Bosque dos Pinheiros, Serra Clara e Porto do Sol. Os trechos duram aproximadamente 70–117 segundos, de acordo com a seed da rota. Perto da estação o trem freia. Cada chegada abre oito segundos de embarque/desembarque, com plataforma e passageiros desenhados pelo programa. Depois o serviço parte automaticamente.

Pausa, minimização e ocultação congelam a viagem. Retornar ou reabrir não concede tempo offline. O contador de chegada é o marco de objetivos; não existe saldo de passageiros, comércio ou economia implícita na animação de embarque.

## Objetivos e personalização

| Objetivo | Como completar | Recompensa |
| --- | --- | --- |
| Primeiro serviço | Uma chegada | Pintura Azul do Porto |
| Cartão de explorador | Quatro chegadas | Pintura Cobre do Poente |
| Uma ferrovia de confiança | Doze chegadas | Pintura Expresso Noturno |

Verde Aurora está disponível desde o começo. Escolha as pinturas liberadas em **Visual**. A demonstração permite experimentar todas. As recompensas são derivadas dos contadores persistidos, portanto reler um evento ou reabrir o painel não duplica prêmios. Os cinco modelos de vagão anteriores continuam disponíveis individualmente.

Em **Viagem**, consulte a próxima parada, o tempo estimado, serviço na plataforma, objetivos, descobertas, conquistas e diário das últimas 32 mensagens. O diário é uma conveniência da sessão; a jornada e os desbloqueios persistem.

## Faixa e painel

Na rc.2, o modo principal usa uma janela independente exatamente no retangulo da barra horizontal primaria, alinhada a esquerda e dimensionada pela altura fisica da barra. Ela fica sobre a area da barra, sem ocupar uma faixa acima dela. O encaixe interno como filha do Explorer foi removido por falha de exibicao reportada no desktop real. Nao ha reserva de espaco: ajuste o deslocamento se houver widgets. Os controles maiores abrem em outra janela nativa, movel e redimensionavel.

- Clique na faixa: abrir/fechar a central.
- Botão direito na faixa: pausar/retomar.
- Na central: Trem, Viagem, Visual, Janela e ajuda.
- Em Janela, desmarcar a integração retorna ao modo flutuante de 640 × 140.
- Esc fecha o painel. Ctrl+Q ou Encerrar sai do aplicativo.
- A bandeja oferece recuperação, posicionamento e saída.

A faixa usa a parte esquerda existente; não reserva espaço no layout privado do Explorer. Widgets e botões alinhados à esquerda podem disputar essa região. A integração com barras verticais, monitores secundários e diferentes substitutos do Explorer ainda não é uma variante oferecida. Se a barra não puder ser encontrada ou tiver geometria incompatível, a aplicação tenta retornar ao modo flutuante e comunica o motivo.

## Salvamento

As preferências continuam em `%LOCALAPPDATA%\CPU Express\config.json`. A jornada usa `config.journey.json`, no mesmo diretório. Um `--config` alternativo produz um save correspondente ao nome desse arquivo.

O checkpoint guarda versão, identidade da rota, seed, distância lógica, tempo ativo e segundos restantes de serviço. Visitas, descobertas e recompensas são reconstruídas em tempo constante, sem executar novamente os eventos antigos. A aparência selecionada fica nas preferências e é validada contra o progresso na abertura.

O jogo salva a cada 30 segundos ativos de execução e na saída normal; o botão **Salvar viagem agora** permite antecipar. Uma interrupção abrupta pode perder o intervalo desde o último salvamento. Arquivos inválidos ou de versões futuras são preservados, com aviso, e não substituídos silenciosamente.

A escrita usa temporário no mesmo diretório, flush/sync e substituição do destino. Falhas de preferências mantêm o estado pendente e retentam com intervalo de cinco segundos; a saída faz uma última tentativa. O perfil tem bloqueio exclusivo no Windows, liberado pelo sistema quando o processo termina. O arquivo `.profile.lock` vazio que permanece no disco não significa que exista um bloqueio ativo.

DEMO e execuções com encerramento automático não carregam nem gravam a jornada normal. Não alteram os objetivos do perfil de trabalho.

## O que veio das pesquisas

Foram inventariados os pacotes de entrada e consolidados 96 achados de A1, A2, A3, A4 e A6. `pesquisa/` na raiz guarda os documentos extraídos e os índices JSON. O APK recebido é um fragmento, não o aplicativo completo; a única DEX descrita nos relatórios pertence ao SDK de anúncios. As pesquisas são evidência estática, não código-fonte recuperado do gameplay.

Aplicações concretas nesta versão: separar ritmo visual/progressão (A2/A3), catálogos com IDs explícitos e objetivos claros (A3), não duplicar recompensas no restart (A3), geometria fixa de rodas e sprites autorais (A4), persistência com retry e estados de pausa previsíveis (A6). A base rc.1 já continha a correção da mailbox de métricas, por isso o patch antigo não foi reaplicado.

Não foram incorporados sprites, músicas, tabelas de balanceamento, mapa ou bibliotecas do APK. A plataforma, passageiros e pinturas novos são procedurais e autorais.

## Próximos avanços por área

1. **Plataforma:** matriz de aceitação com DPI 100/125/150/200%, autohide, reinício do Explorer, múltiplos monitores e conflitos com widgets; encaixe configurável e variantes direita/vertical depois dessa validação.
2. **Jogo:** escolhas de serviço por estação, passageiros/cargas com estado próprio e consequências visíveis; manter objetivos acessíveis sem CPU elevada, sem prazos punitivos e sem confundir animação com economia implementada.
3. **Mundo:** grafo pequeno com ramificações autorais, catálogo versionado e migração de saves antes de mudar o circuito existente.
4. **Arte:** locomotivas adicionais com sockets explícitos, sem deslocar rodas/engates, e sons discretos opcionais com autoria documentada.
5. **Distribuição:** assinatura, instalador opcional e aceitação prolongada. Esta entrega é candidata de testes, não alegação de que todo o checklist Windows já foi cumprido.


## Ajustar posicao e tamanho (rc.3)

Clique no trem e abra Janela. Na area da barra, ajuste Posicao horizontal. Desmarque Tamanho automatico da barra para escolher largura e altura em pixels fisicos; a altura fica limitada a barra e o deslocamento e limitado para manter a faixa dentro dela. A imagem mantem pixels inteiros e pode ganhar margens ou recorte conforme a proporcao escolhida.

Para posicionamento livre, desmarque Na area da barra de tarefas. Use X/Y, Largura e Altura ou arraste o titulo e as bordas da janela. O retangulo flutuante fica salvo separadamente, inclusive coordenadas negativas de outros monitores. Ao restaurar, o programa limita o retangulo a area util do monitor mais proximo para evitar janelas perdidas. Aguarde um segundo apos mover para registrar a geometria. Restaurar posicao e tamanho retorna aos valores iniciais.


RC4: a alternancia de modo espera a atualizacao das bordas antes de restaurar a geometria. A paisagem fica limitada a area visivel mesmo se a UI conservar um tamanho minimo antigo. O retangulo flutuante nao e sobrescrito durante a etapa intermediaria.


## Arte, menus e fluidez (rc.5)

A locomotiva recebeu volume na caldeira, cintas e placa; cada tipo de vagão ganhou acabamentos próprios. Estações usam arquitetura por bioma, iluminação e passageiros animados. Pinheiros e construções têm sombras mais legíveis, e a fumaça tem volumes sobrepostos. Tudo continua autoral e procedural, na grade 320 × 70, sem importar arte do APK.

A central agora apresenta prévia ao vivo, cartões de velocidade/CPU/RAM, ação principal de pausa e navegação por abas. Visual agrupa pinturas com amostras de cor, composição, fluidez e atmosfera. Viagem usa cartões de objetivos; detalhes de estações e conquistas ficam recolhidos. O rodapé mantém Fechar painel e Sair acessíveis. O painel compacto é preservado apenas em backends sem janelas independentes.

Fluido (60 FPS) é o padrão de perfis novos; Econômico (30 FPS) e o ajuste fino continuam em Visual → Fluidez. Perfis existentes preservam o FPS gravado. O agendamento agora mantém uma cadência fixa e salta apresentações perdidas, sem acumular atraso a cada frame ou executar rajadas para compensá-lo. A progressão lógica continua independente da CPU/FPS, pausa congela a viagem e não existe progresso offline. A nitidez da grade de pixels foi preservada: não há interpolação borrada.

A comparação de cadência usa execuções curtas locais com o mesmo limite de 60 FPS e sem painel aberto. Ela mede frames produzidos pelo aplicativo, não a taxa de apresentação do compositor ou um benchmark de consumo de CPU/GPU. Os dados ficam em evidence/current/visual-refresh/comparison.json. Os testes da alternância barra/flutuante são repetidos nesta versão.

## Atmosfera local (rc.6)

São Paulo, SP, Brasil fornece o clima da paisagem, independentemente do bioma fictício da rota. A consulta Open-Meteo usa a posição aproximada do centro da cidade e roda fora da thread de interface. O painel Visual informa temperatura, condição, horário do dado e estado da atualização. A iluminação acompanha dia/noite local; animação e progressão continuam usando tempo ativo do jogo. Sem conexão, o painel distingue dados anteriores de cenário ilustrativo. Clássico e detalhes reduzidos foram retirados do produto.

## Caderno da ferrovia (rc.7)

A navegação mantém Trem, Viagem, Visual, Janela e Ajuda, agora com ícones próprios e controles em relevo. A cabine destaca velocidade, CPU, memória e pausa. O diário inclui um mapa fechado das quatro estações e barras de objetivos com ações para encontrar as pinturas liberadas. A oficina separa vagões, pinturas, clima e fluidez; os cartões mostram exatamente os modelos e cores do trem. As mudanças são persistidas pelo mesmo mecanismo protegido das versões anteriores.

A janela flutuante dos menus abre em 760 × 720 pontos e admite 480 × 480. A área rolável preserva navegação e ações de fechamento. A faixa do trem conserva sua geometria independente. Ícones e miniaturas são originais e locais, sem nova consulta de rede ou dependência externa.

O diário separa Mapa, Objetivos e Diário em páginas próprias. O mapa mostra o trecho atual e leva às recompensas. A oficina omite a prévia panorâmica repetida para priorizar posições e cartões; o trem no desktop acompanha cada escolha.
