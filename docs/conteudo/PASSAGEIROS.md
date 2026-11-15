# Passageiros e vida nas plataformas — Agente 4

2026-11-15. Arte original desenhada em Rust; `PassengerKind::KIND = "fictional"`. Os personagens não representam pessoas reais nem categorias novas do serviço. Nenhum campo de catálogo, configuração ou checkpoint foi acrescentado.

## Tipos e leitura visual

| Tipo | Silhueta e acessórios |
| --- | --- |
| Trabalhador | Capacete, faixa no traje; transporta bagagem durante o embarque. |
| Estudante | Mochila e livro. |
| Familiar | Cabelo preso, bolsa e braço estendido para a criança. |
| Criança | Estatura menor, boné e mochila pequena. |
| Turista | Chapéu, mochila e câmera. |
| Idoso | Cabelo claro e bengala. |
| Viajante | Mala com alça. |
| Carregador | Boné e volumes empilhados; atua como funcionário nos terminais portuários e postos de serviço. |
| Chefe de estação | Quepe, distintivo e bandeirola. |
| Vendedor ambulante | Chapéu e tabuleiro de alimentos; aparece nas paradas de madeira e de planalto. |
| Ciclista | Capacete e bicicleta conduzida pela plataforma. |

As oito ações são esperar, olhar o relógio, andar, acenar, sentar, embarcar, desembarcar e carregar bagagem. Cada ação tem quatro quadros, com pequenas mudanças de braços, passos, cabeça ou altura. A bicicleta acompanha a pessoa a pé. Sentar inclui um banco desenhado pela própria pose, sem depender de um banco na arquitetura. Os quatro trajes reutilizam as cores de telhado, folhagem, água e acabamento da paleta; pele, cabelo e acessórios também usam essa paleta. Iluminação acompanha a estação.

Chuva e tempestade acrescentam guarda-chuva; neve ou `PassengerOptions.cold` acrescentam gorro, cachecol e manga. A qualidade Low preserva essas roupas mesmo quando suprime as partículas de clima. O renderer não deduz temperatura da altitude ou do bioma: frio sem neve é uma indicação explícita do adaptador.

## Serviço, tempo e determinismo

- `Some(StationService)` ativa as pessoas novas, inclusive no caminho já usado pelo aplicativo. `None` conserva a ilustração antiga dos exportadores. Legacy e as excursões mantêm o desenho anterior.
- São até três representantes de embarque e até três de desembarque, como antes. Zero continua zero; esses sprites não são uma contagem individual completa quando há mais de três pessoas. Um funcionário decorativo permanece separado, somente em Full com detalhes.
- A família ocupa **duas** das figuras existentes: familiar e criança, próximos entre si. Um grupo representado por uma única figura nunca recebe uma criança extra. Funcionários não entram nos totais do domínio.
- A escolha dos passageiros e trajes usa seed da jornada, número de chegadas e sentido do serviço (embarque/desembarque). Não depende da ordem dos renders, CPU, relógio externo, FPS ou sorteio persistido. O estilo da estação ajusta a mistura de tipos; no Aurora há uma adaptação visual do bioma existente, sem alterar a rota.
- Os embarcantes aguardam no primeiro quarto do serviço, caminham e terminam atrás do trem. Os desembarcantes saem na primeira metade e aguardam na plataforma. O aceno de partida só começa após o deslocamento inicial, preservando o encontro exato entre aproximação e parada.
- Quadros vêm de `World::elapsed_seconds`, a quatro passos visuais por segundo; deslocamento usa `StationService.progress`, fornecido pelo proprietário do domínio. Pausa congela os dois. Não existe atualização de serviço no desenho.
- Redução de movimento fixa quadro, posição e pose de cada grupo durante o serviço. O embarcante desaparece na conclusão e o desembarcante aparece após o início; são mudanças discretas de estado. O funcionário também fica em pose estática. A opção não congela trem, parallax ou clima.
- Pixels inteiros, framebuffer de altura 70, nearest do consumidor, clipping e ordem de camadas preservados. O trem continua na frente. Não há coleção dinâmica, acesso a arquivos ou alocação no desenho dos passageiros.

## Integração pendente no proprietário da interface

O aplicativo já envia `StationService`, portanto os novos desenhos de serviço entram automaticamente. A Regra 5 revista só permite correções de compilação em matches de `app*`; esta alteração não exigiu nenhuma. **A preferência existente `Config.reduce_menu_motion` ainda não está ligada às pessoas da plataforma.** O integrador deve chamar, junto dos setters de apresentação:

