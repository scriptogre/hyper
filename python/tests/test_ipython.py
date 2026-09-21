import json
from pathlib import Path

import pytest

pytest.importorskip('IPython')
from IPython.core.interactiveshell import InteractiveShell

from hyperhtml.ipython import load_ipython_extension


@pytest.fixture
def shell():
    shell = InteractiveShell()
    load_ipython_extension(shell)
    return shell


def test_cell_compiles_renders_and_exports_component(shell):
    shell.user_ns['name'] = '<Ada>'
    result = shell.run_cell_magic('hyper', 'Greeting',
        'name: str\n---\n<h1>Hello, {name}!</h1>')
    assert result.html == '<h1>Hello, &lt;Ada&gt;!</h1>'
    assert 'def Greeting(' in result.python
    assert shell.user_ns['Greeting'](name='Lin') == '<h1>Hello, Lin!</h1>'


def test_rerun_replaces_component(shell):
    shell.run_cell_magic('hyper', 'Greeting', '<p>First</p>')
    result = shell.run_cell_magic('hyper', 'Greeting', '<p>Second</p>')
    assert result.html == '<p>Second</p>'
    assert shell.user_ns['Greeting']() == '<p>Second</p>'


def test_preview_keeps_html_in_sandbox(shell):
    result = shell.run_cell_magic('hyper', 'Greeting', '<p>Hi</p>')
    output = result._repr_html_()
    assert 'sandbox=""' in output
    assert 'srcdoc="&lt;p&gt;Hi&lt;/p&gt;"' in output
    assert 'Preview' in output and 'Python' in output


@pytest.mark.parametrize('name', ['', '../oops', 'class', 'Greeting name=name'])
def test_invalid_name_is_rejected(shell, name):
    with pytest.raises(ValueError, match='component name'):
        shell.run_cell_magic('hyper', name, '<p>Hi</p>')


def test_demo_notebook_runs(shell):
    path = Path(__file__).parents[2] / 'examples' / 'solveit.ipynb'
    notebook = json.loads(path.read_text())
    for cell in notebook['cells']:
        result = shell.run_cell(''.join(cell['source']))
        result.raise_error()
    assert 'Hello, Answer.AI!' in shell.user_ns['Greeting'](names=['Answer.AI'])


def test_failed_rerun_preserves_last_working_component(shell):
    shell.run_cell_magic('hyper', 'Greeting', '<p>Works</p>')
    with pytest.raises(ValueError):
        shell.run_cell_magic('hyper', 'Greeting', '<p><div>Invalid</div></p>')
    assert shell.user_ns['Greeting']() == '<p>Works</p>'


def test_magic_uses_notebook_variables(shell):
    shell.user_ns['names'] = ['Ada', '<Lin>']
    result = shell.run_cell_magic('hyper', 'Greeting',
        'names: list[str]\n---\nfor name in names:\n    <p>{name}</p>\nend')
    assert result.html == '<p>Ada</p><p>&lt;Lin&gt;</p>'
    assert 'def Greeting(' in result.python


def test_magic_reads_updated_variables(shell):
    source = 'name: str\n---\n<p>{name}</p>'
    shell.user_ns['name'] = 'Ada'
    shell.run_cell_magic('hyper', 'Greeting', source)
    shell.user_ns['name'] = 'Lin'
    assert shell.run_cell_magic('hyper', 'Greeting', source).html == '<p>Lin</p>'


def test_magic_keeps_prop_defaults(shell):
    result = shell.run_cell_magic('hyper', 'Greeting', 'name: str = "Ada"\n---\n<p>{name}</p>')
    assert result.html == '<p>Ada</p>'


def test_named_components_can_be_composed(shell):
    shell.run_cell_magic('hyper', 'Button', '<button>Save</button>')
    result = shell.run_cell_magic('hyper', 'Panel', '<section>\n    <{Button} />\n</section>')

    assert result.html == '<section><button>Save</button></section>'
    assert shell.user_ns['Button']() == '<button>Save</button>'
