# Primeira abertura e aviso do Windows

Texto curto para incluir no próximo pacote autorizado (nenhum pacote foi gerado nesta rodada):

> O CPU Express é distribuído sem assinatura digital. Ao abrir o executável baixado, o Windows pode mostrar um aviso do SmartScreen de aplicativo não reconhecido ou editor desconhecido. Confirme que recebeu o arquivo da origem esperada antes de decidir continuar. Não é necessário desativar o antivírus nem o SmartScreen. Em computadores gerenciados, a política da organização pode impedir a execução. Uma futura assinatura identifica o editor, mas não garante a ausência desse aviso.

Base: [Microsoft — reputação do SmartScreen para desenvolvedores](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation), consultada em 2026-11-14. O aviso de reputação não é, por si só, comprovação de segurança ou de malware.

Na fonte posterior à rc.2, sem `config.json`, a central abre uma acolhida curta para informar cidade/coordenadas do clima (ou manter São Paulo) e escolher faixa ou janela flutuante. Não há GPS nem localização por IP; o nome da cidade não é geocodificado. **Começar viagem** confirma e salva. Fechar a acolhida antes de confirmar não grava preferências; ela volta na próxima abertura. Configuração inválida ou inacessível continua recebendo o aviso de recuperação, sem ser tratada como perfil novo.

A faixa conserva três tentativas com intervalo de dois segundos. Além do retorno do posicionamento, verifica o retângulo obtido e, depois da primeira pintura, visibilidade, área cliente e estado de ocultação do DWM. Ao falhar três vezes, abre o modo flutuante e a central com aviso. Ocultar ou minimizar intencionalmente não dispara essa recuperação. Essas consultas não provam que o compositor mostrou todos os pixels: essa confirmação é visual e permanece separada.

## Roteiro nativo

- Usar um caminho de configuração novo; conferir acolhida e ausência de arquivo antes de confirmar.
- Manter São Paulo e escolher flutuante; confirmar, fechar normalmente e reabrir: preferências mantidas, sem acolhida.
- Repetir com outro perfil, outra cidade/coordenadas e faixa; conferir o local exibido em Visual → Clima.
- Ensaiar indisponibilidade do Explorer apenas em ambiente de teste preparado; confirmar fallback e aviso, sem janela inacessível.
- Ocultar/minimizar intencionalmente e aguardar mais de seis segundos: não trocar de modo. Restaurar pela bandeja.
- Em outro computador, registrar separadamente o aviso do SmartScreen e as políticas aplicadas. O aviso real não foi provocado nesta rodada.
