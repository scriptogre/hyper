"""Load with `%load_ext hyperhtml.ipython`; run `%%hyper Greeting name='Ada'`."""

from dataclasses import dataclass
from html import escape
from inspect import isawaitable
from keyword import iskeyword
from uuid import uuid4

from hyperhtml import _native


@dataclass
class HyperPreview:
    html: str
    python: str

    def _repr_html_(self):
        ident = f'hyper-{uuid4().hex}'
        return f'''<div id="{ident}">
<style>
#{ident} .panel {{display:none}}
#{ident} input:checked + label + .panel {{display:block}}
#{ident} {{display:grid;grid-template-columns:auto auto 1fr;gap:8px}}
#{ident} input {{position:absolute;opacity:0;width:0}}
#{ident} label {{grid-row:1;cursor:pointer;padding:6px 12px;border:1px solid #888;border-radius:4px}}
#{ident} input:checked + label {{background:#ddd;color:#111}}
#{ident} input:focus-visible + label {{outline:2px solid #268bd2}}
#{ident} .panel {{grid-row:2;grid-column:1 / -1}}
#{ident} pre {{overflow:auto;max-height:600px;white-space:pre}}
</style>
<input type="radio" name="{ident}" id="{ident}-preview" checked>
<label for="{ident}-preview">Preview</label>
<div class="panel"><iframe title="Hyper preview" sandbox="" style="width:100%;height:360px;border:0;background:white" srcdoc="{escape(self.html, quote=True)}"></iframe></div>
<input type="radio" name="{ident}" id="{ident}-python">
<label for="{ident}-python">Python</label>
<div class="panel"><pre><code>{escape(self.python)}</code></pre></div>
</div>'''


def load_ipython_extension(ipython):
    def hyper(line, cell):
        """Compile and render trusted Hyper source: %%hyper Name prop=value, ..."""
        parts = line.strip().split(maxsplit=1)
        name = parts[0] if parts else 'Preview'
        if not name.isidentifier() or iskeyword(name):
            raise ValueError('Use a Python identifier for the component name')
        namespace = dict(ipython.user_ns)
        generated = _native.transpile(cell, f'{name}.hyper')
        exec(compile(generated, f'{name}.hyper', 'exec'), namespace)
        props = eval(f'dict({parts[1]})', ipython.user_ns) if len(parts) > 1 else {}
        component = namespace[name]
        rendered = component(**props)
        if isawaitable(rendered):
            rendered.close()
            raise TypeError('The Hyper preview requires a synchronous component')
        ipython.user_ns[name] = component
        return HyperPreview(str(rendered), generated)

    ipython.register_magic_function(hyper, magic_kind='cell', magic_name='hyper')


def unload_ipython_extension(ipython):
    ipython.magics_manager.magics['cell'].pop('hyper', None)
