# E7 — correções priorizadas e limites

A referência de aceitação é a fonte da branch `expansao`, não o executável distribuído rc.3. A revisão foi aplicada com recorte autorizado pelo plano: primeiro Cabine, loja, resultados, moeda e controles; depois legibilidade e cabimento. O acabamento restante está explicitado abaixo.

## O que mudou

- Cabine da central ligada ao cartão **Próxima parada**, com PAUSAR/RETOMAR centralizado, paisagem em escala inteira, medidores e ocupação. CPU conserva identificação DEMO/SIMULADO e validação da amostra em cache. A tela antiga não era apenas uma imagem vencida: havia dois caminhos de apresentação, e a central continuava usando o antigo.
- `réis` em todo texto visível da economia, inclusive erros, migração, Máquina e resumo. Identificadores internos de sprites/fixtures continuam estáveis. Pontos de serviço permanecem registro histórico, com explicação; não foram convertidos em saldo.
- Nenhum `egui::Button`, `ui.button` ou `ui.small_button` no código do produto. `game::ActionButton` adapta `add_enabled`/`add_sized` ao próprio `game::button`, mantendo foco, teclado e rótulos acessíveis. Campos de ajuste, sliders, checkboxes e seletores de valores continuam controles egui.
- Abas em latão/teal, 36 pontos; navegação de páginas com setas de 40 pontos e pontos indicadores, omitida quando existe só uma página. A medição do texto pixelado usa a mesma escala física do desenho, corrigindo recortes descobertos nas imagens.
- Cinco remessas em 3+2 cartões, caixa grande, barra das chances reais, garantia em marcas e acesso aos detalhes antes da compra. Abertura com caixa de 192 pontos, fundo escuro, tremor/tampa/luz e botão para antecipar a revelação; reduzir movimento revela imediatamente.
- Resultados compartilhados por caixa/repetida/compra direta: cartão 360 × 320, faixa/moldura de raridade, nome, novidade/repetida, reembolso com sprite e EQUIPAR/GUARDAR 190 × 48. Vagões têm três slots desenhados. O resultado continua gravado antes da animação.
- Oficina, compra direta e coleção de peças em seis itens por página, cores distintas por raridade, miniaturas centralizadas e peças bloqueadas escurecidas na coleção. Oficina inclui raridade/capacidade dentro do cartão. Favoritos usam mini-composições e colunas de ações fixas.
- Diário com sete entradas de largura fixa e sem contadores zero; mapa Aurora maior, rótulos afastados do trilho, trem desenhado e separador de rota suportado. Ordens vazias usam relógio/EM BREVE, ações maiores e recompensas dos marcos alinhadas. Legendas das barras têm fundo contrastante.
- Acolhida com paisagem maior, modos visuais e ação principal grande; preserva aviso/opt-out de atividade, obrigatório nos Ajustes do plano. Ajuda curta com atalhos e fontes de réis; cartão Ajustes → Sobre reúne informações de crédito e privacidade. O crédito OSM permanece também nos mapas e Ajuda.
- Sala de máquinas preservada: o pedido de ocultá-la na revisão era anterior à E2, que já entregou medidores reais. A linha real e seus contratos de persistência também foram preservados.

## Recortes e pendências da revisão

A numeração abaixo remete à lista priorizada de `REVISAO_TELAS.md`.

- **P0 3–6:** entrega funcional dos novos blocos, com dimensões adaptadas ao espaço. Miniaturas mantêm proporção e escala inteira; não são esticadas para 180 × 90 ou 288 × 144. A abertura continua automática em 1,2 s; ABRIR! antecipa, não sorteia. Ainda faltam bloqueio temporário da doca, selo sobreposto no canto, contagem animada do reembolso, raios/confetes/flash, som por raridade e “abrir outra”. O recorte das texturas existentes foi preservado; há margem externa no cartão, sem reconstruir as miniaturas da Articulada.
- **P0 9–10:** Máquina não foi removida, pois já está pronta. Acolhida conserva o opt-out obrigatório; ainda não bloqueia a doca. Modos têm 72 pontos de altura e paisagem 640 × 140 em 1×, preservando pixels inteiros, em vez de alongar a cena para 180.
- **P1 13–16:** grades/slots/ações foram aplicados; cartões menores que 206 × 190 para caber com duas linhas de navegação. Faltam a faixa interna de raridade na compra direta/coleção, amostra de cor das pinturas, animação de prêmio pronto e as quatro colunas exatas dos marcos propostas pelo revisor.
- **P1 18:** chances sem zeros e percentuais arredondados; a garantia evita “1 aberturas”. Detalhes ainda são paginados, não uma grade completa sem páginas. As chances exatas são calculadas dos pesos vigentes; 87/13 na garantia representa arredondamento de 86,666…/13,333….
- **P1 19–22:** diário e contraste aplicados. Alguns hints continuam em linha separada; textos técnicos de Clima/Fluidez e o arquivo do Diário não foram totalmente relocados. Sobre reúne créditos, mas não remove todas as repetições. Botões principais de serviço têm 44 pontos; estatísticas mantêm texto com ícones. Na linha real, destinos passaram a um por página para acomodar os alvos maiores sem perder a doca.
- **P2 23–26:** faltam cartões ilustrados de ramais, barra segmentada da excursão, postais com arte própria e vitrine do museu.
- **P2 27–30:** Favoritos usa sprites e ações alinhadas, mas não desenha quatro slots vazios tracejados nem foi renomeado. Ajustes conservam subpáginas, coordenadas explícitas e controles técnicos de fluidez/volume. Ajuda recebeu os cartões; ganho sobe acima do trem, mas resumo ainda é textual e não toast temporário com contagem animada.

## Verificação e reprodução

Consulte o relato E7 no fim de `../PLANO_EXPANSAO.md` para resultados finais. O teste `ganhar_reis_comprar_caixa_equipar_salvar_e_reabrir_preserva_tudo` percorre três categorias: renda pela viagem e `advance_checkpoint`, compra paga por clique egui, equipamento, confirmação, `on_exit`, nova instância do aplicativo e leitura de configuração/checkpoint temporários. Compara economia inteira, inventário/RNG/transações/garantia/pendência, serviços, composição, preferências visuais, progresso por linha, distância e ausência de pagamentos no repaint. Não abre janelas nativas.

Os três comandos de aprovação são `cargo fmt --all`, `cargo clippy --locked --all-targets -j 2 -- -D warnings` e `cargo test --locked --all-targets -j 2`. Use a toolchain local de `scripts/dev.ps1`, Cargo offline, `CPU_EXPRESS_OFFLINE=1` e `CPU_EXPRESS_CONFIG` temporário. Para exportar: `CPU_EXPRESS_VISUAL_DIR` explícito e `cargo test --locked --test ux_visual -j 2 -- --ignored --test-threads=1`; rasterizar os JSONs com o Node local por `node docs/ux_rasterizar.cjs <diretório>`. Nenhuma dependência nova ou download.

A conferência nativa pendente está em [CONFERIR_NA_TELA.md](CONFERIR_NA_TELA.md).
