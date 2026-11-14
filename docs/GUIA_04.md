# CPU Express 0.4.0-rc.2 — serviços e escolhas

Esta edição amplia a ferrovia mantendo o encaixe à esquerda, o modo flutuante e os menus independentes. A viagem usa tempo ativo, não a carga de CPU. Pausa, ocultação e fechamento não rendem progresso offline.

## Começar

Abra `cpu-express.exe` ou o iniciador da raiz. Clique no trem para abrir o caderno. **Trem** mostra destino e pausa; **Viagem** reúne mapa, objetivos, diário, serviços e coleção; **Visual** contém vagões, pinturas, clima, fluidez e motores. A janela do caderno guarda sua posição e tamanho separadamente da faixa.

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

Preferências: `%LOCALAPPDATA%\CPU Express\config.json`. Jornada: `config.journey.json`. O formato v3 preserva itinerário, serviços, propriedade, coleção e excursões, acrescentando despacho e devolução. Ao migrar v1 ou v2, o arquivo anterior recebe backup exato (`.v1.backup.json` ou `.v2.backup.json`) antes da substituição. A rc.1 não lê o formato v3: para voltar àquela versão, encerre o jogo novo e restaure uma cópia do backup v2; o progresso feito depois desse backup não existe na versão antiga. Versões futuras ou arquivos inválidos não são sobrescritos pelo salvamento comum.

Em **Viagem → Diário**, use salvar, exportar e importar. A importação valida a origem e guarda cópia do destino antes de substituí-lo, inclusive quando o destino está inválido. A ação substitui a viagem atual; não combina perfis. O app salva periodicamente e na saída normal. Fechamento forçado pode perder o intervalo desde o último save.

## Instalação opcional

A versão portátil funciona sem instalação. Os scripts `install-local.ps1` e `uninstall-local.ps1` oferecem instalação por usuário, sem administrador, quando incluídos no pacote. A inicialização com o Windows é opcional e só é configurada com o parâmetro explícito. A desinstalação preserva o perfil e as viagens. Consulte a ajuda/comentários dos scripts para os parâmetros desta distribuição.

## Limites de validação

Resultados executados para o binário ficam em BUILD-INFO e no pacote separado de evidências. Exportações egui verificam o desenho dos painéis, não o compositor. Medições por frame cobrem simulação, desenho e preparação da textura, não energia ou apresentação da GPU. DPI misto, versões distintas do Explorer, autohide e uso durante um dia inteiro dependem de validação específica. Não há assinatura digital, sincronização em nuvem ou versão para outros sistemas nesta distribuição.
