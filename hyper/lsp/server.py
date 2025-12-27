"""
Hyper LSP Server - Language Server Protocol implementation for .hyper files.

This server:
1. Listens for LSP requests from editors (VSCode, JetBrains, etc.)
2. Transforms .hyper files to Python
3. Proxies requests to Pyright
4. Maps responses back to .hyper positions

Usage:
    python -m hyper.lsp.server

Based on templ's LSP architecture:
    https://github.com/a-h/templ/tree/main/cmd/templ/lspcmd
"""

import asyncio
import json
import logging
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, Optional
from urllib.parse import urlparse, unquote

from .sourcemap import SourceMap, Position
from .transformer import transform, TransformResult

# Set up logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[logging.FileHandler('/tmp/hyper-lsp.log'), logging.StreamHandler()]
)
logger = logging.getLogger('hyper-lsp')


class HyperLanguageServer:
    """
    Language Server for .hyper files.

    Proxies to Pyright for Python intelligence.
    """

    def __init__(self):
        self.documents: dict[str, str] = {}  # URI -> content
        self.source_maps: dict[str, SourceMap] = {}  # URI -> source map
        self.virtual_documents: dict[str, str] = {}  # hyper URI -> generated Python
        self.pyright_process: Optional[subprocess.Popen] = None
        self.request_id = 0

    async def start(self):
        """Start the LSP server."""
        logger.info("Starting Hyper LSP Server")

        # Start Pyright in the background
        await self._start_pyright()

        # Read from stdin, write to stdout (standard LSP transport)
        await self._run_server()

    async def _start_pyright(self):
        """Start Pyright language server as a subprocess."""
        try:
            self.pyright_process = subprocess.Popen(
                ['pyright-langserver', '--stdio'],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            logger.info("Pyright started successfully")
        except FileNotFoundError:
            logger.warning("Pyright not found. Install with: pip install pyright")
            self.pyright_process = None

    async def _run_server(self):
        """Main server loop - read LSP messages from stdin."""
        reader = asyncio.StreamReader()
        protocol = asyncio.StreamReaderProtocol(reader)
        await asyncio.get_event_loop().connect_read_pipe(lambda: protocol, sys.stdin)

        writer_transport, writer_protocol = await asyncio.get_event_loop().connect_write_pipe(
            asyncio.streams.FlowControlMixin, sys.stdout
        )
        writer = asyncio.StreamWriter(writer_transport, writer_protocol, reader, asyncio.get_event_loop())

        while True:
            try:
                message = await self._read_message(reader)
                if message is None:
                    break

                response = await self._handle_message(message)
                if response is not None:
                    await self._write_message(writer, response)

            except Exception as e:
                logger.exception(f"Error handling message: {e}")

    async def _read_message(self, reader: asyncio.StreamReader) -> Optional[dict]:
        """Read a JSON-RPC message from the LSP stream."""
        # Read headers
        headers = {}
        while True:
            line = await reader.readline()
            if not line:
                return None
            line = line.decode('utf-8').strip()
            if not line:
                break
            if ':' in line:
                key, value = line.split(':', 1)
                headers[key.strip().lower()] = value.strip()

        # Read content
        content_length = int(headers.get('content-length', 0))
        if content_length == 0:
            return None

        content = await reader.readexactly(content_length)
        return json.loads(content.decode('utf-8'))

    async def _write_message(self, writer: asyncio.StreamWriter, message: dict):
        """Write a JSON-RPC message to the LSP stream."""
        content = json.dumps(message)
        header = f"Content-Length: {len(content)}\r\n\r\n"
        writer.write(header.encode('utf-8'))
        writer.write(content.encode('utf-8'))
        await writer.drain()

    async def _handle_message(self, message: dict) -> Optional[dict]:
        """Handle an incoming LSP message."""
        method = message.get('method', '')
        msg_id = message.get('id')
        params = message.get('params', {})

        logger.debug(f"Received: {method}")

        # Handle initialization
        if method == 'initialize':
            return self._handle_initialize(msg_id, params)

        elif method == 'initialized':
            return None  # Notification, no response

        elif method == 'shutdown':
            return {'jsonrpc': '2.0', 'id': msg_id, 'result': None}

        elif method == 'exit':
            sys.exit(0)

        # Document synchronization
        elif method == 'textDocument/didOpen':
            await self._handle_did_open(params)
            return None

        elif method == 'textDocument/didChange':
            await self._handle_did_change(params)
            return None

        elif method == 'textDocument/didClose':
            await self._handle_did_close(params)
            return None

        # Language features - proxy to Pyright
        elif method == 'textDocument/completion':
            return await self._proxy_to_pyright(msg_id, method, params)

        elif method == 'textDocument/hover':
            return await self._proxy_to_pyright(msg_id, method, params)

        elif method == 'textDocument/definition':
            return await self._proxy_to_pyright(msg_id, method, params)

        elif method == 'textDocument/references':
            return await self._proxy_to_pyright(msg_id, method, params)

        elif method == 'textDocument/documentSymbol':
            return await self._proxy_to_pyright(msg_id, method, params)

        else:
            logger.debug(f"Unhandled method: {method}")
            if msg_id is not None:
                return {
                    'jsonrpc': '2.0',
                    'id': msg_id,
                    'error': {'code': -32601, 'message': f'Method not found: {method}'}
                }
            return None

    def _handle_initialize(self, msg_id: int, params: dict) -> dict:
        """Handle initialize request."""
        return {
            'jsonrpc': '2.0',
            'id': msg_id,
            'result': {
                'capabilities': {
                    'textDocumentSync': {
                        'openClose': True,
                        'change': 1,  # Full sync
                    },
                    'completionProvider': {
                        'triggerCharacters': ['.', '{'],
                    },
                    'hoverProvider': True,
                    'definitionProvider': True,
                    'referencesProvider': True,
                    'documentSymbolProvider': True,
                },
                'serverInfo': {
                    'name': 'hyper-lsp',
                    'version': '0.1.0',
                }
            }
        }

    async def _handle_did_open(self, params: dict):
        """Handle textDocument/didOpen - transform and cache the document."""
        uri = params['textDocument']['uri']
        content = params['textDocument']['text']

        self.documents[uri] = content

        # Transform to Python
        result = transform(content, uri)
        self.virtual_documents[uri] = result.python_code
        self.source_maps[uri] = result.source_map

        logger.debug(f"Opened {uri}, generated Python:\n{result.python_code[:500]}...")

        # Forward to Pyright with virtual document
        if self.pyright_process:
            await self._send_to_pyright({
                'jsonrpc': '2.0',
                'method': 'textDocument/didOpen',
                'params': {
                    'textDocument': {
                        'uri': result.source_map.python_uri,
                        'languageId': 'python',
                        'version': 1,
                        'text': result.python_code,
                    }
                }
            })

    async def _handle_did_change(self, params: dict):
        """Handle textDocument/didChange - re-transform the document."""
        uri = params['textDocument']['uri']
        # Full sync - take the whole new content
        for change in params.get('contentChanges', []):
            if 'text' in change:
                self.documents[uri] = change['text']

        # Re-transform
        if uri in self.documents:
            result = transform(self.documents[uri], uri)
            self.virtual_documents[uri] = result.python_code
            self.source_maps[uri] = result.source_map

            # Forward to Pyright
            if self.pyright_process:
                await self._send_to_pyright({
                    'jsonrpc': '2.0',
                    'method': 'textDocument/didChange',
                    'params': {
                        'textDocument': {
                            'uri': result.source_map.python_uri,
                            'version': params['textDocument'].get('version', 1),
                        },
                        'contentChanges': [{'text': result.python_code}]
                    }
                })

    async def _handle_did_close(self, params: dict):
        """Handle textDocument/didClose."""
        uri = params['textDocument']['uri']
        self.documents.pop(uri, None)
        self.virtual_documents.pop(uri, None)
        self.source_maps.pop(uri, None)

    async def _proxy_to_pyright(self, msg_id: int, method: str, params: dict) -> dict:
        """
        Proxy a request to Pyright with position mapping.

        1. Transform positions in the request from .hyper to .py
        2. Send to Pyright
        3. Transform positions in the response from .py back to .hyper
        """
        uri = params.get('textDocument', {}).get('uri', '')

        if uri not in self.source_maps:
            return {
                'jsonrpc': '2.0',
                'id': msg_id,
                'result': None
            }

        source_map = self.source_maps[uri]

        # Transform request positions
        transformed_params = self._transform_request_positions(params, source_map)

        # Send to Pyright
        response = await self._send_request_to_pyright(method, transformed_params)

        # Transform response positions back
        if response and 'result' in response:
            response['result'] = self._transform_response_positions(response['result'], source_map)

        return {
            'jsonrpc': '2.0',
            'id': msg_id,
            'result': response.get('result') if response else None
        }

    def _transform_request_positions(self, params: dict, source_map: SourceMap) -> dict:
        """Transform positions in request from .hyper to .py coordinates."""
        params = dict(params)

        # Transform textDocument URI
        if 'textDocument' in params:
            params['textDocument'] = dict(params['textDocument'])
            params['textDocument']['uri'] = source_map.python_uri

        # Transform position
        if 'position' in params:
            hyper_pos = Position(
                line=params['position']['line'],
                character=params['position']['character']
            )
            python_pos = source_map.hyper_to_python(hyper_pos)
            if python_pos:
                params['position'] = {
                    'line': python_pos.line,
                    'character': python_pos.character
                }

        return params

    def _transform_response_positions(self, result: Any, source_map: SourceMap) -> Any:
        """Transform positions in response from .py to .hyper coordinates."""
        if result is None:
            return None

        if isinstance(result, list):
            return [self._transform_response_positions(item, source_map) for item in result]

        if isinstance(result, dict):
            result = dict(result)

            # Transform URI
            if 'uri' in result and result['uri'] == source_map.python_uri:
                result['uri'] = source_map.hyper_uri

            # Transform range
            if 'range' in result:
                result['range'] = self._transform_range(result['range'], source_map)

            # Transform location
            if 'location' in result:
                result['location'] = self._transform_response_positions(result['location'], source_map)

            # Recurse into nested structures
            for key, value in list(result.items()):
                if isinstance(value, (dict, list)):
                    result[key] = self._transform_response_positions(value, source_map)

            return result

        return result

    def _transform_range(self, range_dict: dict, source_map: SourceMap) -> dict:
        """Transform a range from Python to Hyper coordinates."""
        start_pos = Position(
            line=range_dict['start']['line'],
            character=range_dict['start']['character']
        )
        end_pos = Position(
            line=range_dict['end']['line'],
            character=range_dict['end']['character']
        )

        hyper_start = source_map.python_to_hyper(start_pos)
        hyper_end = source_map.python_to_hyper(end_pos)

        if hyper_start and hyper_end:
            return {
                'start': {'line': hyper_start.line, 'character': hyper_start.character},
                'end': {'line': hyper_end.line, 'character': hyper_end.character}
            }

        return range_dict

    async def _send_to_pyright(self, message: dict):
        """Send a notification to Pyright."""
        if not self.pyright_process:
            return

        content = json.dumps(message)
        header = f"Content-Length: {len(content)}\r\n\r\n"
        self.pyright_process.stdin.write(header.encode('utf-8'))
        self.pyright_process.stdin.write(content.encode('utf-8'))
        self.pyright_process.stdin.flush()

    async def _send_request_to_pyright(self, method: str, params: dict) -> Optional[dict]:
        """Send a request to Pyright and wait for response."""
        if not self.pyright_process:
            return None

        self.request_id += 1
        request = {
            'jsonrpc': '2.0',
            'id': self.request_id,
            'method': method,
            'params': params
        }

        content = json.dumps(request)
        header = f"Content-Length: {len(content)}\r\n\r\n"
        self.pyright_process.stdin.write(header.encode('utf-8'))
        self.pyright_process.stdin.write(content.encode('utf-8'))
        self.pyright_process.stdin.flush()

        # Read response (simplified - real impl would be more robust)
        # For now, return None - this needs proper async handling
        return None


async def main():
    """Main entry point."""
    server = HyperLanguageServer()
    await server.start()


if __name__ == '__main__':
    asyncio.run(main())
