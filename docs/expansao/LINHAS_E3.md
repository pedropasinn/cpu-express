# E3 — domínio genérico e consulta da linha

Entrega parcial de desenvolvimento, 16/11/2026. Foi adotado o recorte autorizado nos Ajustes do plano: domínio genérico testável e mapa consultável, mantendo Curitiba–Paranaguá fora do seletor de viagem. **A linha real ainda não é jogável no aplicativo.** Aurora continua sendo a única linha do World e do checkpoint v5. Vitória–Minas permanece rascunho e não aparece na consulta.

## O que aparece no aplicativo

Viagem → Linhas abre uma consulta das 14 paradas de Curitiba–Paranaguá. O esquema numera as paradas na ordem do catálogo, começando em Paranaguá. Clicar em um ponto ou usar Anterior/Próxima mostra nome, quilômetro, altitude conhecida e arquitetura fictícia. Os pontos têm foco de teclado e setas para percorrer a ordem ferroviária. O perfil mantém distância proporcional, não inventa altitude ausente e destaca a estação selecionada. A tela mostra `© OpenStreetMap contributors`; Ajuda conserva o crédito existente.

Consultar não muda a viagem, serviços, descobertas nem moedas. Não há marcador de trem ou selo de descoberta na linha que o jogador ainda não percorre. O texto indica que a viagem está indisponível e que Aurora continua em curso. O mapa usa geometria em cache e não consulta rede. As questões editoriais e fontes do JSON foram preservadas integralmente.

## Arquitetura e cenário

`StationDef.architecture` é opcional e contém `style` e `kind`. Seis estilos são aceitos; `kind` deve ser `fictional`. O domínio não importa o renderer. As quatro escolhas de Aurora feitas na E5 foram transferidas para o catálogo sem mudar o resultado visual. As 14 sugestões de `docs/conteudo/ESTACOES.md` foram aplicadas à linha real; não representam reconstruções dos prédios históricos.

O adaptador resolve partida/chegada pelo ID da estação, inverte as plataformas no retorno e escolhe o próximo trecho adjacente, repetindo o trecho nas pontas de um vai-e-volta. Contexto ausente ou excursão limpa etiquetas, estilos e densidade. A densidade urbana deriva das arquiteturas de centro/terminal nas pontas do trecho. A chamada do World continua em Aurora; os casos reais são exercitados em testes até a integração da linha ativa.

## Contratos preparados no domínio

- `ServiceState<S = StationId>`, `PassengerGroup<S>`, `CargoContract<S>` e `ArrivalSummary<S>` compartilham o mesmo motor de embarque, entrega, despacho, cancelamento, devolução e pontos. Aurora usa o parâmetro padrão, com os quatro enums e JSON anteriores. Não houve migração de produção nem mudança de versão.
- `CatalogStationId { line, station }` qualifica IDs estáveis. Ordinais e nomes exibidos não são chaves persistidas. Serviços genéricos rejeitam referências de outra linha e destinos inexistentes.
- `LineSession` recebe o `Itinerary` efetivo. `playable` recusa rascunhos e conserva os comprimentos por seed de Aurora. `for_inspection` permite exercitar uma linha completa sem habilitá-la no produto. A sessão usa 24 unidades/s, 480 unidades/km, tempo ativo e os mesmos limites de suspensão. Não recebe CPU, relógio de parede, economia ou acesso a disco.
- O serviço é declarado `ContentKind::Fictional`. O aviso é “Os serviços mostrados são do jogo e não representam a operação real.” Demanda e destinos são determinísticos e fictícios, sem alegação sobre operação ferroviária atual.
- A chegada chama a transação compartilhada antes de devolver controle. Na inspeção real, madeira/posto de serviço esperam quatro segundos; demais paradas, oito. Aurora conserva oito em todas. Pausa e passos inválidos não alteram contadores.
- `progress()` expõe o `LineProgress` já reservado no v5: distância, tempo, espera, visitas, percursos e conjunto de descobertas por ID. `restore` valida integralmente progresso e serviços antes de publicar ambos. Não altera a leitura/gravação do checkpoint do aplicativo. A fixture `tests/fixtures/e3-line-inspection.json` é somente entrada de teste desse contrato; não é um novo formato importável de perfil.
- `can_leave_line` consulta elegibilidade: plataforma terminal no vai-e-volta, qualquer plataforma no circuito, nenhum passageiro/carga, contrato aceito/carregado/devolvendo nem excursão. A consulta **não troca linha** nem transfere contadores.

## Integração ainda necessária

1. Substituir a jornada ativa do World e adaptar snapshot/renderer para os IDs genéricos. Manter o adaptador legado e todas as distâncias de `aurora-v1`; não converter a distância de Aurora em outra linha.
2. Definir contadores globais monotônicos para economia, ordens, marcos e diário. As visitas do contrato genérico são locais à linha. Não aplicar esses valores locais diretamente aos baselines globais existentes nem reiniciar moedas/inventário.
3. Integrar checkpoint ativo e mapa `lines`, validar cada linha, proteger IDs desconhecidos e migrar com backup exato caso a representação de serviços no documento mude. Esta entrega não inventa um v6 antecipado.
4. Gravar a troca imediatamente em candidato completo e só publicar após sucesso; testar falha de escrita, reabertura, ofertas pendentes e retorno ao progresso anterior de cada linha. Dar ao jogador um meio explícito de suspender novos embarques para esvaziar a composição sem descartar passageiros.
5. Depois disso, promover a linha a `playable`, deslocar as questões preservadas para notas editoriais, fornecer o aviso de serviços fictícios, ligar seletor e posição real no mapa. Acrescentar “conheça as 14 paradas” e “suba a serra” com resgate único na economia e persistência transacional.

Esses itens não estão concluídos. As descobertas genéricas estão prontas para requisitos, mas não concedem moedas ou missões novas. A linha continua `draft` para que a consulta não prometa uma viagem incompleta.

## Verificação e evidências

Resultados finais e limites estão no relato E3 ao fim de `docs/PLANO_EXPANSAO.md`. Testes novos cobrem ida/volta com serviços, 29 paradas fictícias, restauração e identidade qualificada, elegibilidade de saída, suspensão, partição de frames, paridade de Aurora, catálogo de arquitetura e mapa por teclado sem mutação do jogo. A suíte existente continua responsável pelas migrações v1–v4, backup exato, recuperação e checkpoint v5.

O exportador opcional `export_line_map` em `tests/ux_visual.rs` produz três telas headless com Paranaguá, Marumbi e Curitiba selecionadas. Reprodução offline, com configuração temporária e a toolchain de `scripts/dev.ps1`:

As [três imagens finais](telas-e3-final/README.md) foram geradas e inspecionadas nesta entrega.

```text
cargo test --locked --test ux_visual -j 2 export_line_map -- --ignored
node docs/ux_rasterizar.cjs docs/expansao/telas-e3-final
```

Definir `CPU_EXPRESS_VISUAL_DIR` como o caminho absoluto de uma pasta nova para os JSONs; o comando Node recebe essa mesma pasta. Definir também `CARGO_NET_OFFLINE=true`, `CPU_EXPRESS_OFFLINE=1` e `CPU_EXPRESS_CONFIG` em pasta temporária. PNGs não comprovam compositor, DPI, Narrador ou interação nativa. Não houve release, pacote nem atualização de BUILD-INFO/manifestos distribuídos.
