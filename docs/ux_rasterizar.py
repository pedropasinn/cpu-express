"""Render captured egui tessellation/textures for QA; not a native screenshot."""
from pathlib import Path
import json, sys
import numpy as np
from PIL import Image


def render(path):
    data=json.loads(path.read_text())
    width,height=data["size"]
    ppp=data["pixels_per_point"]
    canvas=np.zeros((height,width,4),dtype=np.float64)
    canvas[:,:,3]=1
    textures={item["id"]:(np.array(item["pixels"],dtype=np.float64).reshape(item["size"][1],item["size"][0],4)/255,item["linear"]) for item in data["textures"]}
    for mesh in data["meshes"]:
        vertices=np.array(mesh["vertices"],dtype=np.float64)
        if not len(vertices):continue
        vertices[:,:2]*=ppp
        clip=np.array(mesh["clip"],dtype=np.float64)*ppp
        tex,linear=textures[mesh["texture"]]
        th,tw,_=tex.shape
        for indices in np.array(mesh["indices"]).reshape(-1,3):
            tri=vertices[indices]
            x0=max(0,int(np.ceil(clip[0]-0.5)),int(np.floor(tri[:,0].min())))
            x1=min(width,int(np.ceil(clip[2]-0.5)),int(np.ceil(tri[:,0].max())))
            y0=max(0,int(np.ceil(clip[1]-0.5)),int(np.floor(tri[:,1].min())))
            y1=min(height,int(np.ceil(clip[3]-0.5)),int(np.ceil(tri[:,1].max())))
            if x0>=x1 or y0>=y1:continue
            a,b,c=tri[:,:2]
            area=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
            if abs(area)<1e-9:continue
            if area<0:
                tri=tri[[0,2,1]]
                a,b,c=tri[:,:2]
                area=-area
            yy,xx=np.mgrid[y0:y1,x0:x1]
            x=xx+0.5;y=yy+0.5
            def edge(u,v):return (v[0]-u[0])*(y-u[1])-(v[1]-u[1])*(x-u[0])
            edges=[edge(b,c),edge(c,a),edge(a,b)]
            mask=np.ones_like(x,dtype=bool)
            for e,(u,v) in zip(edges,[(b,c),(c,a),(a,b)]):
                d=v-u
                top_left=(d[1]<0) or (d[1]==0 and d[0]>0)
                mask &= (e>1e-8) | ((np.abs(e)<=1e-8) & top_left)
            if not mask.any():continue
            weights=np.stack(edges,axis=-1)/area
            uv=weights@tri[:,2:4]
            vc=(weights@tri[:,4:8])/255
            if linear:
                tx=uv[:,:,0]*tw-0.5;ty=uv[:,:,1]*th-0.5
                lx=np.floor(tx).astype(int);ly=np.floor(ty).astype(int)
                fx=(tx-lx)[:,:,None];fy=(ty-ly)[:,:,None]
                xlo=np.clip(lx,0,tw-1);xhi=np.clip(lx+1,0,tw-1)
                ylo=np.clip(ly,0,th-1);yhi=np.clip(ly+1,0,th-1)
                tc=(tex[ylo,xlo]*(1-fx)+tex[ylo,xhi]*fx)*(1-fy)+(tex[yhi,xlo]*(1-fx)+tex[yhi,xhi]*fx)*fy
            else:
                tx=np.clip((uv[:,:,0]*tw).astype(int),0,tw-1)
                ty=np.clip((uv[:,:,1]*th).astype(int),0,th-1)
                tc=tex[ty,tx]
            src=vc*tc
            dst=canvas[y0:y1,x0:x1]
            composite=src+dst*(1-src[:,:,3:4])
            dst[mask]=composite[mask]
    out=path.with_suffix(".png")
    Image.fromarray(np.rint(np.clip(canvas[:,:,:3],0,1)*255).astype(np.uint8)).save(out)
    print(out)

for arg in sys.argv[1:]:
    path=Path(arg)
    for f in sorted(path.glob("*.json")) if path.is_dir() else [path]:render(f)
