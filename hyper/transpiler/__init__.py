"""
Hyper template transpiler.

Transforms .hyper files into valid Python for IDE support and execution.
"""

from pathlib import Path
from _hyper_native import transpile, TranspileResult, SourceMapping


def transpile_to_file(hyper_path: str | Path, output_path: str | Path | None = None) -> Path:
    """Transpile a .hyper file to a .py file."""
    hyper_path = Path(hyper_path)
    if output_path is None:
        output_path = hyper_path.with_suffix('.py')
    else:
        output_path = Path(output_path)

    source = hyper_path.read_text()
    result = transpile(source)
    output_path.write_text(result.python_code)
    return output_path


__all__ = ["transpile", "transpile_to_file", "TranspileResult", "SourceMapping"]
