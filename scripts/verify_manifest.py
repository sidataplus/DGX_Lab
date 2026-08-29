#!/usr/bin/env python3
from __future__ import annotations
import hashlib,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def main():
    failures=[]
    for line in (ROOT/'MANIFEST.sha256').read_text().splitlines():
        digest,rel=line.split('  ',1);path=ROOT/rel
        if not path.is_file(): failures.append(f"missing {rel}");continue
        actual=hashlib.sha256(path.read_bytes()).hexdigest()
        if actual!=digest: failures.append(f"digest mismatch {rel}")
    if failures: print('\n'.join(failures),file=sys.stderr);return 1
    print('manifest verified');return 0
if __name__=='__main__': raise SystemExit(main())
