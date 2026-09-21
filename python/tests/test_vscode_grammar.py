import json
import re
from pathlib import Path


def test_component_declarations_have_shared_textmate_rules():
    path = Path(__file__).parents[2] / "editors/vscode/Syntaxes/hyper.tmLanguage.json"
    grammar = json.loads(path.read_text())
    patterns = grammar["repository"]["definitions"]["patterns"]

    definition = re.compile(patterns[0]["begin"])
    decorator = re.compile(patterns[1]["match"])

    assert definition.search("def Card(*, title: str) -> Component:")
    assert definition.search("async def Card(")
    assert decorator.search("    @subcomponent")


def test_template_expressions_use_python_grammar():
    path = Path(__file__).parents[2] / "editors/vscode/Syntaxes/hyper.tmLanguage.json"
    grammar = json.loads(path.read_text())
    repository = grammar["repository"]

    expression = repository["expression"]
    assert expression["beginCaptures"]["1"]["name"] == (
        "constant.character.format.placeholder.other.python"
    )
    assert expression["endCaptures"]["1"]["name"] == (
        "constant.character.format.placeholder.other.python"
    )
    assert {"include": "#expression"} in repository["tag-attributes"]["patterns"]
    assert {"include": "source.python"} in repository["python-expr"]["patterns"]
