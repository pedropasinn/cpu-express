# Central da expansão — E5

Entrega de fonte na branch `expansao`, em 16/11/2026. Não substitui a distribuição rc.3 nem suas evidências.

## Navegação

A central nativa tem 704 × 676 pontos lógicos, moldura própria, posição persistida e arraste pela faixa do título. O tamanho antigo salvo é ignorado. A área útil do monitor limita uma escala proporcional única, aplicada ao `pixels_per_point` pelo zoom do egui; não há outro layout para telas pequenas. A janela não oferece redimensionamento. O comportamento em monitores com DPI diferente ainda precisa de conferência nativa.

Como o egui aplica zoom no quadro seguinte, o adaptador confere a dimensão lógica reportada e reaplica tamanho e limites nativos quando necessário. Há teste dos comandos emitidos, separado da verificação real do compositor.

| Doca | Páginas |
| --- | --- |
| Trem | Cabine, Vagões, Motores, Pinturas, Favoritos |
| Viagem | Mapa, Missões, Diário, Serviços, Ramais |
| Loja | Apresentação das remessas e preços do domínio; compras e revelação entram na E6 |
| Coleção | Postais paginados; a grade completa de peças entra na E6 |
| Ajustes | Janela, Clima, Fluidez, Sessão |
| Fechar | Fecha apenas a central; não encerra o aplicativo |

O botão `?` abre Ajuda. O saldo em **Réis de Ferro** permanece na faixa do título. Um pequeno `!` na doca Viagem indica prêmio disponível. A faixa de estado tem altura fixa e prioriza erros de salvamento; textos longos podem ser consultados pela dica. A saída explícita está em Ajustes → Sessão.

As listas usam setas: quatro miniaturas por página, três marcos, três ordens e três bilhetes. Um telegrama extra ocupa a segunda página de ordens. Clima separa condição e alteração de cidade; Fluidez separa movimento, atenção/som e informações; Janela separa geometria e comportamento durante o trabalho. Serviços mantém oferta, decisão e ocupação na mesma página. Não há `ScrollArea` na central nativa. O pequeno painel de compatibilidade de backends sem viewports nativos permanece legado.

## Missões e equipamento

Missões lê `EconomyState::missions` e as instâncias de `daily_orders`. Exibe próxima recompensa, progresso, valor e estado recolhido. Ordens não expiram e só chegam pelos critérios já implementados no domínio. Espaços vazios explicam a espera; a tela não cria ordens nem substitutas.

`src/app/missions.rs` envia `ClaimMission`, `ClaimOrder` e `SwapOrder` a `progress::economy_action`. O candidato só chega ao mundo em memória depois de gravado. `Ok(Some(aviso))` é sucesso com aviso de recuperação, sem repetir o resgate. Erros ficam na faixa de estado. Uma viagem temporária informa que precisa de perfil salvo para resgatar; não recebe um pagamento fictício.

Vagões, locomotivas e pinturas usam `StockGallery`, com texturas do renderer e filtro nearest. Moldura e palavra indicam raridade; estado distingue possuído, equipado e ainda não possuído. Posse vem do inventário inclusive na demonstração destes seletores. Vagões mostram ícone e capacidade; a dica da capacidade lista cargas compatíveis. O domínio continua recusando uma composição que não comporta o serviço a bordo. Favoritos permanecem limitados a quatro.

## Conteúdo ligado ao aplicativo

`src/app/scenery.rs` fornece `SceneryContext` atual/próximo a partir das etiquetas do catálogo Aurora. Mapeia as estações fictícias Aurora/Bosque/Serra/Porto para Planalto/Madeira/Pedra/Terminal portuário. Isso muda **intencionalmente** a ambientação apresentada pelo app: agora a arte pendente aparece. O renderer vazio e seus hashes de compatibilidade continuam intactos.

Excursões limpam etiquetas e estilos, sem reaproveitar o contexto do circuito. O adaptador também retorna entradas vazias para linha ou trecho desconhecido. E3 deverá trocar a identificação Aurora pela linha ativa e fornecer arquitetura e sentido pelo itinerário genérico; não foi habilitada uma linha real nesta entrega.

`reduce_menu_motion` passa a controlar menus, passageiros e vida da paisagem; o texto informa esse alcance. Frio visual usa temperatura finita, não vencida e abaixo de 12 °C. Não muda capacidade, pagamento, relógio ou parada.

A nova preferência `Config.use_input_activity`, com default `true` para JSON antigo, prepara a autorização de atividade agregada para E2. Pode ser desligada na acolhida e em Ajustes → Fluidez → Atenção e som. A interface informa que a coleta ainda não está disponível; E5 não consulta entrada, processos ou janelas. E2 deve consumir essa preferência e atualizar a explicação quando ligar a coleta. Não foi alterado o checkpoint v5.

## Reprodução e evidências

Os 45 PNGs em [telas](telas/README.md) vêm das malhas e texturas do aplicativo real, por `tests/ux_visual.rs`, em 704 × 676. São renderizações headless, **não capturas do compositor Windows**. Os cenários usam demonstração e evolução determinística em memória; nenhum lê o perfil real. JSONs intermediários ficam locais, ignorados pelo Git, e podem ser regenerados.

```powershell
$env:CARGO_NET_OFFLINE = 'true'
$env:CPU_EXPRESS_CONFIG = Join-Path $env:TEMP 'cpu-express-e5-isolado/config.json'
$env:CPU_EXPRESS_VISUAL_DIR = Join-Path (Get-Location) 'docs/expansao/telas'
.\scripts\dev.ps1 -CargoArgs @('test','--locked','--test','ux_visual','-j','2','export_expansion_menus','--','--ignored','--nocapture')
py docs/ux_rasterizar.py docs/expansao/telas
```

Neste computador `py` encontrou Python 3.13, mas a criação do processo retornou **Acesso negado**. Nenhuma instalação ou tentativa de contornar essa restrição foi feita. Foi usada a alternativa local `node docs/ux_rasterizar.cjs docs/expansao/telas`, sem dependências, que reproduz as regras de cobertura de triângulos, mistura e filtros do script Python. Não foi feita comparação numérica entre os dois rasterizadores neste ambiente.

Os testes novos verificam cabimento e doca visível em todas as páginas, quatro favoritos com nomes longos, variantes de oferta/devolução/ramal/geometria, paginação, fator proporcional e reaplicação de tamanho após zoom, contexto de cenário e resgate acionado pelo botão com reabertura, repetição recusada e falha de escrita sem alterar carteira. Os testes completos cobrem os contratos existentes de salvamento, serviços e renderer.

Não foram verificados compositor, arraste real, foco/teclado com Windows/Narrador, DPI misto, animação contínua, energia, uso prolongado ou pacote de distribuição. O fluxo completo comprar → equipar → reabrir e a revelação de caixa aguardam E6/E7. BUILD-INFO, manifestos e binário distribuído continuam os anteriores.
