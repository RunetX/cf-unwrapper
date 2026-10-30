#!/usr/bin/env python3
"""One acceptance command for a checkout, including Cargo's package verification."""
import os
from pathlib import Path
import subprocess
import sys
from audit import audit
from check_git_assets import check as check_git_assets

ROOT = Path(__file__).resolve().parents[1]

def run(*args):
    print("+ " + " ".join(args), flush=True)
    env = {k: v for k, v in os.environ.items() if not k.startswith("CFKIT_")}
    result = subprocess.run(args, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace")
    print("\n".join(result.stdout.splitlines()[-30:]), flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)

if __name__ == "__main__":
    os.chdir(ROOT)
    audit()
    check_git_assets()
    run("rustfmt", "--edition", "2021", "--check", "--config", "skip_children=true", "crates/cf-unwrapper/src/lib.rs", "crates/cf-unwrapper-cli/src/main.rs", "crates/cf-unwrapper/tests/roundtrip.rs", "crates/cf-unwrapper-cli/tests/cli.rs")
    run("cargo", "test", "--workspace", "--locked")
    run("cargo", "package", "--manifest-path", "crates/cf-unwrapper/Cargo.toml", "--allow-dirty", "--locked")
    print("check: PASS")
