"""CLI entry point that delegates to the bundled Rust binary."""

import os
import sys
import platform
import subprocess
from pathlib import Path


def get_binary_path() -> Path:
    """Get the path to the bundled hyper binary."""
    pkg_dir = Path(__file__).parent
    system = platform.system().lower()
    machine = platform.machine().lower()

    # Map to binary names
    if system == "darwin":
        if machine in ("arm64", "aarch64"):
            binary_name = "hyper-darwin-arm64"
        else:
            binary_name = "hyper-darwin-x64"
    elif system == "linux":
        if machine in ("arm64", "aarch64"):
            binary_name = "hyper-linux-arm64"
        else:
            binary_name = "hyper-linux-x64"
    elif system == "windows":
        binary_name = "hyper-windows-x64.exe"
    else:
        raise RuntimeError(f"Unsupported platform: {system} {machine}")

    binary_path = pkg_dir / "bin" / binary_name
    if not binary_path.exists():
        raise RuntimeError(
            f"Binary not found: {binary_path}\n"
            f"The hyper binary for {system}-{machine} is not bundled in this package.\n"
            f"Build it with: cd rust/transpiler && cargo build --release"
        )

    return binary_path


def main():
    """Run the bundled hyper CLI."""
    try:
        binary = get_binary_path()
    except RuntimeError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    # Make sure it's executable
    if not os.access(binary, os.X_OK):
        os.chmod(binary, 0o755)

    # Execute the binary with all arguments
    result = subprocess.run([str(binary)] + sys.argv[1:])
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
