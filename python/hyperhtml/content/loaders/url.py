"""URL-based loader for fetching content from HTTP endpoints."""

from typing import Any


def load_from_url(url: str) -> list[dict[str, Any]]:
    """Load data from a URL that returns JSON.

    Args:
        url: HTTP(S) URL that returns JSON (array or single object)

    Returns:
        List of dicts to be converted to typed instances

    Raises:
        ImportError: If urllib is not available
        ValueError: If response is not valid JSON or URL scheme is not allowed
    """
    try:
        from urllib.request import urlopen
        from urllib.parse import urlparse
        import json
    except ImportError as e:
        raise ImportError("urllib is required for URL-based loading") from e

    # Validate URL scheme to prevent file:// and other unexpected protocols
    parsed = urlparse(url)
    if parsed.scheme not in ("http", "https"):
        raise ValueError(f"Only http/https URLs are allowed, got: {parsed.scheme}://")

    # Fetch data
    with urlopen(url) as response:  # nosec B310 - scheme validated above
        data = json.loads(response.read().decode("utf-8"))

    # Ensure we return a list
    if isinstance(data, list):
        return data
    elif isinstance(data, dict):
        # Single object - wrap in list
        return [data]
    else:
        raise ValueError(f"Expected JSON array or object, got {type(data)}")
