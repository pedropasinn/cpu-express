# Vida e movimento na paisagem — Agente 6

Arte original em Rust, `kind: fictional`, de 2026-11-15. São vinhetas de ambientação, sem reprodução de construções reais, fatos históricos, recursos externos ou efeitos no serviço ferroviário. Não há itens persistidos, novas capacidades ou recompensas.

## Conteúdo e ritmo

| Elemento | Etiquetas | Movimento e apresentação | Raridade proposta |
| --- | --- | --- | --- |
| Sinal luminoso e braço semafórico | Todas as etiquetas, detalhes Full | Ciclo de 32 s ativos: 24 s com aspecto fechado, 8 s aberto. Cor, posição da luz e braço distinguem os aspectos; sem piscadas. | Comum |
| Passagem de nível e dois veículos esperando | `plain`, `farmland`, `urban`, sem ponte/viaduto/túnel | Cancela fecha durante a passagem visual do trem; veículos permanecem parados atrás dela. | Comum |
| Bando de três pássaros | Campo, mata e água, antes da noite | Quatro poses de asas em 4 s; deslocamento lento de até 12 px para cada lado em 64 s. | Comum |
| Gado malhado | Campo sem água | Cabeça alterna entre olhar e pastar, ciclo de 16 s, cauda com gesto de 1 px. | Comum |
| Cavalo | Campo sem água | Pescoço/crina e silhueta próprios; ciclo de 24 s para pastar e olhar. | Comum |
| Moinho | `farmland` | Corpo de alvenaria, teto e quatro pás; quatro poses de 2 s, equivalentes a um quarto de volta. | Comum |
| Cata-vento de bombeamento | `plain`, `plateau`, sem água nem fazenda | Torre vazada, leme e rotor, mesmo ciclo discreto do moinho. | Comum |
| Barco de cabine | `river`, `coast`, sem porto | Deslocamento de até 8 px para cada lado em 64 s; pequena esteira de 8 s. | Comum |
| Navio cargueiro | `port` | Aproveita a silhueta original do Agente 2; deriva de até 6 px para cada lado em 96 s. | Comum |
| Farol | `coast` | Torre listrada; lanterna acompanha iluminação, feixe curto varia em 16 s à noite. | Comum |
| Fumaça de chaminé | `farmland`, `urban` | Três pequenas marcas sobem até 11 px em 12 s, junto das chaminés existentes. | Comum |
| Roupas no varal | `farmland`, `urban` | Três peças de cores da paleta; balanço de 1 px, ciclo de 8 s. | Comum |
| Pescador sentado | `river`, `coast`, sem porto | Chapéu, píer, vara e linha; ponta muda 1 px em 16 s. | Comum |
| Janelas ao anoitecer | `urban`, `farmland` | Mistura gradual da cor da janela para a luz da paleta, pelo mesmo `dusk_amount` da cena. | Comum |
| Vaga-lumes | `forest`, `river`, à noite | Três pontos por tile, brilho suave em dois níveis (16 s) e deslocamento de 1 px (32 s). | Comum |
| Queda-d’água | `mountain` | Faixa d’água, filetes e espuma; marcas descem em cinco fases ao longo de 4 s. | Comum |
| Neblina no vale | `mountain`, `viaduct`, `plain` | Três linhas esparsas de baixo contraste no entardecer/noite ou em Fog; deriva de 4 px em 64 s. | Comum |

“Campo” significa `plain`, `farmland` ou `plateau`; “água” significa `river`, `coast` ou `port`. Água suprime os animais de pasto e o cata-vento da baixada: uma combinação porto/costa/baixada não coloca gado na água. A ordem e duplicação das etiquetas não alteram a seleção. O bioma continua definindo a paleta pela rota. Névoa e animais são ilustrações temáticas, não afirmações meteorológicas/ecológicas sobre um ponto real.

Todas as raridades são **propostas para confirmação do Pedro**: comum, disponível como ambientação, sem obtenção individual ou benefício mecânico. Frequência visual não é chance de prêmio. Não há raros/épicos nesta leva, nem motivo para exigir esses elementos em objetivos obrigatórios.

## Contratos e integração

O caminho continua sendo o criado pelo Agente 2:

```rust
renderer.set_scenery_context(SceneryContext {
    current: SegmentScenery::from_tags(&trecho_atual.scenery),
    next: SegmentScenery::from_tags(&proximo_trecho.scenery),
});
renderer.set_scenery_life_options(SceneryLifeOptions {
    reduce_motion: preferencia_de_movimento_reduzido,
});
```

Não há relógio próprio: as poses usam `World::elapsed_seconds()`, e o parallax usa `World::scroll_px()`. O renderer não lê Config, catálogo, métricas, arquivos, rede ou relógio do sistema. Não faz sorteios, não altera World e não aloca por frame. Restaurar a mesma entrada reproduz os mesmos pixels, independentemente da ordem de renders.

As etiquetas continuam **sem ligação ao app nesta branch**, como na entrega do Agente 2. A opção de movimento também aguarda o adaptador do integrador. `Config.reduce_menu_motion` atualmente se refere aos menus: confirmar o alcance/texto da preferência e passar o mesmo booleano para `set_passenger_options` e `set_scenery_life_options`. Não clonar Config por frame. Nenhum match exigiu correção em `app*`; não houve permissão implícita para alterar os menus pela Regra 5 revista.

