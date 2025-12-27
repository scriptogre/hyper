"""
Entry point for running the Hyper LSP server.

Usage:
    python -m hyper.lsp
"""

import asyncio
from .server import main

if __name__ == '__main__':
    asyncio.run(main())
