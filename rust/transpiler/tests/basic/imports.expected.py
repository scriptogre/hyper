from datetime import datetime
import json
from pathlib import Path
from hyper import html, escape


@html
def Imports(*, name: str):
    yield f"""\
<p>{escape(datetime.now().isoformat())}</p>
<pre>{escape(json.dumps({"name": name}, indent=2))}</pre>
<span>{escape(Path("/tmp").name)}</span>"""
