# Cenários por etiqueta — Agente 2

> Atualização E4, 2026-11-16: o aplicativo já recebe etiquetas e movimento desde E5. O conjunto passou a `u32`, com cinco etiquetas novas e 38 construções adicionais; Aurora agora usa essa arte intencionalmente. Consulte [Cenário E4](../expansao/CENARIO_E4.md). O texto abaixo preserva o contrato e as evidências históricas da primeira entrega.

Arte original desenhada em Rust, `kind: fictional`, criada em 2026-11-15. As vinhetas evocam tipos de paisagem; não reproduzem estruturas identificadas, engenharia ou quantidade de túneis da ferrovia real. Nenhum recurso externo, logotipo ou ficha histórica foi usado.

## Entrada e integração

`SegmentScenery::from_tags(&[SceneryTag])` transforma etiquetas já selecionadas pela rota em um conjunto de 12 bits. A ordem e as duplicatas não mudam a imagem. Não há strings, vetores, leitura do catálogo, relógio ou I/O no caminho de desenho. O enum existente do catálogo é apenas o vocabulário da entrada; nenhum dado do catálogo é consultado pelo renderer.

O adaptador da viagem prepara as duas entradas juntas:

```rust
renderer.set_scene_context(Some(contexto_de_biomas));
renderer.set_scenery_context(SceneryContext {
    current: SegmentScenery::from_tags(&trecho_atual.scenery),
    next: SegmentScenery::from_tags(&proximo_trecho.scenery),
});
```

O peso de `SceneContext::blend` é compartilhado: zero mostra o trecho atual, um mostra o próximo. Etiquetas diferentes também transitam quando o bioma é igual. A estrutura não decide qual trecho é o próximo: ida, volta, serviço e excursões continuam pertencendo ao domínio/adaptador. Para uma única paisagem, usar `SceneryContext::new(conjunto)`.

**Aurora permanece opt-in:** o default é vazio e reproduz o caminho anterior, pixel por pixel. O integrador deve manter `SceneryContext::default()` para `aurora-v1`, mesmo que seu catálogo possua etiquetas, e para excursões sem adaptação específica. Ao mudar de linha ou entrar em excursão, atualizar/limpar a entrada para não reutilizar etiquetas antigas. O tema `Legacy` ignora a entrada. Nesta rodada não há ligação com `src/app*` nem seleção de linha jogável.

## Desenho e legibilidade

| Etiqueta | Elementos |
| --- | --- |
| `plain` | Baixada de horizonte baixo, canais rasos e juncos. |
| `farmland` | Lavoura em sulcos, casa rural, silo e cerca. |
| `forest` | Mata de copas largas sobrepostas e troncos, distinta dos pinheiros antigos. |
| `mountain` | Picos recortados, faces sombreadas, encosta verde e mata. |
| `river` | Curso sinuoso, margens com juncos e reflexos. |
| `bridge` | Ponte metálica com treliça posterior e travamento inferior. |
| `viaduct` | Viaduto distante em perspectiva curva; pilares sob a via principal. |
| `tunnel` | Boca de túnel em pedra, arco, aduela central e encosta. |
| `urban` | Prédios de alturas diferentes, janelas e casario de telhado inclinado. |
| `port` | Guindastes treliçados, navio cargueiro, contêineres, cais e mangue. |
| `coast` | Água, banco de areia e manguezal com raízes expostas. |
| `plateau` | Campos elevados e araucárias de copa em guarda-chuva. |

Postes com isoladores e sinais decorativos acompanham as paisagens etiquetadas no modo completo. Os sinais não comandam a viagem e não representam um sistema operacional de sinalização. O modo econômico omite esses detalhes, nervuras e algumas luzes, mantendo os marcos principais.

Túnel é uma **representação em corte**: o trem é desenhado integralmente por cima da boca, sem apagar vagões. A ponte usa a treliça do lado oposto; não há guarda-corpo na frente do trem. Curvas aparecem no viaduto distante; a via do trem permanece horizontal. A composição conserva sockets, rodas, engates e trilhos. As 70 linhas e o filtro nearest permanecem; ampliação de conferência deve ser inteira.

