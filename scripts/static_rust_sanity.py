#!/usr/bin/env python3
"""Cheap source sanity checks when rustc is unavailable; not a compiler substitute."""
from __future__ import annotations
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def strip_strings_and_comments(text: str) -> str:
    out=[];i=0;mode="code";depth=0
    while i<len(text):
        if mode=="code":
            if text.startswith("//",i): mode="line";out.extend("  ");i+=2;continue
            if text.startswith("/*",i): mode="block";depth=1;out.extend("  ");i+=2;continue
            if text[i]=='"': mode="string";out.append(' ');i+=1;continue
            if text[i]=="'" and i+2<len(text):
                # Char/lifetime ambiguity: blank ordinary escaped/one-char literals only.
                j=i+1
                if text[j]=='\\': j+=2
                else: j+=1
                if j<len(text) and text[j]=="'": out.extend(' '*(j-i+1));i=j+1;continue
            out.append(text[i]);i+=1
        elif mode=="line":
            if text[i]=='\n': mode="code";out.append('\n')
            else: out.append(' ')
            i+=1
        elif mode=="block":
            if text.startswith("/*",i): depth+=1;out.extend("  ");i+=2
            elif text.startswith("*/",i): depth-=1;out.extend("  ");i+=2;mode="code" if depth==0 else mode
            else: out.append('\n' if text[i]=='\n' else ' ');i+=1
        else:
            if text[i]=='\\': out.extend("  ");i+=2
            elif text[i]=='"': out.append(' ');i+=1;mode="code"
            else: out.append('\n' if text[i]=='\n' else ' ');i+=1
    return ''.join(out)

def check(path: Path):
    text=strip_strings_and_comments(path.read_text(encoding="utf-8"))
    stack=[];pairs={')':'(',']':'[','}':'{'}
    for idx,ch in enumerate(text):
        if ch in '([{': stack.append((ch,idx))
        elif ch in ')]}':
            if not stack or stack[-1][0]!=pairs[ch]: return f"unmatched {ch} near offset {idx}"
            stack.pop()
    if stack: return f"unclosed delimiter {stack[-1][0]} near offset {stack[-1][1]}"
    # Inner crate attributes belong at crate roots and govern their child modules.
    if path.name == "lib.rs" and "#![forbid(unsafe_code)]" not in path.read_text(encoding="utf-8"):
        return "missing #![forbid(unsafe_code)]"
    return None

def main():
    failures=[]
    for path in sorted([*ROOT.glob('crates/*/src/*.rs'),*ROOT.glob('src-tauri/src/*.rs')]):
        err=check(path)
        if err: failures.append(f"{path.relative_to(ROOT)}: {err}")
    if failures:
        print("STATIC RUST SANITY FAILED\n"+'\n'.join(failures),file=sys.stderr);return 1
    print("static Rust delimiter/unsafe-code sanity passed (not compilation)");return 0
if __name__=='__main__': raise SystemExit(main())
