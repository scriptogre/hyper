"""Slot sentinel for child content injection."""

slot = ...


def is_slot(value) -> bool:
    """Check if value is the slot sentinel."""
    return value is ...
