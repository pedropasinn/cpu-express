"""Writes the draft catalogue line for the Vitória a Minas passenger train from derived-path.json."""
import json, math, re, unicodedata

d = json.load(open("derived-path.json", encoding="utf-8"))
stops = d["stops"]


def slug(name):
    name = name.replace("Estação Ferroviária de ", "").replace("Estação Ferroviária ", "")
    plain = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode()
    return re.sub(r"[^a-z0-9]+", "-", plain.lower()).strip("-"), name


stations, segments = [], []
known = True
for i, s in enumerate(stops):
    sid, name = slug(s["name"])
    leg = s["leg_km"]
    if i and (leg is None or (isinstance(leg, float) and math.isnan(leg))):
        known = False
    st = {"id": sid, "name": name, "verified": False, "source_refs": ["osm"],
          "note": f"OSM nó {s['osm']} ({s['kind']})."}
    if known:
        st["km"] = round(s["km"], 1)
    else:
        st["note"] += " Quilômetro em aberto: há interrupção nos trilhos mapeados antes deste ponto."
    stations.append(st)
for i in range(1, len(stops)):
    a, b = stations[i - 1], stations[i]
    leg = stops[i]["leg_km"]
    measured = leg is not None and not (isinstance(leg, float) and math.isnan(leg))
    seg = {"id": f"{a['id']}--{b['id']}"[:48].rstrip("-"), "from": a["id"], "to": b["id"], "biome": "forest",
           "scenery": ["river", "forest"], "adaptation": "simplified", "source_refs": ["osm"]}
    if "km" in a and "km" in b:
        seg["distance_km"] = round(b["km"] - a["km"], 1)
    elif measured:
        seg["distance_km"] = round(leg, 1)
        seg["note"] = "Distância medida neste trecho; a posição acumulada depende das interrupções anteriores."
    else:
        seg["note"] = "Sem medida: os trilhos mapeados no OpenStreetMap estão interrompidos neste trecho."
    segments.append(seg)
line = {
    "line_format": 1, "id": "vitoria-minas", "name": "Estrada de Ferro Vitória a Minas", "status": "draft",
    "kind": "real", "layout": "shuttle",
    "era": {"label": "Atual: paradas do trem de passageiros conforme o OpenStreetMap em 2026-11-14", "from_year": 2026, "to_year": 2026},
    "gauge_mm": 1000, "country": "BR", "data_license": "ODbL-1.0", "attribution": "© OpenStreetMap contributors",
    "sources": [
        {"id": "osm", "title": "OpenStreetMap, extratos Overpass de 2026-11-14: relação 3331630 (trem de passageiros da EFVM, Pedro Nolasco a Belo Horizonte, operador Vale), relações 7831016 e 7754552 e trilhos do corredor. Cópia em data/sources/vitoria-minas/", "url": "https://www.openstreetmap.org/relation/3331630", "accessed": "2026-11-14", "license": "ODbL-1.0"}
    ],
    "stations": stations, "segments": segments,
    "open_questions": [
        "Trilhos mapeados interrompidos entre Ipaba e Intendente Câmara e entre Rio Piracicaba e Dois Irmãos: o quilômetro das paradas depois de Ipaba fica em aberto até haver fonte.",
        "Extensão: 647 km medidos nos trechos mapeados, sem contar as duas interrupções; conferir com a extensão publicada pelo operador.",
        "Conferir a lista e a ordem das paradas com a tabela do operador; a relação do OpenStreetMap traz um aviso de que está incompleta.",
        "Altitude, bioma e cenário por trecho ainda não levantados: os valores do arquivo são provisórios.",
        "O motor de viagem ainda não percorre linhas de ponta a ponta."
    ],
}
json.dump(line, open("line.json", "w", encoding="utf-8"), ensure_ascii=False, indent=2)
ids = [s["id"] for s in stations]
assert len(set(ids)) == len(ids), ids
assert len({s["id"] for s in segments}) == len(segments)
print(len(stations), "stations;", sum("km" in s for s in stations), "with km;", sum("distance_km" in s for s in segments), "segments measured")
for s in stations:
    print(s.get("km"), s["id"], "|", s["name"])
