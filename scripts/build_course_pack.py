#!/usr/bin/env python3
"""Create a deterministic non-executable .dgxlabpack ZIP."""
from __future__ import annotations
import hashlib,json,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'dist'/'DGX_Lab_SLURM_Fundamentals_v1.0.0.dgxlabpack'
SOURCES=[ROOT/'course-src'/'slurm-fundamentals',ROOT/'scenario-src',ROOT/'question-src']

def files():
    paths=[]
    for base in SOURCES:
        paths.extend(p for p in base.rglob('*') if p.is_file())
    return sorted(paths,key=lambda p:str(p.relative_to(ROOT)))

def main():
    OUT.parent.mkdir(parents=True,exist_ok=True)
    manifest={'schema':'dgxlab.content-pack/v1','id':'slurm-fundamentals','revision':'1.0.0','executable_content':False,'files':[]}
    for path in files():
        data=path.read_bytes();manifest['files'].append({'path':str(path.relative_to(ROOT)),'size':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    manifest_bytes=(json.dumps(manifest,indent=2,sort_keys=True)+'\n').encode()
    epoch=(2026,1,1,0,0,0)
    with zipfile.ZipFile(OUT,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        info=zipfile.ZipInfo('manifest.json',epoch);info.external_attr=0o644<<16;z.writestr(info,manifest_bytes)
        for path in files():
            info=zipfile.ZipInfo(str(path.relative_to(ROOT)),epoch);info.external_attr=0o644<<16;z.writestr(info,path.read_bytes())
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")
if __name__=='__main__': main()
