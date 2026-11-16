# CPU Express 0.4.0-rc.3 — serviços e escolhas

**Desenvolvimento na branch expansao:** a continuação E3 torna Curitiba–Paranaguá jogável com 14 paradas e troca em Viagem → Linhas. O checkpoint atual da fonte é v6, com backups exatos v1–v5. Veja [LINHAS_E3.md](expansao/LINHAS_E3.md) para embarques, troca, missões e persistência. O restante deste guia registra a distribuição rc.3 anterior.

Esta edição amplia a ferrovia mantendo o encaixe à esquerda, o modo flutuante e os menus independentes. A viagem usa tempo ativo, não a carga de CPU. Pausa, ocultação e fechamento não rendem progresso offline.

## Começar

Abra `cpu-express.exe` do portátil rc.3. Clique no trem para abrir o caderno. O jogo não tem botão na barra de tarefas; para mostrar o trem oculto ou minimizado, use **Mostrar trem** no ícone da bandeja (inclusive na seta de ícones ocultos). **Trem** mostra destino e pausa; **Viagem** reúne mapa, objetivos, diário, serviços e coleção; **Visual** contém vagões, pinturas, clima, fluidez e motores. A janela do caderno guarda sua posição e tamanho separadamente da faixa.

## Passageiros e composição

Os grupos têm estação de origem, destino e quantidade. Em cada chegada, o jogo desembarca quem chegou ao destino e embarca novos grupos até a capacidade. Os vagões de passageiros oferecem 24 lugares, panorâmicos 16 e dormitórios 12. Os modelos também diferem visualmente; não há bônus oculto de conforto ou turismo nesta edição.

Escolha a posição no desenho da composição e equipe um vagão. Uma alteração que não comporta os passageiros ou a carga atuais é recusada. Aguarde a entrega antes de reduzir capacidade. É possível guardar até quatro composições favoritas e reaplicá-las quando houver capacidade. A oficina mostra a capacidade antes/depois para cada opção e favorito; ao passar o mouse ou focar, explica ocupação e categoria incompatível; **Viagem → Serviços** mostra a ocupação e os destinos reais.

## Encomendas

Por padrão, a ferrovia aceita automaticamente um contrato por vez, sem prazo. Em **Viagem → Serviços**, desative **Despacho automático de encomendas** para analisar a oferta. A mudança mantém o contrato atual; novas ofertas aguardam sua escolha. Há uma oferta por vez, sem prazo nem mercado de múltiplos contratos.

Encomendas e suprimentos usam vagões de carga (20 unidades por vagão); líquidos usam tanques (12). Antes de aceitar, equipe capacidade compatível. Se estiver na plataforma de origem, o carregamento acontece no aceite; em trânsito, aguarda a próxima passagem pela origem. Entrega continua automática no destino. Trocas posteriores de composição podem fazer um contrato aceito aguardar capacidade; cargas já embarcadas sempre precisam caber.

Uma oferta pode ser recusada, e uma encomenda aceita ainda vazia pode ser cancelada. Para carga embarcada, **Devolver carga à origem** mantém o lote a bordo até descarregar na origem. Não há prêmio nem penalidade pela devolução. O diário distingue carregamento, entrega e devolução. Cancelamento concluído permanece visível até a próxima chegada, quando aparece outra oferta.

No automático, uma oferta pendente é aceita na próxima parada. Desligar o automático não cancela encomendas existentes. A escolha de despacho e os estados de devolução ficam salvos.

Entregar passageiros soma dois pontos por pessoa; carga soma dez por unidade. Os pontos registram serviços e não são moeda de compra. Chegadas, entregas e recompensas são transações do domínio: fechar menus ou consumir notificações não as repete.

## Coleção e locomotivas

As pinturas existentes continuam ligadas às chegadas e ficam registradas como propriedade no save. Entregar 20 passageiros conquista o postal Companhia de viagem e libera a Elétrica da Serra; entregar 20 unidades de carga conquista Correio sobre trilhos e libera a Diesel do Porto. Vapor Aurora permanece disponível desde o início. As novas locomotivas têm desenhos próprios, mantêm rodas e engates alinhados e não emitem fumaça de vapor.

O diário guarda os últimos 32 bilhetes de chegada, com embarques, desembarques, carga e pontos. O mapa permite consultar cada estação. A coleção registra as estações e os postais de serviço.

## Excursões

Após completar o primeiro circuito, **Viagem → Mapa → Explorar ramais** permite agendar Mirante da Serra ou Cais das Conchas. O trem parte de Vila Aurora após o serviço, viaja 90 segundos, visita o destino por oito segundos e retorna em 90 segundos ativos. Passageiros e encomendas do circuito permanecem a bordo; essas visitas são de exploração, sem inventar entregas locais. O postal é concedido uma única vez na descoberta.

A viagem principal retoma do mesmo ponto. As distâncias e identidades do Circuito Aurora original não mudam. A excursão em curso, seu progresso e descobertas ficam salvos. O mapa do ramal mostra ida, visita e retorno, com posição do trem e tempo ativo restante por fase; durante a pausa, indica que o relógio está parado. O circuito principal aguarda o retorno e não mostra outro trem viajando simultaneamente.

## Clima, atenção e som

