# Origem dos avisos do projeto consolidado

O inventário reúne **144 pacotes de registry e 328 arquivos de avisos/licenças** da união dos alvos GNU Windows x64 e MSVC Windows x64. A frente Windows forneceu 143 pacotes / 326 textos; sua integridade foi conferida contra o manifesto original. A comparação com `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc` encontrou uma dependência adicional: `windows_x86_64_msvc 0.52.6`. Seus textos MIT e Apache-2.0 foram copiados sem alteração do pacote exato já disponível no registry local.

Nenhuma consulta de rede foi necessária para esta ampliação. Os arquivos fornecidos e suas atribuições foram preservados; o índice, fontes, tamanhos e SHA-256 estão em `licenses/manifest.json`. Os avisos da biblioteca padrão correspondem a Rust 1.100.0. Avisos MinGW mantêm suas fontes originais e não são apresentados como avisos da toolchain MSVC.

O script de build compara as versões/SPDX de cada dependência resolvida do alvo solicitado com este inventário e verifica os 328 textos antes de empacotar. Uma ausência, alteração de arquivo ou divergência de versão interrompe o pacote. A cobertura do grafo e a integridade dos textos não afirmam quais símbolos contribuíram para o binário, nem substituem os termos originais das licenças.
