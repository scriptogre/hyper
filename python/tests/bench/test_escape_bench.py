"""Micro benchmark: escape_html alone, the per-variable hot path.

Isolates the 5x `.replace()` chain. Swap in MarkupSafe (or any change), then:

    pytest python/tests/bench/test_escape_bench.py --benchmark-compare=baseline

Cases span the common shape (plain text, no special chars, all 5 scans miss)
to the worst case (every char escaped).
"""

from __future__ import annotations

import pytest

from hyperhtml.helpers import escape_html

PLAIN = "A sturdy, well-made item for everyday use." * 4
SOME = 'Bolts & Nuts <model "X-1"> for the price of one'
HEAVY = '<>&"\'' * 40

CASES = {"plain": PLAIN, "some_special": SOME, "heavy": HEAVY}


@pytest.mark.parametrize("kind", list(CASES), ids=list(CASES))
def test_escape_single(benchmark, kind):
    s = CASES[kind]
    benchmark(lambda: escape_html(s))


def test_escape_loop_batch(benchmark):
    """1000 escapes in a tight loop, mirroring a large product list."""
    values = [SOME if i % 3 else PLAIN for i in range(1000)]

    def run():
        return [escape_html(v) for v in values]

    benchmark(run)
