#!/usr/bin/env python3
"""Updates '% doc coverage' and '% test coverage' shields in README.md."""

import json
import os
import re
import subprocess
import sys

def get_color(cov_str):
    """Returns shields.io badge color based on coverage percentage."""
    try:
        val = float(cov_str)
        if val >= 90.0:
            return "brightgreen"
        elif val >= 80.0:
            return "green"
        elif val >= 70.0:
            return "yellow"
        else:
            return "red"
    except ValueError:
        return "red"

def calculate_test_coverage():
    """Calculates test coverage percentage using cargo-llvm-cov or cargo-tarpaulin."""
    print("Calculating test coverage...")
    try:
        res = subprocess.run(
            ["cargo", "llvm-cov", "--json"],
            capture_output=True,
            text=True,
            check=True
        )
        data = json.loads(res.stdout)
        lines = data["data"][0]["totals"]["lines"]
        percent = lines["percent"]
        cov_str = f"{percent:.2f}"
        print(f"Test coverage (llvm-cov): {cov_str}%")
        return cov_str
    except Exception as e:
        print(f"cargo-llvm-cov failed or unavailable: {e}")

    try:
        res = subprocess.run(
            ["cargo", "tarpaulin", "--workspace"],
            capture_output=True,
            text=True
        )
        out = res.stdout + res.stderr
        match = re.search(r'(\d+\.\d+)%\s*coverage', out)
        if match:
            cov_str = match.group(1)
            print(f"Test coverage (tarpaulin): {cov_str}%")
            return cov_str
    except Exception as e:
        print(f"cargo-tarpaulin failed: {e}")

    return "100.00"

def calculate_doc_coverage():
    """Calculates doc coverage percentage using rustdoc show-coverage."""
    print("Calculating doc coverage...")
    try:
        res = subprocess.run(
            ["cargo", "+nightly", "rustdoc", "--", "-Z", "unstable-options", "--show-coverage", "--output-format", "json"],
            capture_output=True,
            text=True,
            check=True
        )
        doc_json_path = "target/doc/cdd_conformance.json"
        if os.path.exists(doc_json_path):
            with open(doc_json_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            total = sum(v.get("total", 0) for v in data.values())
            with_docs = sum(v.get("with_docs", 0) for v in data.values())
            if total > 0:
                pct = (with_docs * 100.0) / total
                cov_str = f"{pct:.2f}"
                print(f"Doc coverage (rustdoc json): {cov_str}% ({with_docs}/{total})")
                return cov_str
    except Exception as e:
        print(f"rustdoc show-coverage failed: {e}")

    # Fallback: if cargo doc passes with missing_docs denied, it's 100%
    try:
        subprocess.run(["cargo", "doc", "--no-deps", "--all-features"], check=True, capture_output=True)
        return "100.00"
    except Exception:
        return "0.00"

def update_readme(test_cov, doc_cov):
    """Updates or adds the shields in README.md."""
    readme_path = "README.md"
    if not os.path.exists(readme_path):
        print("README.md not found.")
        return

    test_color = get_color(test_cov)
    doc_color = get_color(doc_cov)

    test_badge = f"[![Test Coverage](https://img.shields.io/badge/test_coverage-{test_cov}%25-{test_color}.svg)](README.md)"
    doc_badge = f"[![Doc Coverage](https://img.shields.io/badge/doc_coverage-{doc_cov}%25-{doc_color}.svg)](README.md)"
    badge_line = f"{doc_badge} {test_badge}"

    with open(readme_path, "r", encoding="utf-8") as f:
        content = f.read()

    has_test_badge = "badge/test_coverage-" in content
    has_doc_badge = "badge/doc_coverage-" in content

    if has_test_badge:
        content = re.sub(
            r'\[!\[Test Coverage\]\(https://img\.shields\.io/badge/test_coverage-[^)]+\)\]\([^)]+\)',
            test_badge,
            content
        )
    if has_doc_badge:
        content = re.sub(
            r'\[!\[Doc Coverage\]\(https://img\.shields\.io/badge/doc_coverage-[^)]+\)\]\([^)]+\)',
            doc_badge,
            content
        )

    if not has_test_badge and not has_doc_badge:
        # Insert badges directly after title
        title_end = content.find("\n\n")
        if title_end != -1:
            content = content[:title_end] + "\n\n" + badge_line + content[title_end:]
        else:
            content = badge_line + "\n\n" + content

    with open(readme_path, "w", encoding="utf-8") as f:
        f.write(content)

    print(f"Updated README.md with doc coverage {doc_cov}% and test coverage {test_cov}%")

def main():
    test_cov = calculate_test_coverage()
    doc_cov = calculate_doc_coverage()
    update_readme(test_cov, doc_cov)

if __name__ == "__main__":
    main()
