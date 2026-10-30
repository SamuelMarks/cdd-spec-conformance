#!/usr/bin/env python3
"""Updates '% doc coverage' and '% test coverage' shields in README.md."""

import json
import os
import re
import subprocess
import sys
from typing import Tuple


def get_color(cov_str: str) -> str:
    """Returns shields.io badge color for a given coverage percentage string.

    Args:
        cov_str: Percentage string (e.g. "100" or "95.5").

    Returns:
        Shields.io color identifier string.
    """
    try:
        cov = float(cov_str)
    except ValueError:
        return "red"
    if cov >= 100.0:
        return "brightgreen"
    if cov >= 90.0:
        return "green"
    if cov >= 80.0:
        return "yellowgreen"
    if cov >= 70.0:
        return "yellow"
    return "red"


def calculate_test_coverage() -> str:
    """Calculates test coverage (minimum of lines, functions, and regions) via cargo-llvm-cov or cargo-tarpaulin.

    Returns:
        Formatted percentage string (e.g. "100.00" or "98.50").

    Raises:
        SystemExit: If neither coverage tool succeeds.
    """
    try:
        res = subprocess.run(
            ["cargo", "llvm-cov", "--all-targets", "--json", "--summary-only"],
            capture_output=True,
            text=True,
            check=True,
        )
        data = json.loads(res.stdout)
        totals = data["data"][0]["totals"]
        line_pct = float(totals["lines"]["percent"])
        fn_pct = float(totals["functions"]["percent"])
        reg_pct = float(totals["regions"]["percent"])
        metrics = [line_pct, fn_pct, reg_pct]
        branches = totals.get("branches", {})
        if branches.get("count", 0) > 0:
            metrics.append(float(branches["percent"]))
        pct = min(metrics)
        if pct >= 99.995:
            return "100.00"
        return f"{pct:.2f}"
    except (
        subprocess.SubprocessError,
        OSError,
        json.JSONDecodeError,
        KeyError,
        IndexError,
        ValueError,
    ) as exc:
        if os.environ.get("VERBOSE"):
            print(f"cargo llvm-cov fallback triggered: {exc}", file=sys.stderr)

    try:
        res = subprocess.run(
            ["cargo", "tarpaulin", "--out", "Stdout"],
            capture_output=True,
            text=True,
            check=True,
        )
        match = re.search(r"(\d+\.\d+)%\s+coverage", res.stdout)
        if match:
            val = float(match.group(1))
            if val >= 99.995:
                return "100.00"
            return f"{val:.2f}"
    except (subprocess.SubprocessError, OSError, ValueError) as exc:
        print(f"Failed to calculate test coverage: {exc}", file=sys.stderr)
        sys.exit(1)

    print("Failed to parse test coverage output", file=sys.stderr)
    sys.exit(1)


def calculate_doc_coverage() -> str:
    """Calculates documentation coverage using nightly rustdoc or fallback parser.

    Returns:
        Formatted percentage string (e.g. "100.00" or "99.10").
    """
    try:
        res = subprocess.run(
            [
                "cargo",
                "+nightly",
                "rustdoc",
                "--",
                "-Z",
                "unstable-options",
                "--show-coverage",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        out = res.stdout + chr(10) + res.stderr
        target_dir = os.environ.get("CARGO_TARGET_DIR", "target")
        txt_path = os.path.join(target_dir, "doc", "cdd_conformance.txt")
        if os.path.exists(txt_path):
            with open(txt_path, "r", encoding="utf-8") as tf:
                out += chr(10) + tf.read()
        matches = re.findall(r"\|\s*Total\s*\|\s*\d+\s*\|\s*(\d+\.\d+)%", out)
        if matches:
            val = float(matches[-1])
            if val >= 99.995:
                return "100.00"
            return f"{val:.2f}"
    except (subprocess.SubprocessError, OSError, ValueError) as exc:
        if os.environ.get("VERBOSE"):
            print(f"nightly rustdoc fallback triggered: {exc}", file=sys.stderr)

    total_items, documented_items = count_documented_pub_items("src")
    if total_items == 0:
        return "100.00"
    pct = (documented_items / total_items) * 100.0
    if pct >= 99.995:
        return "100.00"
    return f"{pct:.2f}"


def count_documented_pub_items(src_dir: str) -> Tuple[int, int]:
    """Counts module, public, and private Rust items and how many have `//!` or `///` doc comments.

    Args:
        src_dir: Root source directory to walk.

    Returns:
        Tuple of `(total_items, documented_items)`.
    """
    total_items = 0
    documented_items = 0
    item_pattern = re.compile(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+|const\s+)?(?:fn|struct|enum|mod|trait|type|const)\s+"
    )
    for root, _, files in os.walk(src_dir):
        for file in files:
            if not file.endswith(".rs"):
                continue
            path = os.path.join(root, file)
            with open(path, "r", encoding="utf-8") as f:
                lines = f.readlines()

            total_items += 1
            if any(line.strip().startswith("//!") for line in lines):
                documented_items += 1

            has_doc = False
            for line in lines:
                stripped = line.strip()
                if stripped == "#[cfg(test)]":
                    break
                if stripped.startswith("///"):
                    has_doc = True
                elif stripped.startswith("#["):
                    continue
                elif item_pattern.match(line):
                    total_items += 1
                    if has_doc:
                        documented_items += 1
                    has_doc = False
                elif stripped == "":
                    has_doc = False
                else:
                    has_doc = False
    return total_items, documented_items


def update_readme(test_cov: str, doc_cov: str) -> None:
    """Updates the coverage badges in README.md.

    Args:
        test_cov: Calculated test coverage percentage string.
        doc_cov: Calculated documentation coverage percentage string.
    """
    readme_path = "README.md"
    if not os.path.exists(readme_path):
        return

    with open(readme_path, "r", encoding="utf-8") as f:
        content = f.read()

    test_color = get_color(test_cov)
    doc_color = get_color(doc_cov)

    test_badge = (
        f"[![Test Coverage](https://img.shields.io/badge/test_coverage-"
        f"{test_cov}%25-{test_color}.svg)](README.md)"
    )
    doc_badge = (
        f"[![Doc Coverage](https://img.shields.io/badge/doc_coverage-"
        f"{doc_cov}%25-{doc_color}.svg)](README.md)"
    )

    test_regex = re.compile(
        r"\[!\[test coverage\]\(https://img\.shields\.io/badge/test_coverage-[^\)]+\)\]\([^\)]+\)",
        re.IGNORECASE,
    )
    doc_regex = re.compile(
        r"\[!\[doc coverage\]\(https://img\.shields\.io/badge/doc_coverage-[^\)]+\)\]\([^\)]+\)",
        re.IGNORECASE,
    )

    if test_regex.search(content):
        content = test_regex.sub(test_badge, content)
    if doc_regex.search(content):
        content = doc_regex.sub(doc_badge, content)

    with open(readme_path, "w", encoding="utf-8") as f:
        f.write(content)

    print(
        f"Updated README.md with doc coverage {doc_cov}% and test coverage {test_cov}%"
    )


def main() -> None:
    """Entrypoint for updating test and doc coverage shields in README.md."""
    test_cov = calculate_test_coverage()
    doc_cov = calculate_doc_coverage()
    update_readme(test_cov, doc_cov)


if __name__ == "__main__":
    main()
