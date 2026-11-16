Li `docs/PLANO_EXPANSAO.md` inteiro e confrontei com os arquivos indicados. Não editei, compilei nem executei testes. **O plano não cabe com segurança numa noite: E1 e E3 concentram mudanças estruturais, e a economia contradiz os desbloqueios preservados.**

As principais divergências e lacunas técnicas são:

- **E5 pede miniaturas já integradas:** `src/ui_stock.rs`, `StockGallery::new`, renderiza os modelos; `src/app/workshop.rs`, `appearance_controls`, já usa essas texturas. Hoje existem **19 vagões, 15 cargas, 10 locomotivas e 10 pinturas**; a proposta econômica ainda descreve conjuntos anteriores.
- **E2 pressupõe uma espera de carga inexistente:** `src/simulation/service.rs`, `ServiceState::arrive`, embarca/desembarca instantaneamente; `src/simulation/world.rs`, `advance_expedition`, aplica oito segundos de parada geral. Reduzi-la altera também frequência de chegadas e renda.
- **A estimativa de 40–60 moedas/h por chegada está errada:** `Route::from_seed`, em `src/simulation/route.rs`, produz circuitos de 8.600–9.400 unidades; a 24 unidades/s e quatro paradas de oito segundos, são aproximadamente **68–74 moedas/h**, sem excursões.
- **O início proposto não corresponde à composição existente:** `ServiceState::migrated` equipa Panorâmico, Dormitório e Carga; `composition: [ServiceWagon; 3]` não admite vaga vazia. Definir o terceiro vagão e se posse significa modelo reutilizável ou quantidade de exemplares.
- **`Itinerary` não substitui sozinho o motor:** `JourneySnapshot` contém quatro descobertas e `StationId` só quatro variantes; `ServiceState::arrive` usa `% 4`. Em `src/progress.rs`, `Checkpoint::restore` e `decode_world` recusam outras linhas, e `encode_world` grava Aurora.
- **“Passar nas validações” não comprova serviço real:** `data/lines/curitiba-paranagua.json` mantém dúvidas sobre embarque e demanda; `catalog::validate_line` rejeita `open_questions` em linha jogável. Resolver editorialmente e identificar serviços fictícios; simplesmente apagar dúvidas seria incorreto.
- **E2 não é extensão pronta do coletor:** `SysinfoProvider::sample`, em `src/metrics/sysinfo_provider.rs`, coleta apenas CPU/RAM; `Cargo.toml` habilita somente `system` no sysinfo. Novos sinais exigem implementação e validação; tempo ativo deve continuar vindo de `SimulationClock`, não do uptime.

Para caber numa noite, recortaria cada seção assim:

- **E1:** carteira, posse, renda, uma remessa geral e compra direta, com migração e transação persistida. Adiar quatro remessas, ordens e museu. `app.rs::persist_progress` salva a cada 30 segundos: compras precisam de confirmação imediata, sem confirmar estado em memória quando a gravação falhar.
- **E2:** mostrar CPU/RAM e ligar efeitos visuais; no máximo acrescentar um provedor Windows. Adiar renda por sinais, telegramas e alteração de serviço. Preservar validade parcial e expiração das amostras de `metrics/mod.rs`.
- **E3:** separar em outra entrega estrutural. Nesta noite, integrar arquitetura/cenário e consulta do mapa; manter a linha como rascunho. Torná-la jogável exige jornada, identidades, serviços, excursões e restauração coerentes, não apenas seletor.
- **E4:** conectar o conteúdo existente antes de desenhar 30 elementos: `app.rs::step_frame` não chama `set_scenery_context`, `set_station_styles` nem as opções de movimento. Atenção: `SegmentScenery(u16)` já usa 12 bits; cinco etiquetas adicionais únicas excedem 16, e `Port` já existe.
- **E5:** priorizar Trem, oficina paginada e os 11 marcos de `Mission::progress`; reutilizar miniaturas. Adiar redesenho integral. `floating_panel` hoje permite mínimo 480×480: travar 704×676 e depois reduzir a janela exige layout adaptativo, não só trocar constantes.
- **E6:** uma loja funcional com saldo, probabilidades, compra direta e resultado persistido; revelação estática inicialmente. Adiar animação, cinco famílias de ícones, resumo acumulado e partículas de moedas.
- **E7:** reservar tempo explícito para revisão e correção de bloqueadores. Nomear quem produz `REVISAO_TELAS.md`; incluir interação, teclado, DPI e retorno após reinício. PNG não verifica foco, clique, persistência nem compositor.

