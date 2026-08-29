#!/usr/bin/env python3
from __future__ import annotations
import hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
EXCLUDE={'MANIFEST.sha256'}

def main():
    lines=[]
    for path in sorted((p for p in ROOT.rglob('*') if p.is_file()),key=lambda p:str(p.relative_to(ROOT))):
        rel=str(path.relative_to(ROOT))
        if rel in EXCLUDE or rel.endswith('.pyc') or '/__pycache__/' in '/'+rel:
            continue
        digest=hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {rel}")
    (ROOT/'MANIFEST.sha256').write_text('\n'.join(lines)+'\n',encoding='utf-8')
    print(f"wrote {len(lines)} checksums")
if __name__=='__main__': main()
