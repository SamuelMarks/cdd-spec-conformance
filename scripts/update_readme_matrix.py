#!/usr/bin/env python3
"""Updates the Ecosystem Conformance Matrix table in README.md."""

import os
import subprocess
import sys

TARGET_ECOSYSTEM_REPOS = (
    "cdd-c",
    "cdd-cpp",
    "cdd-csharp",
    "cdd-go",
    "cdd-java",
    "cdd-kotlin",
    "cdd-php",
    "cdd-python-all",
    "cdd-ruby",
    "cdd-rust",
    "cdd-sh",
    "cdd-swift",
    "cdd-ts",
)


def has_all_ecosystem_repos(parent_dir: str) -> bool:
    """Returns True if all 13 core CDD language repositories exist in `parent_dir`."""
    return all(
        os.path.isdir(os.path.join(parent_dir, repo)) for repo in TARGET_ECOSYSTEM_REPOS
    )


def main() -> None:
    """Runs `cdd-conformance matrix` to update README.md with ecosystem scores."""
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    parent_dir = os.path.dirname(root_dir)
    readme_path = os.path.join(root_dir, "README.md")

    bin_candidates = [
        os.path.join(root_dir, "target", "release", "cdd-conformance"),
        os.path.join(root_dir, "target", "debug", "cdd-conformance"),
    ]

    matrix_args = ["matrix", "--repos-dir", parent_dir]
    if has_all_ecosystem_repos(parent_dir):
        matrix_args.extend(["--readme", readme_path])

    cmd = None
    for candidate in bin_candidates:
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            cmd = [candidate, *matrix_args]
            break

    if cmd is None:
        cmd = [
            "cargo",
            "run",
            "--quiet",
            "--",
            *matrix_args,
        ]

    try:
        subprocess.run(cmd, cwd=root_dir, check=True)
    except subprocess.CalledProcessError as exc:
        print(f"Failed to update README.md matrix: {exc}", file=sys.stderr)
        sys.exit(exc.returncode)


if __name__ == "__main__":
    main()
