#!/usr/bin/env python3
"""Enforce the workspace's version-alignment policy.

The CLI tracks the library: every library release implies a CLI release of
equal or greater bump, and the CLI's `moneymoney` dependency pin must match
the library's current major.minor.

Concrete checks performed against the workspace's `Cargo.toml` files:

  1. `moneymoney-cli`'s `[dependencies] moneymoney = { version = "..." }`
     must reference the same major.minor as the library's own version.
  2. `moneymoney-cli`'s package version must be >= the library's version
     (SemVer-aware; pre-release suffixes are ignored for the comparison).

Exits 0 when aligned, 1 otherwise. Intended to run in CI and locally before
publishing.
"""

import re
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
LIB_MANIFEST = REPO_ROOT / "Cargo.toml"
CLI_MANIFEST = REPO_ROOT / "moneymoney-cli" / "Cargo.toml"


def parse_version(s: str) -> tuple[int, int, int]:
    m = re.match(r"(\d+)\.(\d+)\.(\d+)", s)
    if not m:
        raise ValueError(f"unparseable version: {s!r}")
    return (int(m.group(1)), int(m.group(2)), int(m.group(3)))


def parse_req(req: str) -> tuple[int, int]:
    """Extract major.minor from a Cargo version requirement string.

    Accepts the forms `0.3`, `0.3.1`, `^0.3`, `=0.3.1`, `~0.3` etc. Returns
    only major.minor since that's what the alignment policy cares about.
    """
    m = re.match(r"[\^~=]?(\d+)\.(\d+)", req)
    if not m:
        raise ValueError(f"unparseable dependency req: {req!r}")
    return (int(m.group(1)), int(m.group(2)))


def main() -> int:
    lib = tomllib.loads(LIB_MANIFEST.read_text())
    cli = tomllib.loads(CLI_MANIFEST.read_text())

    lib_version = lib["package"]["version"]
    cli_version = cli["package"]["version"]

    dep = cli["dependencies"]["moneymoney"]
    cli_lib_req = dep["version"] if isinstance(dep, dict) else dep

    print(f"library version:           {lib_version}")
    print(f"cli version:               {cli_version}")
    print(f"cli's moneymoney dep req:  {cli_lib_req}")

    errors: list[str] = []

    lib_mm = parse_version(lib_version)[:2]
    req_mm = parse_req(cli_lib_req)
    if req_mm != lib_mm:
        errors.append(
            f"moneymoney-cli's `moneymoney` dependency requirement "
            f"({cli_lib_req}) does not match the library's major.minor "
            f"({lib_version}). Update `moneymoney-cli/Cargo.toml`."
        )

    if parse_version(cli_version) < parse_version(lib_version):
        errors.append(
            f"moneymoney-cli version ({cli_version}) is older than the "
            f"library version ({lib_version}). Bump moneymoney-cli to at "
            f"least {lib_version}."
        )

    if errors:
        print("", file=sys.stderr)
        for e in errors:
            print(f"ERROR: {e}", file=sys.stderr)
        return 1

    print("OK: version alignment policy satisfied")
    return 0


if __name__ == "__main__":
    sys.exit(main())
