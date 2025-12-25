"""Markdown parser with YAML frontmatter support."""

from pathlib import Path

import markdown as md_lib


class MarkdownParser:
    """Markdown parser with optional YAML frontmatter."""

    @staticmethod
    def can_parse(path: Path) -> bool:
        return path.suffix.lower() in (".md", ".markdown")

    @staticmethod
    def parse(content: bytes, target_type: type | None = None) -> dict:
        text = content.decode("utf-8")

        # Try to parse frontmatter if present
        if text.startswith("---"):
            try:
                import yaml

                _, frontmatter, body = text.split("---", 2)
                data = yaml.safe_load(frontmatter) or {}
                markdown_content = body.strip()
                data["body"] = markdown_content
                data["html"] = md_lib.markdown(markdown_content)
                return data
            except ValueError:
                # If split fails, treat as regular markdown
                pass
            except ImportError:
                raise ImportError(
                    "YAML frontmatter requires PyYAML. Install with: uv add pyyaml"
                )

        # No frontmatter or failed to parse
        markdown_content = text.strip()
        return {
            "body": markdown_content,
            "html": md_lib.markdown(markdown_content),
        }
