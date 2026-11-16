# Estações — seis arquiteturas originais

Integração E3 (16/11/2026): o campo opcional `StationDef.architecture` e as 14 sugestões abaixo estão no catálogo, com validação de estilo e `kind: fictional`. Aurora busca os mesmos estilos da E5 no catálogo. A linha real está jogável; faixa e prévia escolhem as arquiteturas de partida/chegada também no retorno. Consulte [LINHAS_E3.md](../expansao/LINHAS_E3.md); o relato original abaixo foi preservado.

Agente 3, 15/11/2026. Todas as arquiteturas novas têm `kind: fictional`, exposto como `StationStyle::KIND`; são desenhos próprios em código, não reconstruções dos edifícios reais. A paleta acompanha bioma/iluminação existentes, mas a **silhueta é escolhida por estilo** e nunca pelo índice da parada.

| Ordem da galeria | Estilo estável | Característica |
| --- | --- | --- |
| 1 | `port_terminal` | Galpão longo, cobertura serrilhada e torre de controle |
| 2 | `mountain_stone` | Empena íngreme, base de pedra, contrafortes e chaminé |
| 3 | `timber_halt` | Abrigo aberto, tábuas, banco e placa independente |
| 4 | `plateau` | Telhado amplo, varanda e relógio baixo |
| 5 | `urban_terminal` | Alas simétricas, saguão envidraçado e torre de relógio |
| 6 | `service_post` | Cabine pequena, reservatório elevado e ferramentas |

## Caminho de integração

`SceneRenderer::set_station_styles(StationStyles { departing, arriving })` recebe `Option<StationStyle>` para as duas plataformas visíveis. Usar `StationStyle::ALL` nas ferramentas. `None` conserva o caminho anterior; limpar o contexto ao trocar de linha. Não acrescentar campo em `SceneContext`, não ler catálogo dentro do renderer e não resolver estilo pelo ordinal. Excursões ignoram o override e mantêm Mirante/Cais, ainda sem serviços reais próprios. Legacy mantém seu renderer anterior.

A posição continua sendo a da jornada atual do `World`. Esta API permite conferir a arte agora, **não resolve sozinha o limite de quatro estações do motor atual**. Ao integrar `Itinerary`, fornecer posições de chegada/partida no adaptador preservando os limites; não fingir uma nova linha usando índices Aurora.

Grade 320 × 70, baseline y=54–56 e arte dentro dos limites horizontais antigos x−7..x+71. Sem alocações por frame, sem nova animação arquitetônica, sem mexer em rodas/engates/sockets. A camada de cenário do Agente 2 permanece antes da estação; trem depois. Contagens/animação de embarque continuam compartilhadas com o código antigo, limitadas a três representantes por direção e controladas pelo serviço ativo. Silhuetas sobrevivem em qualidade Low, retirando apenas pequenos detalhes.

## Campo de catálogo proposto — não implementado

Pedir ao integrador `StationDef.architecture: Option<StationArchitectureDef>` com `style` validado contra os seis IDs, `kind: fictional` e default `None`. O tipo de definição deve morar no domínio, sem importar renderer; o adaptador converte para `StationStyle`. Não confundir `line.kind: real` com fidelidade da ilustração. Um futuro `kind: real` na arquitetura precisaria de fontes próprias e desenho comprovado. O identificador de arquitetura é conteúdo, não novo estado de checkpoint. Atualizar validação/esquema e exemplos em conjunto; esta rodada **não editou `simulation/catalog.rs` nem dados de linha**.

Sugestão para as 14 paradas na ordem atual do catálogo, escolhida como linguagem visual, sem alegar equivalência histórica:

| Parada | Estilo proposto | Motivo de apresentação |
| --- | --- | --- |
| Paranaguá | `port_terminal` | Entrada da linha no ambiente portuário |
| Alexandra | `timber_halt` | Parada baixa e pequena na baixada |
| Saquarema | `service_post` | Alternar abrigo e equipamento de serviço |
| Morretes | `plateau` | Varanda e relógio para um centro intermediário; o nome do estilo não exige altitude elevada |
| Porto de Cima | `service_post` | Posto compacto na transição para subida |
| Engenheiro Lange | `mountain_stone` | Entrada visual da serra |
| Marumbi | `timber_halt` | Abrigo aberto junto da mata |
| Cadeado | `mountain_stone` | Empena e pedra na encosta |
| Véu de Noiva | `service_post` | Equipamento simples entre obras de serra |
| Banhado | `timber_halt` | Pausa de silhueta leve antes do alto |
| Roça Nova | `mountain_stone` | Marco visual do topo da subida |
| Piraquara | `plateau` | Varanda e telhado amplo no planalto |
| Pinhais | `plateau` | Continuidade de arquitetura antes da capital |
| Curitiba (Rodoferroviária) | `urban_terminal` | Terminal largo como destino de ponta |

Essas escolhas não confirmam operação de embarque atual. Fonte para nomes/ordem: catálogo local e [LINHAS.md](../LINHAS.md). A aprovação artística é separada da pesquisa histórica e da habilitação da linha.

## Reprodução local e legenda

Usar `scripts/dev.ps1`, offline, uma invocação Cargo por vez e `-j 2`:

```text
cargo run --locked -j 2 --example station_gallery -- docs/conteudo/agente3-estacoes-final.ppm --estilos
node scripts/content-png.cjs docs/conteudo/agente3-estacoes-final.ppm docs/conteudo/agente3-estacoes-final.png 2
cargo run --locked -j 2 --example station_gallery -- docs/conteudo/agente3-aurora-ramais-final.ppm
node scripts/content-png.cjs docs/conteudo/agente3-aurora-ramais-final.ppm docs/conteudo/agente3-aurora-ramais-final.png 1
cargo run --locked -j 2 --example map_gallery -- docs/conteudo/agente3-mapas-final-v2.ppm
node scripts/content-png.cjs docs/conteudo/agente3-mapas-final-v2.ppm docs/conteudo/agente3-mapas-final-v2.png 1
```

Exportadores recusam sobrescrever destino: ao repetir, usar nome novo. A galeria antiga de estações foi mantida como modo padrão: seis linhas Pines/Ridge/Harbor/Aurora/Mirante/Conchas; três colunas aproximação/serviço/partida. O modo `--estilos` acrescenta seis linhas na ordem acima, quatro colunas dia/entardecer/noite/chuva. Ampliação por dois é nearest, sem suavização. A galeria de mapas usa duas células de 480 × 480: 14 paradas à esquerda, 29 sintéticas à direita; números são ordinais do esquema, ponto âmbar é o trem, cheios são descobertas e vazados são pendentes. Parte superior é percurso; inferior é altitude, com o mesmo trem.

A imagem `agente3-mapas-final.png` é a primeira conferência, preservada como histórico: sua seta tratava ida/volta como direção horizontal. `agente3-mapas-final-v2.png` corrige a seta para apontar da estação de origem à próxima no esquema, inclusive nas curvas e na volta. As coordenadas do módulo não mudaram nessa revisão.

Resultados executados e limites estão no relato ao fim de [RODADA_CONTEUDO.md](../RODADA_CONTEUDO.md). Imagem headless não comprova compositor, DPI, interface acessível ou leitura na faixa real de 60 pixels.