```rust
renderer.set_passenger_options(PassengerOptions {
    reduce_motion: config.reduce_menu_motion,
    cold: /* indicação de frio derivada de condição meteorológica válida */ false,
});
```

Confirmar com Pedro se a preferência hoje descrita como movimento nos menus deve abranger também a plataforma; adaptar seu texto no trabalho de UI. Para frio sem neve, propõe-se casaco abaixo de 12 °C, sem efeito mecânico, com definição de validade/fallback pelo adaptador meteorológico. Esse limiar **não foi implementado**. Sem temperatura disponível, `cold: false`; neve já aciona o agasalho. Repassar a preferência ao alterá-la e ao construir o renderer. Não clonar `Config` por frame.

Para as linhas futuras, manter `set_station_styles` do Agente 3: o público do porto privilegia trabalhadores, o de serra/madeira turistas e ciclistas, e as demais estações usam a mistura geral. Isso é composição artística, não uma estimativa de demanda real. O funcionário varia pelo estilo. Limpar os overrides ao trocar de linha, conforme o contrato existente.

## Galerias locais

O exportador `passenger_gallery` usa a mesma rotina de sprite do renderer e a fonte pixelada do jogo. Nas folhas, cada linha identifica um tipo; os números 0–3 identificam quadros na folha de ações e roupas na folha climática.

- `agente4-passageiros-final.png`: onze tipos × oito ações × quatro quadros, 1664 × 1540, nearest 2×.
- `agente4-clima-final.png`: dia, entardecer, noite, chuva, neve e frio estático; quatro roupas por célula, 1248 × 1540, nearest 2×. A última coluna mostra o mesmo quadro estático em quatro roupas.
- `agente4-estacoes-final.png`: exportador existente `station_gallery --estilos`, seis arquiteturas × dia/entardecer/noite/chuva, 2560 × 840, nearest 2×.
- `agente4-aurora-ramais-final.png`: exportador existente `station_gallery`, quatro estações e dois ramais em aproximação/serviço/partida, 1920 × 840, nearest 2×. Aproximação tem zero passageiros futuros; ramais não reciclam serviço do circuito.

A folha `agente4-passageiros-revisao.png` é a primeira inspeção, preservada como histórico. Não é evidência de um binário distribuído. Os PNGs finais são exportações locais da fonte desta rodada, não capturas do compositor.

Reprodução, usando nomes de saída novos (os exportadores recusam sobrescrever):

```powershell
$env:CARGO_NET_OFFLINE = 'true'
$env:CARGO_BUILD_JOBS = '2'
.\scripts\dev.ps1 -CargoArgs @('run','--locked','--example','passenger_gallery','-j','2','--','docs/conteudo/pessoas-nova.ppm')
.\scripts\dev.ps1 -CargoArgs @('run','--locked','--example','passenger_gallery','-j','2','--','docs/conteudo/clima-nova.ppm','--clima')
.\scripts\dev.ps1 -CargoArgs @('run','--locked','--example','station_gallery','-j','2','--','docs/conteudo/estacoes-nova.ppm','--estilos')
.\scripts\dev.ps1 -CargoArgs @('run','--locked','--example','station_gallery','-j','2','--','docs/conteudo/aurora-nova.ppm')
node scripts/content-png.cjs docs/conteudo/pessoas-nova.ppm docs/conteudo/pessoas-nova.png 2
```

Executar os comandos sequencialmente, nunca mais de uma compilação própria. O resumo dos gates efetivamente executados consta no relato final em `docs/RODADA_CONTEUDO.md`.

## Balanceamento e raridades propostos

**Comum para todos os onze tipos.** Eles são habitantes e viajantes, disponíveis como ambientação, sem desbloqueio ou vantagem de serviço. Nenhuma raridade rara/épica é proposta para pessoas. As quatro roupas também são comuns; eventual coleção de figurinos exige desenho separado. A frequência artística não é raridade de obtenção.

Confirmar: limite atual de três representantes por direção; família com duas figuras; quatro quadros por segundo; gestos pequenos; um funcionário decorativo; distribuição temática por estação; bancos vinculados à pose. Não mudam capacidades, recompensas, tempo ativo, distância, contratos, economia ou formato v4.
