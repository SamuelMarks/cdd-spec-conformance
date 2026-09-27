#!/usr/bin/env python3
"""Updates the Ecosystem Conformance Matrix table in README.md."""

import os
import subprocess
import sys

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    readme_path = os.path.join(root_dir, "README.md")
    repos_dir = os.path.dirname(root_dir)

    print("Updating Ecosystem Conformance Matrix in README.md...")
    cmd = [
        "cargo",
        "run",
        "--",
        "matrix",
        "--repos-dir",
        repos_dir,
        "--readme",
        readme_path,
    ]
    try:
        subprocess.run(cmd, cwd=root_dir, check=True)
    except subprocess.CalledProcessError as e:
        print(f"Failed to update README.md matrix: {e}", file=sys.stderr)
        sys.exit(e.returncode)

if __name__ == "__main__":
    main()
