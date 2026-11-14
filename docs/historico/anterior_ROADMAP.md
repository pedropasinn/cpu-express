# CPU Express — evoluções posteriores à versão 0.1

Este roteiro registra ideias para versões futuras. Não descreve funcionalidades já entregues e não estabelece datas.

A prioridade da versão 0.1 é fechar o fluxo essencial: janela compacta Windows, trem animado e velocidade vinculada à CPU real, com núcleo testável e coleta independente da renderização. A validação em Windows real e qualquer correção necessária desse fluxo têm precedência sobre os itens seguintes.

## Versão 0.2 — Aparência

- Sprites originais mais elaborados, preservando leitura em uma janela pequena.
- Diferentes biomas e paisagens.
- Ciclo de dia e noite.
- Efeitos climáticos.
- Janela transparente, caso seja tecnicamente adequada e confiável.

**Critério orientador:** melhorar a aparência sem prejudicar consumo de recursos, controles ou previsibilidade da janela. Transparência deve ser avaliada no Windows de destino antes de se tornar padrão.

## Versão 0.3 — Computador como mundo

- Vagões representando utilização de RAM.
- Atividade de disco gerando cargas.
- Processos representados como passageiros.
- Indicadores de rede como entregas.
- Temperatura da CPU como elemento visual, quando a métrica estiver disponível.

**Critério orientador:** cada metáfora deve indicar claramente o que a métrica mede e o que é interpretação visual. Ausência de um sensor não deve impedir a viagem. Os provedores continuam separados da simulação.

## Versão 0.4 — Jogo

- Distância percorrida apresentada ao usuário.
- Estações.
- Melhorias da locomotiva.
- Pequenas conquistas.
- Histórico local.

**Critério orientador:** acrescentar progressão ao companheiro de desktop mantendo o funcionamento offline. O deslocamento interno usado para renderizar a cena na 0.1 não equivale a um histórico persistente de distância percorrida.

## Versão 0.5 — Linux

- Implementação específica da integração com o desktop.
- Testes em X11 e Wayland.
- Suporte aos diferentes comportamentos dos compositores.
- Empacotamento adequado.

**Critério orientador:** estabelecer uma experiência Linux validada, com documentação das diferenças de posicionamento, painéis e regras de janela. A feature `host-validation` da 0.1 é uma ferramenta de laboratório e não representa esse porte concluído.

## O que fica preservado entre versões

- Arte original ou recursos com origem e licença verificáveis.
- Núcleo de simulação independente de APIs Windows.
- Coleta que não bloqueia a thread de renderização.
- Métricas ausentes tratadas explicitamente.
- Experimentos com CPU simulada disponíveis sem estressar a máquina.
- Medidas de desempenho identificando ambiente e método, sem resultados presumidos.
