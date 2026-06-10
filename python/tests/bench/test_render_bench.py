"""Macro benchmark: render a full page end to end.

`product_page(...)` eagerly iterates the generator, escapes every variable, and
joins the chunks into one string. This is the real "render a page" cost.

    pytest python/tests/bench/test_render_bench.py --benchmark-save=baseline
    # apply an optimization, then:
    pytest python/tests/bench/test_render_bench.py --benchmark-compare=baseline
"""

from __future__ import annotations

import pytest

from .conftest import make_products

TITLE = "All Products"


@pytest.mark.parametrize("n", [10, 100, 500], ids=["small", "medium", "large"])
def test_render_full_page(benchmark, product_page, n):
    products = make_products(n)
    result = benchmark(lambda: product_page(title=TITLE, products=products))
    assert result.startswith("<html")


def test_render_stream_chunks(benchmark, product_page):
    """Generation only (no final join), to isolate generator + escape cost."""
    products = make_products(100)
    chunks = benchmark(lambda: list(product_page.stream(title=TITLE, products=products)))
    assert len(chunks) > 100
