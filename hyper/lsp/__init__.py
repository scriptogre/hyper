"""
Hyper LSP - Language Server Protocol proxy for .hyper files.

This module provides a language server that:
1. Transforms .hyper files into virtual Python files
2. Proxies requests to Pyright for Python intelligence
3. Maps positions between source .hyper files and generated Python

Architecture:
    Editor <-> Hyper LSP <-> Pyright
                    |
                    +---> Source Map (hyper positions <-> python positions)
"""
