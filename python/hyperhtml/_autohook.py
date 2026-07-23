"""Install the `.hyper` import hook."""

from __future__ import annotations

import sys

from hyperhtml._loader import HyperFinder


def _install_finder() -> HyperFinder:
    """Install and return the process-wide Hyper finder."""
    for finder in sys.meta_path:
        if isinstance(finder, HyperFinder):
            return finder

    finder = HyperFinder()
    sys.meta_path.insert(0, finder)
    return finder


def _uninstall_finder() -> None:
    """Remove all Hyper finders from `sys.meta_path`."""
    sys.meta_path[:] = [finder for finder in sys.meta_path if not isinstance(finder, HyperFinder)]


_install_finder()
