from datetime import datetime
import json
from pathlib import Path
from hyper import html, replace_markers


@html
def Imports(*, name: str):
    yield replace_markers(f"""\
<p>‹ESCAPE:{datetime.now().isoformat()}›</p>
<pre>‹ESCAPE:{json.dumps({"name": name}, indent=2)}›</pre>
<span>‹ESCAPE:{Path("/tmp").name}›</span>""")
