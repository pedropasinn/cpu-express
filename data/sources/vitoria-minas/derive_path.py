"""Vitória a Minas: distance of each passenger stop along the mapped main tracks.

Inputs: osm-route.json (route relation 3331630, ordered stops) and osm-corridor.json
(main and branch tracks plus station nodes of the corridor). Output: derived-path.json.
"""
import heapq, json, math


def hav(a, b):
    R = 6371.0088
    p1, p2 = math.radians(a[0]), math.radians(b[0])
    dl = math.radians(b[1] - a[1])
    h = math.sin((p2 - p1) / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    return 2 * R * math.asin(math.sqrt(h))


route = json.load(open("osm-route.json", encoding="utf-8"))
corridor = json.load(open("osm-corridor.json", encoding="utf-8"))
rel = [x for x in route["elements"] if x["type"] == "relation"][0]
rnodes = {x["id"]: x for x in route["elements"] if x["type"] == "node"}
stops = [rnodes[m["ref"]] for m in rel["members"] if m["type"] == "node" and m["ref"] in rnodes]
terminal = [x for x in corridor["elements"] if x["type"] == "node" and x["id"] == 12486937155]
stops += terminal

pos, adj, edge = {}, {}, {}


def key(p):
    return (round(p["lat"], 7), round(p["lon"], 7))


def add(points, pen, way):
    for a, b in zip(points, points[1:]):
        pos[a] = a
        pos[b] = b
        L = hav(a, b)
        adj.setdefault(a, []).append((b, L * pen, L))
        adj.setdefault(b, []).append((a, L * pen, L))
        if way is not None or (a, b) not in edge:
            edge[(a, b)] = edge[(b, a)] = way


# Tracks listed by the route relations close the stretches that carry no usage tag.
for name in ("osm-route.json", "osm-more.json"):
    for r in [x for x in json.load(open(name, encoding="utf-8"))["elements"] if x["type"] == "relation"]:
        for m in r["members"]:
            if m["type"] == "way" and m.get("geometry"):
                add([key(p) for p in m["geometry"]], 1.2, None)
gaps = json.load(open("osm-gaps.json", encoding="utf-8"))
for w in gaps["elements"] + corridor["elements"]:
    if w["type"] != "way":
        continue
    t = w["tags"]
    add([key(p) for p in w["geometry"]], 1.0 if t.get("usage") == "main" and not t.get("service") else 2.5, w)
ids = list(pos)


def nearest(pt):
    return min(ids, key=lambda n: (pos[n][0] - pt[0]) ** 2 + (pos[n][1] - pt[1]) ** 2)


def shortest(a, b):
    dist, prev, pq = {a: 0.0}, {}, [(0.0, a)]
    while pq:
        c, u = heapq.heappop(pq)
        if u == b:
            path = [b]
            while path[-1] != a:
                path.append(prev[path[-1]])
            return path[::-1]
        if c > dist.get(u, 1e18):
            continue
        for v, wt, _ in adj[u]:
            nc = c + wt
            if nc < dist.get(v, 1e18):
                dist[v] = nc
                prev[v] = u
                heapq.heappush(pq, (nc, v))
    return None


snapped = [nearest((s["lat"], s["lon"])) for s in stops]
total, rows, full = 0.0, [], []
for i, s in enumerate(stops):
    leg = None
    if i:
        path = shortest(snapped[i - 1], snapped[i])
        if path is None:
            # The mapped tracks are interrupted here: the distance stays unknown.
            print("  NO PATH to", s["tags"].get("name"), "straight km", round(hav(pos[snapped[i - 1]], pos[snapped[i]]), 1))
            leg = float("nan")
            full.append(snapped[i])
        else:
            leg = sum(hav(pos[a], pos[b]) for a, b in zip(path, path[1:]))
            full.extend(path if not full else path[1:])
        total += leg
    t = s.get("tags", {})
    rows.append({"osm": s["id"], "name": t.get("name"), "km": round(total, 2), "leg_km": None if leg is None else round(leg, 2),
                 "off_m": round(hav(pos[snapped[i]], (s["lat"], s["lon"])) * 1000), "lat": s["lat"], "lon": s["lon"], "kind": t.get("railway")})
    print(rows[-1]["km"], rows[-1]["leg_km"], rows[-1]["off_m"], rows[-1]["name"])
km = [0.0]
for a, b in zip(full, full[1:]):
    km.append(km[-1] + (hav(pos[a], pos[b]) if (a, b) in edge else 0.0))
feats, cur = [], None
gauges = {}
for i, (a, b) in enumerate(zip(full, full[1:])):
    w = edge.get((a, b)) or {"id": 0, "tags": {"gauge": "lacuna" if (a, b) not in edge else "sem etiqueta"}}
    t = w["tags"]
    gauges[t.get("gauge")] = gauges.get(t.get("gauge"), 0) + km[i + 1] - km[i]
    kind = "tunnel" if t.get("tunnel") == "yes" else "bridge" if t.get("bridge") and t.get("bridge") != "no" else None
    if kind:
        if cur and cur["way"] == w["id"] and w["id"]:
            cur["to"] = km[i + 1]
        else:
            cur = {"kind": kind, "way": w["id"], "from": km[i], "to": km[i + 1], "name": t.get("name"), "btype": t.get("bridge")}
            feats.append(cur)
    else:
        cur = None
print("length", round(km[-1], 1), "tunnels", sum(f["kind"] == "tunnel" for f in feats), "bridges", sum(f["kind"] == "bridge" for f in feats))
print({k: round(v, 1) for k, v in gauges.items()})
json.dump({"stops": rows, "features": feats, "path": [[pos[n][0], pos[n][1], round(k, 3)] for n, k in zip(full, km)]},
          open("derived-path.json", "w", encoding="utf-8"), ensure_ascii=False)
