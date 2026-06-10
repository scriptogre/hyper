# Render benchmarks

Repeatable before/after harness for runtime optimizations. Measures the
compiled-template render path, not a comparison against other engines (that
lives in `docs/benchmarks.md`).

`conftest.py` compiles `templates/*.hyper` with the release binary once per
session, then imports the generated component.

## Run

```bash
just build                                  # release binary must exist

# Capture a baseline
uv run pytest python/tests/bench --benchmark-save=baseline

# Apply one optimization, then compare
uv run pytest python/tests/bench --benchmark-compare=baseline

# Fail if any benchmark regresses >5% on the median
uv run pytest python/tests/bench \
    --benchmark-compare=baseline --benchmark-compare-fail=median:5%
```

Saved runs live in `.benchmarks/`. List them with `uv run pytest-benchmark list`.

## What each test covers

| Test | Measures |
|---|---|
| `test_render_full_page[small/medium/large]` | Full page render: bind + iterate + escape + join. 10 / 100 / 500 products. |
| `test_render_stream_chunks` | Generation only, no final join. Isolates generator + escape cost. |
| `test_escape_single[plain/some_special/heavy]` | `escape_html` on one string, common to worst case. |
| `test_escape_loop_batch` | 1000 escapes in a tight loop (large list hot path). |

## Workflow rules

1. **Profile before optimizing.** `python -m cProfile` or `py-spy` to confirm
   where time goes. Don't optimize on a hunch.
2. **One change per run.** Change a single thing, re-run `--benchmark-compare`,
   then decide. Bundled changes hide which one helped.
3. **Quiet machine.** Close other apps; the `large` render is GC-sensitive and
   its stddev spikes under load.