Ordem de desenho de etiquetas combinadas: relevo distante → água e baixada → agricultura → mata → rio → porto/mangue → planalto → cidade → viaduto distante → base da via, estruturas e boca de túnel → trilhos existentes → estação/clima → trem. A ordem das etiquetas recebidas não altera essa precedência. Estruturas podem coexistir; são motivos artísticos repetidos, não contagem de obras reais.

As camadas novas usam tiles de 320 colunas, velocidades de 1/32, 1/8 e 1/4 e períodos de 10.240, 2.560 e 1.280 pixels do mundo. Todos dividem `SCROLL_PERIOD_PX = 184.320`. A largura do viewport não altera esses períodos. As cores vêm da paleta existente e recebem o ciclo de luz e o véu de chuva/neblina do cenário antes do desenho do trem. Na combinação explícita `highland` + `forest`, o terreno usa os tons existentes de floresta, com céu e luz do bioma: encosta úmida em vez da serra árida do Aurora. A entrada vazia não passa por essa adaptação.

## Galerias e reprodução

`scenery_gallery` usa o renderer real, seguindo o exportador `art_review`. A folha contém 16 linhas e quatro colunas: **dia (180), entardecer (348), noite (480), chuva (180)**. Cada célula é 320 × 70, sem suavização. As primeiras 12 linhas seguem a tabela acima; as quatro últimas combinam porto/costa/baixada, montanha/mata/rio/ponte, montanha/mata/viaduto/túnel e planalto/fazenda/cidade.

O argumento opcional `--linha` gera outra folha com os 13 trechos de `curitiba-paranagua`, na ordem do catálogo, de Paranaguá até Curitiba. Usa exatamente as combinações de etiquetas e o bioma de cada trecho, inclusive `highland` na serra, com a adaptação de terreno florestal descrita acima. A consulta ao catálogo embutido acontece no exportador, antes de desenhar. Essa folha não significa que a linha deixou de ser rascunho ou que seu percurso já está ligado ao aplicativo.

As folhas finais usam uma jornada em memória a 1.280 unidades e 60 segundos ativos, sem plataformas visíveis em 320 colunas. Não leem nem gravam o perfil. As prévias `agente2-cenarios.png`, `agente2-cenarios-viagem.png` e `agente2-trechos-viagem.png` foram preservadas como etapas de revisão: a primeira ainda mostrava a estação inicial sobre o cenário; a terceira revelou a necessidade de deixar a serra com mata verde. **Conferir a entrega pelos arquivos `*-final.png`**, gerados novamente da fonte final. Os PPMs intermediários também permanecem locais, ignorados pelo Git existente.

Executar um comando de Cargo por vez, sem rede, com no máximo dois jobs. Os caminhos de saída devem ser novos; os exportadores de cenário e o conversor recusam sobrescrita.

```powershell
$env:CARGO_NET_OFFLINE = 'true'
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','scenery_gallery','-j','2','--','docs/conteudo/agente2-cenarios-final.ppm')"
node scripts/content-png.cjs docs/conteudo/agente2-cenarios-final.ppm docs/conteudo/agente2-cenarios-final.png
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','scenery_gallery','-j','2','--','docs/conteudo/agente2-trechos-final.ppm','--linha')"
node scripts/content-png.cjs docs/conteudo/agente2-trechos-final.ppm docs/conteudo/agente2-trechos-final.png
```

Resultados executados, nomes das imagens e limites de conferência ficam no relato do Agente 2 em `docs/RODADA_CONTEUDO.md`. Imagem headless não comprova compositor, DPI, animação no desktop nem leitura numa barra de 60 pixels físicos.

## Decisões para Pedro confirmar

Não há mudança de recompensa, moeda, velocidade, capacidade ou tempo de viagem. As decisões são visuais: túnel em corte para conservar a leitura do trem; ponte com treliça posterior; frequência fixa de marcos no parallax, sem equivalência a quilômetros reais. Para ligar obras a posições precisas ou representar exatamente os 12 túneis, será necessário especificar marcos por posição do trecho em outra rodada, sem inferir isso da repetição da arte.
