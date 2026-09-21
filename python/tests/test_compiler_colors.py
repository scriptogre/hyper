import pytest

from hyperhtml import _native as compiler

INVALID_COMPONENT = '''
def Greeting(name: str) -> Component:
    <p>{name}</p>
'''


@pytest.mark.parametrize('transpile', [compiler.transpile, compiler.transpile_file])
def test_compiler_diagnostics_are_plain_by_default(transpile):
    with pytest.raises(ValueError) as error:
        transpile(INVALID_COMPONENT, 'Greeting.hyper')

    assert '\x1b[' not in str(error.value)
    assert 'Component props must be keyword-only.' in str(error.value)


@pytest.mark.parametrize('transpile', [compiler.transpile, compiler.transpile_file])
def test_compiler_can_render_cli_colors(transpile):
    with pytest.raises(ValueError) as error:
        transpile(INVALID_COMPONENT, 'Greeting.hyper', color=True)

    diagnostic = str(error.value)
    assert '\x1b[1;31merror:' in diagnostic
    assert '\x1b[1;38;5;73m' in diagnostic
    assert 'Add ' in diagnostic
    assert '`*,`' in diagnostic


@pytest.mark.parametrize('transpile', [compiler.transpile, compiler.transpile_file])
def test_color_does_not_change_generated_code(transpile):
    assert transpile('<p>Hello</p>', 'Greeting.hyper', color=True) == transpile(
        '<p>Hello</p>', 'Greeting.hyper'
    )
