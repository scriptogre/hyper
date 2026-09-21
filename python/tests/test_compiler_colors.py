import pytest

from hyperhtml import _native as compiler


@pytest.mark.parametrize('transpile', [compiler.transpile, compiler.transpile_file])
def test_compiler_diagnostics_are_plain_by_default(transpile):
    with pytest.raises(ValueError) as error:
        transpile('if True:\n    <p>Hello</p>', 'Greeting.hyper')

    assert '\x1b[' not in str(error.value)
    assert "Close with 'end'" in str(error.value)


@pytest.mark.parametrize('transpile', [compiler.transpile, compiler.transpile_file])
def test_compiler_can_render_cli_colors(transpile):
    with pytest.raises(ValueError) as error:
        transpile('if True:\n    <p>Hello</p>', 'Greeting.hyper', color=True)

    diagnostic = str(error.value)
    assert '\x1b[1;31merror:' in diagnostic
    assert '\x1b[1;38;5;73m' in diagnostic
    assert 'Close with ' in diagnostic
    assert '`end`' in diagnostic


@pytest.mark.parametrize('transpile', [compiler.transpile, compiler.transpile_file])
def test_color_does_not_change_generated_code(transpile):
    assert transpile('<p>Hello</p>', 'Greeting.hyper', color=True) == transpile(
        '<p>Hello</p>', 'Greeting.hyper'
    )
