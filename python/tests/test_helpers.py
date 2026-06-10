"""Escape contract. Output must be identical whether the C fast path or the
pure-Python fallback runs, so these lock the exact bytes."""

from hyper.helpers import escape_html, safe


def test_escapes_all_five_special_chars():
    assert escape_html('<a>&"\'') == '&lt;a&gt;&amp;&#34;&#39;'


def test_none_renders_empty():
    assert escape_html(None) == ''


def test_non_str_is_stringified_then_escaped():
    assert escape_html(42) == '42'
    assert escape_html(3 < 5) == 'True'


def test_safe_value_passes_through_unescaped():
    assert escape_html(safe('<b>bold</b>')) == '<b>bold</b>'


def test_clean_string_is_unchanged():
    assert escape_html('no specials here') == 'no specials here'