A ordem também precisa mudar:

- Definir **um integrador de persistência, app e Config** e o esquema antes de E1. Deixar E3 decidir depois entre v5/v6 provoca retrabalho nas migrações; contadores globais e por linha precisam de identidades acordadas.
- E1 deve publicar contratos estáveis para E5/E6; E2 depende deles e das decisões de serviço. O balanceamento final precisa ocorrer **depois** dos efeitos de E2 e das linhas de E3, pois ambos alteram renda.
- A integração visual de E4 pode anteceder a nova linha. Manter E5/E6 no fim da fila, atrás de dois trabalhos estruturais, arrisca entregar só domínio sem o que o jogador verá.

Os problemas de economia e vínculo com a máquina são:

- **Os marcos esvaziam a loja:** `ServiceWagon::required_stops` libera todos os vagões até 16 chegadas; `EngineStyle::required_reward` libera todos os motores com os dois postais de 20 entregas; `Livery::required_stops` libera todas as pinturas até 48 chegadas. Preservar isso para novos perfis transforma compras em antecipações breves. Separar direitos adquiridos de progressão nova.
- **A renda acumula fontes sobrepostas:** em `ServiceState::arrive`, `reward = units × 10`; portanto o plano paga aproximadamente **1,5 moeda por unidade de carga**, somando unidade e contrato. Acrescenta passageiros, chegadas, ordens e máquina, sem o teto temporal da proposta original.
- **A primeira compra não fecha com o alvo:** saldo inicial 30, preço 60 e renda 90–130/h dão aproximadamente **14–20 minutos**, antes dos marcos. Definir arredondamento e guardar restos das divisões por quatro/dois para não perder pequenas entregas.
- **Ordens/telegramas estão subespecificados:** faltam baselines, teto temporal, descarte sem punição, tratamento de objetivos simultâneos e capacidade realmente disponível. Possuir um vagão não garante poder equipá-lo enquanto há passageiros/carga a bordo.
- **Os conjuntos precisam existir antes das probabilidades:** “Plataforma e conforto” não tem cobertura rara/épica definida pelas propostas. Definir elegibilidade, chances efetivas da garantia, reembolso da caixa grátis e versão do conjunto; não completar raridades apenas para preencher tabela.
- **O simulador não tem estratégia definida:** escolher caixas, compra direta, composição e resgate muda completamente os resultados. Variar sementes, incluir migração e ambas as linhas; informar quantas execuções não completam a coleção em 30 dias, sem inventar percentis.
- **Saturação não elimina incentivo ao desperdício:** até o limite, CPU, RAM, disco e rede artificiais continuam rendendo vantagem. CPU+entrada oferecem diretamente apenas +20%; passageiros extras, paradas menores e telegramas escapam do teto declarado de +30%. Recomendo efeitos cosméticos, preservando renda independente da máquina.
- **Privacidade precisa de redação precisa:** preferir entrada desligada inicialmente e sem histórico persistido. “Nada sai do computador” deve limitar-se às métricas: `src/local_weather.rs`, `fetch`/`request_path`, envia coordenadas ao serviço meteorológico. Prever ensaio real de antivírus/SmartScreen, sem prometer ausência de alertas nem pedir exclusões.

Para o jogador, falta especificar:

- Em `welcome_controls` e na página Trem: objetivo imediato, o que acontece automaticamente, diferença entre pontos/moedas, tempo ativo e ausência de progresso oculto/offline; explicar que aparência da velocidade não mede avanço.
- Em serviços/oficina/loja: por que a oferta não cabe, como conseguir a peça necessária, quais benefícios são apenas visuais e a alternativa determinística às caixas. Definir troca de linha com passageiros, contrato aceito vazio e excursão; Aurora é circuito e não possui terminal declarado.

Mudar antes de começar:

1. **P0 — Cortar escopo:** uma economia mínima utilizável, conteúdo existente conectado e páginas essenciais; retirar a generalização de linhas da noite.
2. **P0 — Fixar persistência e posse:** integrador único, terceiro slot, direitos legados, IDs de transação, compra pendente e comportamento diante de falha.
3. **P0 — Recalcular progressão:** corrigir renda, resolver desbloqueios versus loja, definir conjuntos e limitar recorrências.
4. **P1 — Retirar recompensa por consumo da máquina:** efeitos visuais, privacidade explícita e tempo ativo único.
5. **P1 — Reservar entrega verificável:** percurso completo “ganhar → comprar → equipar → reabrir”, revisão interativa e contingência quando alguma seção não terminar.