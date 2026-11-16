# CPU Express — validação local dos sprites SVG originais. Licença MIT.
# Uso: powershell -NoProfile -File assets/icons/validar.ps1
$ErrorActionPreference = 'Stop'
$nomes = @(
    'locomotiva','mapa','oficina','engrenagem','ajuda','fechar','passageiro',
    'caixote','tanque','encomenda','bilhete','estacao','relogio','estrela',
    'cadeado','confirmar','pausar','retomar','salvar','sol','nuvem','chuva',
    'cpu','memoria','pontos','postal','porto','montanha','tunel','ponte',
    'bandeira','som','mudo','dica','seta-esquerda','seta-direita','despacho',
    'devolver','vagao-passageiros','vagao-dormitorio','vagao-panoramico',
    'vagao-carga','vagao-tanque',
    'moeda','pilha-moedas','saco-moedas','caixa-patio','caixa-oficina',
    'caixa-plataforma','caixa-garagem','caixa-atelie'
)
$paleta = @('#342016','#794E2D','#B27C48','#FFF6DE','#EBDAB5','#F6D080',
    '#BF8D37','#1C7064','#104640','#2F647E','#B23A30','#7FB4C8','#5B8C4A')
$arquivos = @(Get-ChildItem -LiteralPath $PSScriptRoot -Filter '*.svg' -File)
if ($arquivos.Count -ne $nomes.Count) { throw "Esperados $($nomes.Count) SVGs, encontrados $($arquivos.Count)." }
$total = 0
$maiorPaleta = 0
foreach ($nome in $nomes) {
    $arquivo = Join-Path $PSScriptRoot "$nome.svg"
    if (!(Test-Path -LiteralPath $arquivo)) { throw "Ausente: $nome" }
    $config = New-Object System.Xml.XmlReaderSettings
    $config.DtdProcessing = [System.Xml.DtdProcessing]::Prohibit
    $config.XmlResolver = $null
    $leitor = [System.Xml.XmlReader]::Create($arquivo, $config)
    $doc = New-Object System.Xml.XmlDocument
    $doc.XmlResolver = $null
    try { $doc.Load($leitor) } finally { $leitor.Dispose() }
    $raiz = $doc.DocumentElement
    if ($raiz.LocalName -ne 'svg' -or $raiz.NamespaceURI -ne 'http://www.w3.org/2000/svg' -or
        $raiz.GetAttribute('viewBox') -cne '0 0 16 16' -or
        $raiz.GetAttribute('shape-rendering') -cne 'crispEdges') { throw "$nome`: raiz incompatível." }
    foreach ($atributo in $raiz.Attributes) {
        if ($atributo.Name -notin @('xmlns','viewBox','shape-rendering')) { throw "$nome`: atributo extra na raiz." }
    }
    $cores = @{}
    $pixels = New-Object 'string[,]' 16,16
    $retangulos = 0
    foreach ($no in $raiz.ChildNodes) {
        if ($no.NodeType -eq [System.Xml.XmlNodeType]::Whitespace) { continue }
        if ($no.NodeType -ne [System.Xml.XmlNodeType]::Element -or $no.LocalName -ne 'rect' -or
            $no.NamespaceURI -ne $raiz.NamespaceURI -or $no.HasChildNodes) { throw "$nome`: somente rect vazio é permitido." }
        if ($no.Attributes.Count -ne 5) { throw "$nome`: rect deve conter exatamente x, y, width, height, fill." }
        foreach ($atributo in $no.Attributes) {
            if ($atributo.Name -notin @('x','y','width','height','fill')) { throw "$nome`: atributo proibido." }
        }
        foreach ($campo in @('x','y','width','height')) {
            if ($no.GetAttribute($campo) -cnotmatch '^(0|[1-9][0-9]*)$') { throw "$nome`: $campo não é inteiro não negativo." }
        }
        $x=[int]$no.GetAttribute('x'); $y=[int]$no.GetAttribute('y')
        $w=[int]$no.GetAttribute('width'); $h=[int]$no.GetAttribute('height')
        if ($w -lt 1 -or $h -lt 1 -or $x+$w -gt 16 -or $y+$h -gt 16) { throw "$nome`: rect fora da grade." }
        $cor=$no.GetAttribute('fill')
        if ($paleta -cnotcontains $cor) { throw "$nome`: cor não permitida: $cor" }
        $cores[$cor]=$true
        for ($j=$y; $j -lt $y+$h; $j++) {
            for ($i=$x; $i -lt $x+$w; $i++) {
                if ($pixels[$i,$j]) { throw "$nome`: retângulos sobrepostos." }
                $pixels[$i,$j]=$cor
            }
        }
        $retangulos++
    }
    if ($retangulos -eq 0 -or $cores.Count -gt 6 -or !$cores.ContainsKey('#342016')) { throw "$nome`: desenho vazio, sem contorno ou com mais de 6 cores." }
    $vazios=0
    for ($j=0; $j -lt 16; $j++) {
        for ($i=0; $i -lt 16; $i++) {
            if (!$pixels[$i,$j]) { $vazios++; continue }
            if ($pixels[$i,$j] -eq '#342016') { continue }
            # Um pixel colorido não pode tocar transparência ou sair da grade:
            # a silhueta externa deve conservar a borda escura de um pixel.
            foreach ($passo in @(@(1,0),@(-1,0),@(0,1),@(0,-1))) {
                $nx=$i+$passo[0]; $ny=$j+$passo[1]
                if ($nx -lt 0 -or $nx -ge 16 -or $ny -lt 0 -or $ny -ge 16 -or !$pixels[$nx,$ny]) {
                    throw "$nome`: pixel colorido sem contorno em ($i,$j)."
                }
            }
        }
    }
    if ($vazios -eq 0) { throw "$nome`: não há transparência." }
    $total += $retangulos
    $maiorPaleta = [Math]::Max($maiorPaleta,$cores.Count)
}
Write-Output "APROVADO: $($nomes.Count) SVGs; $total rects inteiros, sem sobreposição; grade 16x16; somente paleta permitida; máximo $maiorPaleta cores; transparência e contorno externo verificados."
