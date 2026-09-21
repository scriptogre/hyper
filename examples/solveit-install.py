"""Install Hyper in a demo-only folder. Run once before loading the notebook extension."""

import hashlib
import io
import json
import subprocess
import sys
import urllib.request
import zipfile
from pathlib import Path

from packaging.tags import sys_tags
from packaging.utils import parse_wheel_filename


uv_version = '0.12.17'
demo = Path('.hyper-demo').resolve()
tools = demo / 'tools' / f'uv-{uv_version}'
packages = demo / 'packages'
uv = next((path for path in tools.rglob('uv') if path.is_file()), None)

if uv is None:
    with urllib.request.urlopen(f'https://pypi.org/pypi/uv/{uv_version}/json') as response:
        release = json.load(response)

    supported_tags = set(sys_tags())
    for candidate in release['urls']:
        if not candidate['filename'].endswith('.whl'):
            continue

        wheel_tags = parse_wheel_filename(candidate['filename'])[3]
        if wheel_tags & supported_tags:
            wheel = candidate
            break
    else:
        raise RuntimeError('No uv wheel supports this Python environment')

    with urllib.request.urlopen(wheel['url']) as response:
        wheel_bytes = response.read()

    expected_hash = wheel['digests']['sha256']
    if hashlib.sha256(wheel_bytes).hexdigest() != expected_hash:
        raise RuntimeError('The uv download failed its checksum check')

    with zipfile.ZipFile(io.BytesIO(wheel_bytes)) as archive:
        archive.extractall(tools)

    uv = next(path for path in tools.rglob('uv') if path.is_file())
    uv.chmod(0o755)

subprocess.run(
    [str(uv), 'pip', 'install', '--target', str(packages), 'hyperhtml==0.1.2'],
    check=True,
)

if str(packages) not in sys.path:
    sys.path.insert(0, str(packages))
