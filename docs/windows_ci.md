# CI Windows do projeto consolidado

O workflow `.github/workflows/windows.yml` compila e testa o aplicativo que integra Windows, arte, mundo, métricas e interface. Ele foi preparado nesta consolidação; nenhum job de GitHub Actions foi disparado por esta entrega.

## Uso

Coloque o conteúdo de `cpu-express/` na raiz de um repositório GitHub. O workflow responde a push, pull request ou acionamento manual em Actions. Ele usa Windows Server 2022, o alvo `x86_64-pc-windows-msvc` e a versão fixa de Rust de `rust-toolchain.toml`. Os commits das duas ações são preservados da entrega Windows original.

O mesmo processo pode ser executado em um Windows com Rust, rustfmt, clippy e a toolchain MSVC já instalados:

```powershell
.\scripts\build-windows.ps1
```

O script executa formatação, Clippy com warnings tratados como erro, testes nativos e build release com dependências travadas pelo lockfile. Compara o hash de `Cargo.lock` antes/depois, verifica a existência do novo executável e só então cria um ZIP. As falhas interrompem o empacotamento.

## Conteúdo do pacote

- Executável novo, proveniente do build desta fonte.
- SHA-256, versão real do compilador, alvo e comandos registrados em `BUILD-INFO.json`.
- Licenças e avisos das dependências, com verificação de versões, SPDX declarado, tamanho e hash de cada texto.
- `LICENSE-AUDIT.json`, que compara o inventário fornecido com o grafo Cargo efetivamente resolvido para o alvo.
- Guia de uso, checklist Windows e script de medição.

O inventário da frente Windows, produzido para GNU Windows x64, foi ampliado na consolidação com os dois textos da crate específica de MSVC que faltava. A união contém 144 pacotes e 328 arquivos de avisos. O script aceita o inventário somente quando cada dependência do alvo solicitado tem exatamente a mesma identificação e metadados de licença, e todos os textos permanecem intactos. Também exige avisos Rust da mesma versão do compilador. Avisos MinGW conservam sua proveniência original; não representam licenciamento MSVC/SDK. Uma lacuna interrompe o pacote e pede a atualização dos avisos.

Os relatórios e executáveis de entregas anteriores não são copiados como evidência deste build. A demonstração `windows_desktop` permanece disponível em `examples/` para diagnóstico separado, mas não ocupa o lugar do jogo integrado no pacote.

## O que a aprovação significa

O job comprova os comandos automatizados que executou. Não abre nem inspeciona a GUI; foco, bandeja, Explorer, DPI e múltiplos monitores ainda exigem o [checklist Windows](windows_checklist.md) em Windows 10/11. Não há assinatura digital, criação de release, instalação, envio de mensagens ou publicação de site. Os artefatos de teste e logs ficam associados ao job por 14 dias.

O token tem `contents: read`, o checkout não persiste credenciais, nenhum segredo do projeto é utilizado e o workflow não usa `pull_request_target`.