São Paulo, SP, Brasil continua como padrão. **Visual → Clima** permite informar outra cidade e suas coordenadas. O nome identifica o local; não é uma busca geográfica. Não há GPS nem localização por IP. Uma troca de cidade não aceita resultados atrasados da anterior.

A última condição válida fica em cache entre aberturas, com idade e erro visíveis. Nascer/pôr do sol permitem iluminação gradual; vento e precipitação vêm do modelo meteorológico. Na ausência de dados solares válidos, permanece o estado dia/noite informado. Dados antigos não ganham um horário de observação novo.

**Visual → Fluidez** mantém a arte completa, permite reduzir animações dos menus e ocultar avisos sobre a paisagem durante o trabalho. Sons de eventos são opcionais, desligados inicialmente e respeitam o modo tranquilo. Não há música nem áudio de terceiros.

## Salvamento e recuperação

Preferências: `%LOCALAPPDATA%\CPU Express\config.json`. Jornada: `config.journey.json`. O formato v3 preserva itinerário, serviços, propriedade, coleção e excursões, acrescentando despacho e devolução. Na rc.3 o formato é o v4, que registra também a linha percorrida e a versão do catálogo; o v3 migra com backup exato (`.v3.backup.json`) e a rc.2 não lê o v4. Ao migrar v1 ou v2, o arquivo anterior recebe backup exato (`.v1.backup.json` ou `.v2.backup.json`) antes da substituição. A rc.1 não lê o formato v3: para voltar àquela versão, encerre o jogo novo e restaure uma cópia do backup v2; o progresso feito depois desse backup não existe na versão antiga. Versões futuras ou arquivos inválidos não são sobrescritos pelo salvamento comum.

Na rc.3, cada salvamento guarda o checkpoint anterior em `config.journey.previous.json`. Se o arquivo principal estiver danificado na abertura, ele é renomeado para `config.journey.damaged-….json`, sem alteração de conteúdo, e a viagem retoma do checkpoint anterior com um aviso; perde-se no máximo o intervalo desde o save anterior. Um save gravado por versão mais nova nunca é movido nem rebaixado.

Em **Viagem → Diário**, use salvar, exportar e importar. A importação valida a origem e guarda cópia do destino antes de substituí-lo, inclusive quando o destino está inválido. A ação substitui a viagem atual; não combina perfis. O app salva periodicamente e na saída normal. Fechamento forçado pode perder o intervalo desde o último save.

## Instalação opcional

A versão portátil funciona sem instalação. Os scripts `install-local.ps1` e `uninstall-local.ps1` oferecem instalação por usuário, sem administrador, quando incluídos no pacote. A inicialização com o Windows é opcional e só é configurada com o parâmetro explícito. A desinstalação preserva o perfil e as viagens. Consulte a ajuda/comentários dos scripts para os parâmetros desta distribuição.

## Ajustes da faixa na rc.3

Segure e arraste a faixa para movê-la horizontalmente. As laterais ajustam a largura; a lateral esquerda mantém a direita fixa. Um movimento menor que 4 pixels físicos continua sendo clique para abrir a central; o botão direito pausa. Os sliders em **Janela** e **Restaurar posição e tamanho** continuam disponíveis. Geometria de arrastes e sliders só é gravada ao soltar.

**Encaixar na altura da barra** vem ligado. Desligue para ajustar de 24 a 240 pixels físicos, inclusive pela borda superior: a base permanece alinhada à base da barra e a parte maior cobre a área de trabalho acima dela. Não há reserva de espaço para outros aplicativos. **Cantos arredondados** usa suporte do Windows 11; sem suporte (incluindo Windows 10), permanece retangular. Voltar à janela flutuante restaura sua moldura e remove o recorte da faixa.

A paisagem tem 70 linhas e largura interna entre 160 e 32768 colunas. Quando a barra inteira é menor que 160 px, a janela usa a largura disponível e recorta a cena. A largura acompanha o viewport dividido pela escala inteira; a última coluna pode ser recortada para preencher uma largura física não divisível pela escala. Filtros permanecem nearest, sem alongar o trem. Abaixo de 320 colunas o trem é recentrado; na largura mínima parte da traseira fica fora da vista, preservando o tamanho dos pixels e da locomotiva. Os períodos de parallax não dependem da largura. Os buffers do renderer só realocam ao mudar a largura; o upload de textura egui mantém seu staging habitual. O contrato de compatibilidade continua sendo 320 × 70.

Preferências acrescentadas: `taskbar_fit_height: true` e `taskbar_rounded: false`; JSON antigo recebe esses defaults. A largura persistida admite até 32768 px para comportar barras acima de 2400 px e é limitada pela barra atual na aplicação. Jornada e formato de checkpoint não são alterados por estes ajustes. Estas mudanças estão incluídas na rc.3; os pacotes rc.2 permanecem históricos.

## Limites de validação

Resultados executados para o binário ficam em BUILD-INFO e no pacote separado de evidências. Exportações egui verificam o desenho dos painéis, não o compositor. Medições por frame cobrem simulação, desenho e preparação da textura, não energia ou apresentação da GPU. DPI misto, versões distintas do Explorer, autohide e uso durante um dia inteiro dependem de validação específica. Não há assinatura digital, sincronização em nuvem ou versão para outros sistemas nesta distribuição.
