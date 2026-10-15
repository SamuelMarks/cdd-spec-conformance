#!/usr/bin/env python3
"""Updates the Ecosystem Conformance Matrix table in README.md."""

import os
import subprocess
import sys


def main() -> None:
    """Runs `cdd-conformance matrix` to update README.md with ecosystem scores."""
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    parent_dir = os.path.dirname(root_dir)
    readme_path = os.path.join(root_dir, "README.md")

    bin_candidates = [
        os.path.join(root_dir, "target", "release", "cdd-conformance"),
        os.path.join(root_dir, "target", "debug", "cdd-conformance"),
    ]

    cmd = None
    for candidate in bin_candidates:
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            cmd = [
                candidate,
                "matrix",
                "--repos-dir",
                parent_dir,
                "--readme",
                readme_path,
            ]
            break

    if cmd is None:
        cmd = [
            "cargo",
            "run",
            "--quiet",
            "--",
            "matrix",
            "--repos-dir",
            parent_dir,
            "--readme",
            readme_path,
        ]

    try:
        subprocess.run(cmd, cwd=root_dir, check=True)
    except subprocess.CalledProcessError as exc:
        print(f"Failed to update README.md matrix: {exc}", file=sys.stderr)
        sys.exit(exc.returncode)


if __name__ == "__main__":
    main()
