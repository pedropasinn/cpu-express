# CPU Express 0.4.0-rc.3

Jogo ferroviário de desktop para Windows, com faixa na barra de tarefas, janela flutuante e menus independentes.

## Usar

Abra `cpu-express.exe` na distribuição portátil rc.3. Clique no trem para abrir a central; botão direito pausa/retoma. Não há botão do jogo na barra de tarefas; use a faixa e o ícone da bandeja.

- [Guia atual](docs/GUIA_04.md): passageiros, despacho automático/manual, devoluções, vagões, locomotivas, excursões, clima e saves.
- [Validação rc.3 e limites](docs/VALIDACAO_RC3.md): verificações desta distribuição. A validação rc.2 fica como histórico.
- [Plano de execução](docs/PLANO_EXECUCAO.md): entregas e pendências.
- [Prioridades](docs/ROADMAP.md).
- [Primeira abertura e SmartScreen](docs/windows_primeira_abertura.md): acolhida e comportamento desta rc.3.

## Desenvolver

Na branch `expansao`, Curitiba–Paranaguá já é jogável: **Viagem → Linhas**. Use **Encerrar embarques**, aguarde o trem vazio numa parada elegível e escolha **Viajar nesta linha**. A fonte de desenvolvimento usa checkpoint v6 com migração e backup v1–v5; a distribuição rc.3 abaixo permanece histórica. [Guia da integração E3](docs/expansao/LINHAS_E3.md).

Esta é a única base atual de código. `src/` contém domínio, renderer, UI e integração Windows; `tests/` e `examples/` fornecem verificações e exportadores. `scripts/dev.ps1` encontra a toolchain local sem alterar o PATH permanentemente. Cargo.lock fixa as dependências.

```powershell
.\scripts\build-rc3.ps1
.\scripts\dev.ps1 run --locked
```

Os caches de compilação são recriados quando necessários; a primeira compilação após a limpeza leva mais tempo. `target/rc3/release/cpu-express.exe` mantém uma cópia do binário validado para empacotamento. `evidence/rc3/` contém as evidências desta versão; `evidence/current/` conserva registros históricos da rc.2.

O pacote portátil, o código-fonte e as evidências são distribuídos em ZIPs separados. Nesta rodada, `scripts/package-rc3.ps1` e `scripts/validate-portable.ps1` adaptam o fluxo de package-current.py/validate-portable.py para PowerShell, pois o Python local não pôde ser executado. Exigem hashes das fontes e do binário e usam perfil isolado. Licenças em LICENSE, THIRD_PARTY_NOTICES.md, licenses/ e data/sources/curitiba-paranagua/README.md. Para o teste com amigos, veja [ROTEIRO_BETA.md](docs/ROTEIRO_BETA.md).

## Estado e dados

O progresso normal fica em `%LOCALAPPDATA%\CPU Express`, separado deste projeto. O formato v4 registra linha e versão do catálogo e migra v1/v2/v3 com backup exato (inclusive `.v3.backup.json`); não substitua saves por arquivos antigos. A CPU afeta a velocidade visual, não a progressão lógica. Pausa e fechamento não acumulam progresso offline.
