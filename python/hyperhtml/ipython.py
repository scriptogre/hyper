"""Load with `%load_ext hyperhtml.ipython`; run `%%hyper Button`."""

from dataclasses import dataclass
from html import escape
from inspect import isawaitable, signature
from keyword import iskeyword
from uuid import uuid4

from hyperhtml import _native as compiler


class HyperCompileError(ValueError):
    def _render_traceback_(self):
        return [str(self).strip()]


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
        """Compile a named component and preview it using notebook variables."""
        name = line.strip()
        if not name.isidentifier() or iskeyword(name):
            raise ValueError('Provide a component name, for example: %%hyper Button')

        filename = f'{name}.hyper'
        try:
            python = compiler.transpile(cell, filename)
        except ValueError as error:
            raise HyperCompileError(str(error)) from None

        namespace = ipython.user_ns.copy()
        exec(compile(python, filename, 'exec'), namespace)
        component = namespace[name]

        props = {
            prop: ipython.user_ns[prop]
            for prop in signature(component).parameters
            if prop in ipython.user_ns
        }
        rendered = component(**props)
        if isawaitable(rendered):
            rendered.close()
            raise TypeError('The Hyper preview requires a synchronous component')

        ipython.user_ns[name] = component
        return HyperPreview(str(rendered), python)

    ipython.register_magic_function(hyper, magic_kind='cell', magic_name='hyper')


def unload_ipython_extension(ipython):
    ipython.magics_manager.magics['cell'].pop('hyper', None)
