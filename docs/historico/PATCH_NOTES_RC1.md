# CPU Express 0.2.0-rc.1 — consolidação

- Reúne as cinco frentes sobre a linha A, com um único código-fonte e executável.
- Liga a bandeja e o controlador Windows à interface, preservando fechar/ocultar/minimizar como ações distintas.
- Liga rota real e renderer; acrescenta floresta original para cobrir o quarto bioma.
- Expõe estações, progresso, descobertas e conquistas na interface.
- Oferece tema, qualidade, clima, luz, nuvens e cinco tipos de vagão em preferências persistentes.
- Preserva campos JSON desconhecidos inclusive nas preferências visuais aninhadas.
- Corrige descarte de métricas parciais válidas e revalida leituras em cache.
- Suspende simulação/rasterização durante minimização e ocultação; restaura sem acumular tempo ausente.
- Adapta cabeçalho/HUD ao espaço estreito e preserva a proporção da paisagem.
- Corrige fixtures e exportadores de arte que usavam passos rejeitados pelo domínio novo.
- Atualiza CI, empacotamento, documentação e avisos de dependências.

Validação e limites: `docs/VALIDATION.md`. Progresso permanece restrito à sessão. Aceitação Windows real permanece pendente.
