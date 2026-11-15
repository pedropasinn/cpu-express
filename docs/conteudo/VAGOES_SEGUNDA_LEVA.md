# Segunda leva de vagões — Agente 5

2026-11-15. Oito modelos originais desenhados em Rust, todos `kind: fictional`. Não representam veículos históricos, operadores reais nem recursos extraídos de outro jogo. Os onze modelos anteriores permanecem disponíveis com seus mesmos identificadores, capacidades e desenhos.

## Função e raridade proposta

As capacidades são unidades abstratas de contrato: não equivalem a toneladas, animais individuais, automóveis visíveis ou quantidade de objetos no sprite. Cada modelo aceita somente a carga da sua linha. Todos têm zero lugares para passageiros. Uma unidade de cada vagão comporta a maior oferta atual, de 12 unidades.

| Vagão / ID | Carga / ID | Capacidade | Liberação provisória | Raridade proposta e justificativa |
| --- | --- | ---: | ---: | --- |
| Vagão de gado / `livestock` | Gado / `livestock` | 12 | 3 chegadas | Comum: serviço rural próprio, com laterais ventiladas; continua necessário mesmo com os modelos raros. |
| Porta-contêineres / `container` | Contêineres lacrados / `containers` | 20 | 5 chegadas | Comum: operação intermodal em plataforma com dois módulos de alturas diferentes. Não substitui encomendas soltas. |
| Cegonha de automóveis / `autorack` | Automóveis / `vehicles` | 12 | 7 chegadas | Incomum: dois pisos abertos para veículos, sem aceitar carga geral. |
| Tanque de gás / `pressure_tank` | Gás pressurizado / `gas` | 12 | 9 chegadas | Incomum: vaso arredondado com domo de válvula protegido. O tanque antigo continua exclusivo para líquidos. |
| Guindaste de socorro / `crane` | Máquinas de manutenção / `machinery` | 12 | 16 chegadas | Épico: lança treliçada recolhida, gancho e cabine. Destaque visual com papel logístico, sem multiplicador. |
| Bagagem com bicicletas / `bicycle` | Bicicletas / `bicycles` | 16 | 11 chegadas | Comum: cabine curta e bicicletário aberto. Bicicletas são carga contratada, sem exigir grupos de passageiros novos. |
| Berço de bobinas / `coil` | Bobinas de aço / `steel_coils` | 18 | 12 chegadas | Incomum: três bobinas em berços baixos; produto industrial distinto de minério bruto e toras. |
| Feira itinerante / `fair` | Equipamentos de feira / `fair_equipment` | 16 | 14 chegadas | Raro: teto listrado, bandeirola e estruturas dobradas. Transporta equipamentos, sem eventos obrigatórios ou animais de circo. |

Capacidades, marcos e raridades são decisões autorais para Pedro confirmar. A leitura de `docs/economia/notas-B.md`, especialmente utilidade estratégica e coleção, orientou as funções exclusivas; o livro não fornece estes números nem raridades. Não há cadeia de melhoria linear: nenhum novo vagão atende a carga de outro. O limite de três posições exige escolher quais serviços atender. Capacidades acima de 12 não aumentam a oferta atual, a recompensa nem a velocidade.

O guindaste transporta máquinas para manutenção pela mesma transação de carga. O desenho não promete reparação de via, acidente, resgate interativo, içamento animado ou vantagem em rampa. A feira não abre comércio; gado e gás não acrescentam desgaste, risco ou prazo. Esses comportamentos não existem nesta entrega.

## Domínio, catálogo e compatibilidade

