#!/usr/bin/env python3
"""Deterministic public-source boundary audit; no external files or corpus required."""
import hashlib
import json
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]
LIB = ROOT / "crates/cf-unwrapper"

def audit():
    inventory = json.loads((ROOT / "docs/source-inventory.json").read_text(encoding="utf-8"))
    expected = {entry["path"] for entry in inventory["files"]}
    actual = {p.relative_to(ROOT).as_posix() for base in [LIB / "src", LIB / "formats"] for p in base.rglob("*") if p.is_file()}
    assert actual == expected, f"source allowlist changed: {actual ^ expected}"
    for entry in inventory["files"]:
        p = ROOT / entry["path"]
        if p.suffix != ".rs":
            assert hashlib.sha256(p.read_bytes()).hexdigest() == entry["sha256"], f"review embedded asset change: {p.name}"
    for p in ROOT.glob("crates/**/*.rs"):
        text = p.read_text(encoding="utf-8-sig")
        assert not re.search(r"\b[A-Z]:[\\/]", text), f"machine path: {p}"
        assert not re.search(r"(?:crate|cf_unwrapper)::(?:bsl|platform|debugger|highlight)\b", text), f"excluded module: {p}"
        for ref in re.findall(r'include_(?:str|bytes)!\(\s*"([^"]+)"', text):
            asset = (p.parent / ref).resolve()
            assert asset.is_relative_to(LIB.resolve()) and asset.is_file(), f"external/missing include: {p}: {ref}"
        if "#[cfg(test)]" in text:
            tests = text.split("#[cfg(test)]", 1)[1]
            assert "std::env::var(" not in tests and "std::env::var_os(" not in tests, f"environment-gated tests: {p}"
    lib = tomllib.loads((LIB / "Cargo.toml").read_text(encoding="utf-8"))
    cli = tomllib.loads((ROOT / "crates/cf-unwrapper-cli/Cargo.toml").read_text(encoding="utf-8"))
    assert lib["dependencies"] == {"flate2": "1.1"}
    assert set(cli["dependencies"]) == {"cf-unwrapper"}
    assert cli["dependencies"]["cf-unwrapper"]["path"] == "../cf-unwrapper"
    forbidden = {"platform", "bsl", "debugger", "highlight"}
    assert not forbidden.intersection(p.name for p in (LIB / "src").iterdir())
    assert (ROOT / "LICENSE").read_bytes() == (LIB / "LICENSE").read_bytes()
    print(f"audit: {len(expected)} source/asset allowlist entries, includes and dependency boundary OK")

if __name__ == "__main__":
    audit()
