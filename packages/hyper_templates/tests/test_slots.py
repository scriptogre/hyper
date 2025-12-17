"""Tests for slot system."""

import pytest
from hyper_templates.slots import slot, is_slot


class TestSlotSentinel:
    """Tests for the slot sentinel."""

    def test_slot_is_ellipsis(self):
        """The slot sentinel is the Ellipsis object."""
        assert slot is ...

    def test_is_slot_with_ellipsis(self):
        """is_slot returns True for Ellipsis."""
        assert is_slot(...) is True

    def test_is_slot_with_string(self):
        """is_slot returns False for strings."""
        assert is_slot("...") is False

    def test_is_slot_with_none(self):
        """is_slot returns False for None."""
        assert is_slot(None) is False

    def test_is_slot_with_int(self):
        """is_slot returns False for integers."""
        assert is_slot(0) is False

    def test_slot_in_dict(self):
        """Slot can be used as a dict value."""
        data = {"content": slot}
        assert is_slot(data["content"]) is True

    def test_slot_identity(self):
        """Slot is always the same object (identity)."""
        from hyper import slot as slot2
        assert slot is slot2
