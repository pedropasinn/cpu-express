import json, urllib.request
d = json.load(open("derived.json", encoding="utf-8"))
OFFSET = 0.11  # mean difference between the four OSM milestones (68, 73, 77, 81) and the computed distance
want = [("paranagua", "Paranaguá", 7788512109), ("alexandra", "Alexandra", 7788388567), ("saquarema", "Saquarema", 7788425399),
        ("morretes", "Morretes", 7785745224), ("porto-de-cima", "Porto de Cima", 4071710619), ("engenheiro-lange", "Engenheiro Lange", 7801571295),
        ("marumbi", "Marumbi", 6354522329), ("cadeado", "Cadeado", 1116998738), ("veu-de-noiva", "Véu de Noiva", 7801571293),
        ("banhado", "Banhado", 10874965340), ("roca-nova", "Roça Nova", 7801552973), ("piraquara", "Piraquara", 7801607158),
        ("pinhais", "Pinhais", 6247172545), ("curitiba", "Curitiba", 7782327184)]
by = {p[6]: p for p in d["points"]}
st = []
for sid, name, osm in want:
    p = by[osm]
    km = 0.0 if sid == "paranagua" else round(p[0] + OFFSET, 1)
    st.append({"id": sid, "name": name, "km": km, "osm": osm, "lat": p[7], "lon": p[8], "kind": p[3], "ref": p[4], "operator": p[9], "start": p[10]})
url = ("https://api.open-meteo.com/v1/elevation?latitude=" + ",".join(str(s["lat"]) for s in st)
       + "&longitude=" + ",".join(str(s["lon"]) for s in st))
ele = json.load(urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "cpu-express-research/0.4"}), timeout=60))
json.dump({"request": url, "response": ele}, open("elevation.json", "w"), indent=1)
for s, e in zip(st, ele["elevation"]):
    s["ele"] = e
feats = [dict(f, km=round(f["from"] + OFFSET, 2), m=round((f["to"] - f["from"]) * 1000)) for f in d["features"]]
BIO = {"paranagua": ("coast", ["port", "coast", "plain"]), "alexandra": ("coast", ["plain", "forest"]),
       "saquarema": ("forest", ["plain", "forest", "river"]), "morretes": ("forest", ["forest", "river"]),
       "porto-de-cima": ("highland", ["forest", "mountain"]), "engenheiro-lange": ("highland", ["mountain", "forest"]),
       "marumbi": ("highland", ["mountain", "forest"]), "cadeado": ("highland", ["mountain", "forest"]),
       "veu-de-noiva": ("highland", ["mountain", "forest"]), "banhado": ("highland", ["mountain", "forest"]),
       "roca-nova": ("meadow", ["plateau", "forest"]), "piraquara": ("meadow", ["plateau", "farmland"]),
       "pinhais": ("meadow", ["plateau", "urban"])}
segs = []
for a, b in zip(st, st[1:]):
    inside = [f for f in feats if a["km"] <= f["km"] < b["km"]]
    tun = [f for f in inside if f["kind"] == "tunnel"]
    bri = [f for f in inside if f["kind"] == "bridge"]
    bio, sc = BIO[a["id"]]
    sc = list(sc)
    if tun:
        sc.append("tunnel")
    if bri:
        sc.append("bridge")
    if any(f.get("btype") == "viaduct" for f in bri):
        sc.append("viaduct")
    note = (f"OSM: {len(tun)} túnel(is) somando {sum(f['m'] for f in tun)} m e {len(bri)} ponte(s)/viaduto(s) "
            f"somando {sum(f['m'] for f in bri)} m; altitude de {round(a['ele'])} m a {round(b['ele'])} m.")
    if a["id"] == "marumbi":
        note += " Inclui o viaduto de 125 m no km 62,3 do traçado, que coincide com a ponte sobre o Rio São João (km 62,21) do acervo IMS."
    if a["id"] == "banhado":
        note += " Inclui o túnel de Roça Nova, o mais longo do traçado (448 m)."
    segs.append({"id": f"{a['id']}-{b['id']}", "from": a["id"], "to": b["id"], "distance_km": round(b["km"] - a["km"], 1),
                 "biome": bio, "scenery": sc, "adaptation": "simplified", "source_refs": ["osm", "open-meteo-dem"], "note": note})