Movimento reduzido fixa a fase decorativa em zero: animais, asas, pás, barcos, navio, feixe, fumaça, varais, pescador, sinais, vaga-lumes, água e névoa ficam em poses estáticas. Todas as cancelas ficam fechadas. A iluminação ainda responde ao contexto, e o deslocamento normal da paisagem acompanha a viagem; esta opção não pausa o jogo nem altera o movimento anterior de nuvens, clima, passageiros ou trem. Pausa/suspensão congelam o World e, portanto, esta ambientação. Low ou detalhes desligados também fixam a fase; Low omite sinais/cancelas, pássaros, fumaça, vaga-lumes e névoa adicional, mantendo as silhuetas principais.

Os desenhos entram na camada intermediária (tiles de 320, razão 1/8) e na camada próxima (320, razão 1/4), antes dos trilhos, estações, clima e trem. O navio continua na camada intermediária. Os deslocamentos locais são limitados e independentes da largura. Períodos espaciais de 2.560 e 1.280 dividem `SCROLL_PERIOD_PX = 184.320`; não há índices de tile variando a arte ao fechar o wrap. O recorte é equivalente em 160, 320, 641 e 1920 colunas. Resolução de 70 linhas, nearest, buffers, rodas, engates, baseline e sockets permanecem.

A cancela fecha quando sua posição na tela cai entre −96 e 285 colunas, uma margem conservadora que cobre o trem normal e sua posição recentrada em viewports estreitos. Fora disso fica levantada; em movimento reduzido sempre fechada. Não há movimento de carros atravessando os trilhos. Os sinais são decoração e não comandam avanço, segurança operacional ou serviço. Sua fase não representa autorização real de circulação. Pontes, túneis e viadutos suprimem a passagem de nível.

Entrada vazia e Legacy conservam o desenho anterior. Manter `SceneryContext::default()` para Aurora e limpar etiquetas ao trocar de linha ou entrar em excursões sem adaptação. Etiquetas explícitas ganham os elementos novos intencionalmente; não se atualizaram hashes antigos para aceitá-los. Novas obras georreferenciadas, fauna por estação, tráfego ou sinalização funcional precisam de especificação separada.

## Imagens de conferência

O exportador existente `scenery_gallery` recebeu `--vida` e `--estatico`. As 16 linhas preservam a ordem do [guia de cenário](CENARIO.md): plain, farmland, forest, mountain, river, bridge, viaduct, tunnel, urban, port, coast, plateau; depois porto/costa/baixada, montanha/mata/rio/ponte, montanha/mata/viaduto/túnel e planalto/fazenda/cidade.

- `agente6-cenarios-final.png`: cenas de 320 × 70, em dia/entardecer/noite/chuva, ampliadas nearest 2×. 2.560 × 2.240.
- `agente6-quadros-final-v2.png`: cenas de 640 × 70, com quatro instantes ativos (0, 2, 8 e 24 s), parallax fixo. 2.560 × 1.120 em pixels nativos; a largura extra deixa uma repetição dos motivos fora da frente do trem. Linhas com `forest`, `coast` ou `mountain` usam noite para conferir vaga-lumes, feixe e névoa; demais linhas usam dia.
- `agente6-estatico-final-v2.png`: mesma folha de quatro instantes, com redução de movimento. Cada linha contém quatro cenas idênticas; luz e deslocamento foram mantidos constantes.

`agente6-cenarios-revisao.png` fica preservada como revisão anterior à supressão de pasto na água. As folhas `agente6-quadros-final.png` e `agente6-estatico-final.png`, anteriores à v2, também foram preservadas: a estação seguinte aparecia na metade direita da vista de 640 colunas. A v2 usa o meio do segundo trecho do Aurora, que mantém ambas as estações fora da imagem; somente o exportador escolhe esse checkpoint, sem alterar a rota. A folha de condições de 320 colunas mantém a distância anterior de 1.280 unidades. A comparação dos PPMs v2 confirmou diferença entre instantes nas 16 linhas animadas e igualdade exata das quatro colunas nas 16 linhas reduzidas.

Nenhuma dessas imagens é captura do aplicativo no desktop. São exportações do renderer real, de checkpoints apenas em memória, sem acesso ao perfil do jogador. A composição pode encobrir elementos pequenos em certos pontos do parallax; eles não passam à frente do trem para ganhar destaque.

Exemplo de reprodução local, uma compilação por vez, no máximo dois jobs e sem rede; usar nomes de saída novos:

```powershell
$env:CARGO_NET_OFFLINE = 'true'
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/dev.ps1 -CargoArgs @('run','--locked','--example','scenery_gallery','-j','2','--','docs/conteudo/nova-vida.ppm','--vida')"
node scripts/content-png.cjs docs/conteudo/nova-vida.ppm docs/conteudo/nova-vida.png
```

Trocar `--vida` por `--estatico` para a folha estática. Sem o argumento, exporta as quatro condições em 320 colunas. O modo anterior `--linha` permanece disponível. Exportador e conversor recusam sobrescrita. Resultados dos gates e limitações estão no relato do Agente 6 ao fim de [RODADA_CONTEUDO.md](../RODADA_CONTEUDO.md).
