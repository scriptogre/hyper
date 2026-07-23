"""Check that source distribution metadata names files in the archive."""

from __future__ import annotations

import sys
import tarfile
from email.parser import BytesParser
from pathlib import Path


def main(path: Path) -> None:
    with tarfile.open(path) as archive:
        names = set(archive.getnames())
        metadata_name = next(name for name in names if name.endswith("/PKG-INFO"))
        root = metadata_name.removesuffix("/PKG-INFO")
        metadata = BytesParser().parsebytes(archive.extractfile(metadata_name).read())

    missing = [
        name
        for license_file in metadata.get_all("License-File", [])
        if (name := f"{root}/{license_file}") not in names
    ]
    if missing:
        raise SystemExit(f"Missing metadata files: {', '.join(missing)}")

    print(f"Source metadata files present: {path.name}")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