- Acrescentadas oito variantes a `ServiceWagon`, `CoachStyle` e `CargoKind`, com conversões nos dois sentidos e listas `ALL`. Catálogo espelhado por paridade de nomes, capacidades, marcos e compatibilidades para todos os modelos/cargas.
- `CargoKind::SPECIALIZED` contém as oito cargas novas após as quatro anteriores. A seleção existente considera somente especialidades equipadas com capacidade para a oferta. Equipar apenas material anterior conserva a sequência anterior, inclusive a primeira leva: a lista filtrada mantém valores e ordem.
- Uma oferta já existente não muda ao equipar. Termine, recuse ou devolva pelo fluxo normal; a próxima oferta considera a composição. Despacho manual/automático, carregamento na origem, entrega no destino, bloqueio de retirada de capacidade ocupada e devolução permanecem transações do `World`.
- Ofertas continuam entre 4 e 12 unidades, com os mesmos dez pontos de registro por unidade e sem prazo. Os pontos existentes não são saldo gastável. Não há moeda, saldo ou mecanismo de caixa acrescentado.
- Os marcos usam o contador durável de chegadas existente. Não há campo novo, inventário, alteração de versão do catálogo nem migração de checkpoint: o formato continua v4. Os identificadores anteriores não foram renomeados ou removidos. Executáveis anteriores que desconheçam as variantes novas não conseguem abrir esses saves; seu caminho de leitura deve recusá-los e preservá-los, sem conversão silenciosa.
- O renderer não lê catálogo, métricas, disco nem relógio. Os sprites são estáticos; rodas usam a fase visual existente. Nenhum movimento decorativo novo exige ligação à preferência de reduzir movimento.

## Desenho e miniaturas

O corpo cabe no recorte atual de 42 × 28 pixels nativos: engates em `x−2..x+39`, rodas em `x+7` e `x+30`, piso da roda em `y=57`, corpo a partir de `y=32`. O chassi continua compartilhado. Não há mudança no socket da locomotiva, fumaça, trilho, grade de 320 × 70, filtro nearest ou wrap da paisagem.

As oito silhuetas incluem vazios e alturas diferentes, não só troca de cor. As mesmas rotinas desenham a faixa e a galeria da oficina. A única mudança em `src/ui_stock.rs` é derivar o comprimento da constante de miniaturas de `CoachStyle::ALL.len()`; criação/upload continuam ocorrendo uma vez por galeria. Layouts e telas não foram redesenhados.

## Imagens de conferência

As folhas são exportações headless do renderer desta fonte, sem abrir o perfil real ou consultar rede. Colunas: dia, entardecer, noite e chuva. Linhas na ordem da tabela acima.

- `agente5-vagoes-final.png`: cenas completas rotuladas, 2560 × 1120, nearest 2×.
- `agente5-miniaturas-final.png`: recortes reais das miniaturas, 672 × 896, nearest 4×; rodas e engates completos.

Reprodução com nomes de destino novos, sempre uma invocação de compilação por vez e até dois jobs:

```powershell
$env:CARGO_NET_OFFLINE = 'true'
& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','stock_gallery','-j','2','--','docs/conteudo/nova-folha.ppm','--segunda')
node scripts/content-png.cjs docs/conteudo/nova-folha.ppm docs/conteudo/nova-folha.png 2
& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','stock_gallery','-j','2','--','docs/conteudo/novos-recortes.ppm','--recortes')
node scripts/content-png.cjs docs/conteudo/novos-recortes.ppm docs/conteudo/novos-recortes.png 4
```

Sem opção, `stock_gallery` conserva o formato de cenas anterior e percorre todos os 19 vagões. Os modos novos selecionam somente esta leva. Títulos são sobrepostos pelo exportador com a fonte original do jogo, fora da área dos sprites; não alteram o renderer.

## Pedidos para integração

1. Mostrar por espécie a capacidade e a função na oficina, usando `CargoKind::ALL`, `cargo_capacity`, `name` e `wagon_hint`. A prévia atual ainda resume encomendas/suprimentos e líquidos. Não interpretar o total de carga como capacidade intercambiável.
2. Mostrar `required_stops`/`available` antes de equipar; o domínio já recusa o vagão bloqueado. As telas atuais ainda não antecipam todos os requisitos na ficha.
3. Criar ícones por carga na UI futura. O fallback atual de carga seca é genérico; nomes e validação do domínio já distinguem as oito cargas.
4. Confirmar raridades e marcos da tabela antes de substituir a obtenção provisória. Não exigir uma especialidade rara em objetivo obrigatório sem caminho determinístico para obtê-la. Nenhuma obrigação nova foi criada aqui.
5. Contadores duráveis por carga, propriedade de vagões, contratos especiais e economia precisam de projeto de save próprio. Não deduzir entregas por tipo do diário limitado, nem usar o campo de pontos existente como saldo.

Resultados efetivamente executados e limites de validação constam no relato do Agente 5 ao fim de `docs/RODADA_CONTEUDO.md`. As imagens não comprovam compositor, legibilidade na faixa real de 60 pixels físicos ou comportamento em DPI misto.