KIND = {"station": "estação", "halt": "parada", "site": "local ferroviário", "service_station": "posto de serviço"}
stations = []
for s in st:
    note = (f"OSM nó {s['osm']} ({KIND[s['kind']]}" + (f", sigla {s['ref']}" if s["ref"] else "")
            + (f", operador {s['operator']}" if s["operator"] else "") + (f", abertura {s['start']}" if s["start"] else "") + ").")
    if s["id"] == "paranagua":
        note += " Origem da quilometragem; hoje os trilhos mapeados terminam cerca de 0,8 km antes do prédio da estação."
    if s["id"] == "curitiba":
        note += " Estação Rodoferroviária, ponto de partida atual do trem de passageiros; a estação original de 1885 é hoje o Museu Ferroviário."
    stations.append({"id": s["id"], "name": s["name"], "km": s["km"], "elevation_m": round(s["ele"]), "verified": True,
                     "source_refs": ["osm", "open-meteo-dem"], "note": note})
line = {
    "line_format": 1, "id": "curitiba-paranagua", "name": "Estrada de Ferro Paranaguá–Curitiba", "status": "draft",
    "kind": "real", "layout": "shuttle",
    "era": {"label": "Atual: traçado e pontos em operação conforme o OpenStreetMap em 2026-11-14", "from_year": 2026, "to_year": 2026},
    "gauge_mm": 1000, "country": "BR", "data_license": "ODbL-1.0", "attribution": "© OpenStreetMap contributors",
    "sources": [
        {"id": "osm", "title": "OpenStreetMap, extrato Overpass de 2026-11-14 (base 2026-11-14T13:24:21Z): trilhos, pontos operacionais, marcos quilométricos, túneis e pontes. Cópia em data/sources/curitiba-paranagua/", "url": "https://www.openstreetmap.org/copyright", "accessed": "2026-11-14", "license": "ODbL-1.0"},
        {"id": "open-meteo-dem", "title": "Open-Meteo Elevation API (Copernicus DEM GLO-90): altitude nas coordenadas de cada ponto", "url": "https://open-meteo.com/en/docs/elevation-api", "accessed": "2026-11-14"},
        {"id": "wikidata", "title": "Wikidata: estações ferroviárias na região (Marumbi, Morretes, Engenheiro Lange, Banhado, Rodoferroviária de Curitiba), usadas para conferir nomes e posições", "url": "https://www.wikidata.org/", "accessed": "2026-11-14"},
        {"id": "ims-ferrez", "title": "Instituto Moreira Salles, acervo: Estrada de Ferro Paranaguá-Curitiba (linha de 110 km; ponte sobre o Rio São João no km 62,21; As Três Pontes no km 61,72)", "url": "https://acervos.ims.com.br/index.php/Detail/objects/191295?lang=pt_BR", "accessed": "2026-11-14"},
        {"id": "aen-pr", "title": "Agência Estadual de Notícias do Paraná: 13 túneis, 30 pontes e viadutos, 10 estações intermediárias; construção de 1880 a 1885", "url": "https://www.parana.pr.gov.br/aen/Noticia/Viagem-de-trem-mais-bonita-do-mundo-e-Maria-Fumaca-Caicara-sao-opcoes-turisticas-do", "accessed": "2026-11-14"},
        {"id": "uepg-estacoes", "title": "Repositório UEPG: estudo sobre as catorze estações da ferrovia em seis municípios", "url": "https://ri.uepg.br/bitstreams/c6fb937c-a9c9-45de-8aba-607c6b67b743/download", "accessed": "2026-11-14"},
    ],
    "stations": stations, "segments": segs,
    "open_questions": [
        "O motor de viagem ainda não percorre linhas de ponta a ponta (shuttle) nem mais de quatro estações; a linha fica como rascunho até o checkpoint v4.",
        "Serviço de passageiros atual: confirmar em fonte do operador quais pontos têm embarque hoje (o trem turístico parte de Curitiba e vai a Morretes); os demais são pontos operacionais de carga.",
        "Demanda de passageiros e tipos de carga por estação ainda não definidos (porto em Paranaguá, turismo em Morretes e Marumbi).",
        "Cenário por trecho: biomas e etiquetas são leitura editorial dos dados; os marcos visuais de cada trecho ainda não foram desenhados.",
    ],
}
json.dump(line, open("line.json", "w", encoding="utf-8"), ensure_ascii=False, indent=2)
for s in stations:
    print(s["km"], s["elevation_m"], s["name"])
for g in segs:
    print(g["id"], g["distance_km"], g["scenery"])
print("tunnels", sum(f["kind"] == "tunnel" for f in feats), "m", sum(f["m"] for f in feats if f["kind"] == "tunnel"),
      "bridges", sum(f["kind"] == "bridge" for f in feats))
