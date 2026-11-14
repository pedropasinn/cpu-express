import json, math, heapq, sys
d=json.load(open('extract.json',encoding='utf-8'))
d2=json.load(open('extract2.json',encoding='utf-8'))
def hav(a,b):
    R=6371.0088; p1,p2=math.radians(a[0]),math.radians(b[0]); dl=math.radians(b[1]-a[1])
    h=math.sin((p2-p1)/2)**2+math.cos(p1)*math.cos(p2)*math.sin(dl/2)**2
    return 2*R*math.asin(math.sqrt(h))
pos={}; adj={}; edge={}
for w in d['elements']:
    if w['type']!='way' or w.get('tags',{}).get('railway')!='rail': continue
    t=w['tags']; pen=1.0
    if t.get('service') in('yard','spur','siding','crossover'): pen=3.0
    if t.get('usage')=='industrial': pen=4.0
    ns=w['nodes']; g=w['geometry']
    for n,p in zip(ns,g): pos[n]=(p['lat'],p['lon'])
    for a,b in zip(ns,ns[1:]):
        L=hav(pos[a],pos[b])
        adj.setdefault(a,[]).append((b,L*pen,L)); adj.setdefault(b,[]).append((a,L*pen,L))
        edge[(a,b)]=edge[(b,a)]=w
def nearest(pt): return min(pos,key=lambda n:hav(pos[n],pt))
A=nearest((-25.5165409,-48.506529)); B=nearest((-25.4374349,-49.2562661))
print('snap km',hav(pos[A],(-25.5165409,-48.506529)),hav(pos[B],(-25.4374349,-49.2562661)))
dist={A:0}; prev={}; pq=[(0,A)]
while pq:
    c,u=heapq.heappop(pq)
    if u==B: break
    if c>dist.get(u,1e18): continue
    for v,wt,L in adj[u]:
        nc=c+wt
        if nc<dist.get(v,1e18): dist[v]=nc; prev[v]=u; heapq.heappush(pq,(nc,v))
if B not in prev: print('NO PATH'); sys.exit(1)
path=[B]
while path[-1]!=A: path.append(prev[path[-1]])
path.reverse()
km=[0.0]
for a,b in zip(path,path[1:]): km.append(km[-1]+hav(pos[a],pos[b]))
print('path nodes',len(path),'length km',round(km[-1],2))
def project(pt):
    best=(1e9,0)
    for i,n in enumerate(path):
        dd=hav(pos[n],pt)
        if dd<best[0]: best=(dd,i)
    return round(km[best[1]],2), round(best[0]*1000)
pts=[]
for x in d2['elements']+d['elements']:
    t=x.get('tags',{})
    if x['type']=='node' and t.get('railway') in('station','halt','site','yard','service_station','milestone'):
        k,off=project((x['lat'],x['lon']))
        pts.append((k,off,t.get('name'),t.get('railway'),t.get('railway:ref'),t.get('railway:position'),x['id'],x['lat'],x['lon'],t.get('operator'),t.get('start_date'),t.get('ele'),t.get('wikidata')))
seen=set()
pts=[p for p in sorted(pts) if not (p[6] in seen or seen.add(p[6]))]
for p in pts: print(p)
# structures along the path
feats=[];cur=None
for i,(a,b) in enumerate(zip(path,path[1:])):
    w=edge[(a,b)];t=w['tags']
    kind='tunnel' if t.get('tunnel') in('yes','building_passage') else 'bridge' if t.get('bridge') and t.get('bridge')!='no' else None
    key=(w['id'],kind)
    if kind:
        if cur and cur['key']==key: cur['to']=km[i+1]
        else:
            cur={'key':key,'kind':kind,'way':w['id'],'from':km[i],'to':km[i+1],'name':t.get('name') or t.get('bridge:name') or t.get('tunnel:name'),'btype':t.get('bridge')}
            feats.append(cur)
    else: cur=None
print('tunnels',sum(f['kind']=='tunnel' for f in feats),'bridges',sum(f['kind']=='bridge' for f in feats))
for f in feats: print(f['kind'],round(f['from'],2),round((f['to']-f['from'])*1000),'m',f['name'],f['btype'],f['way'])
names={}
for a,b in zip(path,path[1:]):
    t=edge[(a,b)]['tags']; names[(t.get('name'),t.get('usage'),t.get('gauge'),t.get('electrified'),t.get('operator'),t.get('maxspeed'))]=names.get((t.get('name'),t.get('usage'),t.get('gauge'),t.get('electrified'),t.get('operator'),t.get('maxspeed')),0)+hav(pos[a],pos[b])
for k,v in names.items(): print(round(v,1),k)
json.dump({'path':[[pos[n][0],pos[n][1],round(k,4)] for n,k in zip(path,km)],'points':pts,'features':[{k:v for k,v in f.items() if k!='key'} for f in feats]},open('derived.json','w',encoding='utf-8'),ensure_ascii=False)
