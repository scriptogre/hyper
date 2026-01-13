"""Tests for Markdown mixin functionality."""

import os

import pytest
import pydantic

from hyper.content import MarkdownCollection


@pytest.fixture
def content_dir(tmp_path):
    """Set up test content directory with markdown files."""
    content = tmp_path / "content"
    content.mkdir()
    os.chdir(content)
    yield content


def test_markdown_basic_properties(content_dir):
    """Test basic markdown properties: body, html, slug."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str
        author: str

        class Meta:
            pattern = "posts/*.md"

    # Create test markdown file
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "hello-world.md").write_text(
        """---
title: Hello World
author: Alice
---

# Welcome

This is **bold** text."""
    )

    posts = BlogPost.load()
    assert len(posts) == 1

    post = posts[0]
    assert post.title == "Hello World"
    assert post.author == "Alice"
    assert post.body == "# Welcome\n\nThis is **bold** text."
    assert "<h1>" in post.html
    assert "<strong>bold</strong>" in post.html
    assert post.slug == "posts/hello-world"  # Auto-generated from relative path


def test_markdown_slug_override(content_dir):
    """Test that slug can be overridden by defining it in the model."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str
        slug: str  # Explicitly define to allow frontmatter override

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.md").write_text(
        """---
title: My Post
slug: custom-slug-here
---

Content"""
    )

    posts = BlogPost.load()
    post = posts[0]
    # When slug is explicitly defined in model, frontmatter can set it
    assert post.slug == "custom-slug-here"


def test_markdown_headings_extraction(content_dir):
    """Test heading extraction from markdown content."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.md").write_text(
        """---
title: My Post
---

# Main Title

Some intro text.

## Section 1

Content here.

### Subsection 1.1

More content.

## Section 2

Final content."""
    )

    posts = BlogPost.load()
    post = posts[0]

    # Check headings (4 total: h1, h2, h3, h2)
    assert len(post.headings) == 4
    assert post.headings[0].level == 1
    assert post.headings[0].text == "Main Title"
    assert post.headings[0].slug == "main-title"

    assert post.headings[1].level == 2
    assert post.headings[1].text == "Section 1"
    assert post.headings[1].slug == "section-1"

    assert post.headings[2].level == 3
    assert post.headings[2].text == "Subsection 1.1"
    assert post.headings[2].slug == "subsection-11"

    assert post.headings[3].level == 2
    assert post.headings[3].text == "Section 2"
    assert post.headings[3].slug == "section-2"


def test_markdown_toc_flat(content_dir):
    """Test TOC flat headings list."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.md").write_text(
        """---
title: My Post
---

# Title
## Section
### Subsection"""
    )

    posts = BlogPost.load()
    post = posts[0]

    # TOC headings is the same as post.headings
    assert post.toc.headings == post.headings
    assert len(post.toc.headings) == 3


def test_markdown_toc_nested(content_dir):
    """Test TOC nested structure."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.md").write_text(
        """---
title: My Post
---

# Main Title

## Section 1

### Subsection 1.1

### Subsection 1.2

## Section 2"""
    )

    posts = BlogPost.load()
    post = posts[0]

    nested = post.toc.nested()

    # Should have 1 root (h1)
    assert len(nested) == 1
    assert nested[0]["heading"].text == "Main Title"

    # h1 should have 2 children (h2s)
    assert len(nested[0]["children"]) == 2
    assert nested[0]["children"][0]["heading"].text == "Section 1"
    assert nested[0]["children"][1]["heading"].text == "Section 2"

    # First h2 should have 2 children (h3s)
    assert len(nested[0]["children"][0]["children"]) == 2
    assert nested[0]["children"][0]["children"][0]["heading"].text == "Subsection 1.1"
    assert nested[0]["children"][0]["children"][1]["heading"].text == "Subsection 1.2"


def test_markdown_with_msgspec(content_dir):
    """Test Markdown mixin works with msgspec.Struct."""
    import msgspec
    from hyper.content import MarkdownCollection

    class BlogPost(MarkdownCollection, msgspec.Struct):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.md").write_text(
        """---
title: Test
---

# Heading"""
    )

    posts = BlogPost.load()
    post = posts[0]

    # Verify it's a msgspec.Struct
    assert isinstance(post, msgspec.Struct)

    # Verify markdown properties work
    assert post.title == "Test"
    assert "# Heading" in post.body
    assert "<h1>" in post.html
    assert post.slug == "posts/post1"  # Now includes path
    assert len(post.headings) == 1


def test_markdown_without_frontmatter(content_dir):
    """Test markdown files without frontmatter still work."""

    class Doc(MarkdownCollection, pydantic.BaseModel):
        class Meta:
            pattern = "docs/*.md"

    docs_dir = content_dir / "docs"
    docs_dir.mkdir()
    (docs_dir / "readme.md").write_text(
        """# README

This is a readme file."""
    )

    docs = Doc.load()
    assert len(docs) == 1

    doc = docs[0]
    assert "# README" in doc.body
    assert "<h1>" in doc.html
    assert doc.slug == "docs/readme"  # Includes directory
    assert len(doc.headings) == 1
    assert doc.headings[0].text == "README"


def test_markdown_slug_with_nested_path(content_dir):
    """Test slug generation for nested markdown files."""

    class Doc(MarkdownCollection, pydantic.BaseModel):
        class Meta:
            pattern = "docs/**/*.md"

    docs_dir = content_dir / "docs"
    guides_dir = docs_dir / "guides"
    guides_dir.mkdir(parents=True)

    (guides_dir / "getting-started.md").write_text(
        """---
title: Getting Started
---

Content"""
    )

    docs = Doc.load()
    doc = docs[0]

    # Slug should include the full relative path
    assert doc.slug == "docs/guides/getting-started"


# ==========================================
# Advanced Markdown Tests (from test_markdown_advanced.py)
# ==========================================

try:
    import msgspec

    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False

from dataclasses import is_dataclass, fields  # noqa: E402
from hyper.content import MarkdownSingleton  # noqa: E402


@pytest.fixture
def advanced_content_dir(tmp_path):
    """Set up test content directory with markdown files."""
    content = tmp_path / "content"
    content.mkdir()
    os.chdir(content)
    yield content


# ===========================================
# isinstance() and issubclass() Support Tests
# ===========================================


def test_isinstance_with_pydantic(content_dir):
    """Test isinstance() works with MarkdownCollection + pydantic."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "test.md").write_text(
        """---
title: Test Post
---

# Content"""
    )

    posts = BlogPost.load()
    post = posts[0]

    # Test isinstance with MarkdownCollection
    assert isinstance(post, MarkdownCollection)
    assert isinstance(post, pydantic.BaseModel)

    # Test issubclass
    assert issubclass(BlogPost, MarkdownCollection)
    assert issubclass(BlogPost, pydantic.BaseModel)


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_isinstance_with_msgspec(content_dir):
    """Test isinstance() works with MarkdownCollection + msgspec."""

    class BlogPost(MarkdownCollection, msgspec.Struct):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "test.md").write_text(
        """---
title: Test Post
---

# Content"""
    )

    posts = BlogPost.load()
    post = posts[0]

    # Test isinstance with MarkdownCollection
    assert isinstance(post, MarkdownCollection)
    assert isinstance(post, msgspec.Struct)

    # Test issubclass
    assert issubclass(BlogPost, MarkdownCollection)
    assert issubclass(BlogPost, msgspec.Struct)


def test_isinstance_with_auto_dataclass(content_dir):
    """Test isinstance() works with auto-dataclass MarkdownCollection."""

    class BlogPost(MarkdownCollection):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "test.md").write_text(
        """---
title: Test Post
---

# Content"""
    )

    posts = BlogPost.load()
    post = posts[0]

    # Test isinstance with MarkdownCollection
    assert isinstance(post, MarkdownCollection)

    # Test issubclass
    assert issubclass(BlogPost, MarkdownCollection)


def test_isinstance_with_singleton(content_dir):
    """Test isinstance() works with MarkdownSingleton."""

    class Config(MarkdownSingleton, pydantic.BaseModel):
        theme: str

        class Meta:
            pattern = "config.md"

    (content_dir / "config.md").write_text(
        """---
theme: dark
---

# Config"""
    )

    config = Config.load()

    # Test isinstance with MarkdownSingleton
    assert isinstance(config, MarkdownSingleton)
    assert isinstance(config, pydantic.BaseModel)

    # Test issubclass
    assert issubclass(Config, MarkdownSingleton)
    assert issubclass(Config, pydantic.BaseModel)


# ===========================================
# msgspec Base Class Order Tests
# ===========================================


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_wrong_order_raises_helpful_error():
    """Test that wrong base class order gives helpful error message."""

    with pytest.raises(TypeError) as exc_info:

        class BlogPost(msgspec.Struct, MarkdownCollection):
            title: str

    error_msg = str(exc_info.value)
    assert "MarkdownCollection must come before msgspec.Struct" in error_msg
    assert "class YourClass(MarkdownCollection, msgspec.Struct)" in error_msg
    assert "Not: class YourClass(msgspec.Struct, MarkdownCollection)" in error_msg


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_singleton_wrong_order_raises_helpful_error():
    """Test that wrong base class order with Singleton gives helpful error."""

    with pytest.raises(TypeError) as exc_info:

        class Config(msgspec.Struct, MarkdownSingleton):
            theme: str

    error_msg = str(exc_info.value)
    assert "MarkdownSingleton must come before msgspec.Struct" in error_msg
    assert "class YourClass(MarkdownSingleton, msgspec.Struct)" in error_msg


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_correct_order_works(content_dir):
    """Test that correct base class order works fine."""

    class BlogPost(MarkdownCollection, msgspec.Struct):
        title: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "test.md").write_text(
        """---
title: Test Post
---

# Content"""
    )

    # Should load without errors
    posts = BlogPost.load()
    assert len(posts) == 1
    assert posts[0].title == "Test Post"


# ===========================================
# Auto-dataclass Tests
# ===========================================


def test_auto_dataclass_without_decorator(content_dir):
    """Test that MarkdownCollection auto-applies @dataclass."""

    class BlogPost(MarkdownCollection):
        title: str
        author: str

        class Meta:
            pattern = "posts/*.md"

    # Verify it's a real dataclass
    assert is_dataclass(BlogPost)
    assert hasattr(BlogPost, "__dataclass_fields__")

    # Verify dataclass utilities work
    field_list = fields(BlogPost)
    field_names = [f.name for f in field_list]
    assert "title" in field_names
    assert "author" in field_names


def test_auto_dataclass_has_init_repr_eq():
    """Test that auto-dataclass has __init__, __repr__, __eq__."""

    class BlogPost(MarkdownCollection):
        title: str

    # Check for generated methods
    assert hasattr(BlogPost, "__init__")
    assert hasattr(BlogPost, "__repr__")
    assert hasattr(BlogPost, "__eq__")

    # Verify they work (can't fully test without loader, but can check signatures)
    import inspect

    sig = inspect.signature(BlogPost.__init__)
    params = list(sig.parameters.keys())
    assert "title" in params


def test_auto_dataclass_with_pydantic_does_not_apply():
    """Test that auto-dataclass doesn't apply when using pydantic."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str

    # Should be a pydantic model, not a dataclass
    assert hasattr(BlogPost, "model_fields")
    # is_dataclass returns True for pydantic models in some versions, so we check for pydantic-specific attrs
    assert hasattr(BlogPost, "model_validate")


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_auto_dataclass_with_msgspec_does_not_apply():
    """Test that auto-dataclass doesn't apply when using msgspec."""

    class BlogPost(MarkdownCollection, msgspec.Struct):
        title: str

    # Should be a msgspec.Struct, not a dataclass
    assert hasattr(BlogPost, "__struct_fields__")
    assert isinstance(BlogPost, type(msgspec.Struct))


def test_auto_dataclass_singleton(content_dir):
    """Test that MarkdownSingleton also auto-applies @dataclass."""

    class Config(MarkdownSingleton):
        theme: str

        class Meta:
            pattern = "config.md"

    # Verify it's a real dataclass
    assert is_dataclass(Config)
    assert hasattr(Config, "__dataclass_fields__")


# ===========================================
# msgspec Field Ordering Tests
# ===========================================


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_field_order():
    """Test that msgspec puts inherited fields (id, body, html) first."""

    class BlogPost(MarkdownCollection, msgspec.Struct):
        title: str
        author: str

    # Check field order
    assert BlogPost.__struct_fields__ == ("id", "body", "html", "title", "author")

    # id, body, html come first (from _MarkdownStructBase)
    # then user fields (title, author)


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_annotations_include_all_fields():
    """Test that msgspec struct fields include all fields."""

    class BlogPost(MarkdownCollection, msgspec.Struct):
        title: str
        author: str

    # Check struct fields instead of annotations (msgspec doesn't expose all in __annotations__)
    struct_fields = BlogPost.__struct_fields__
    assert "id" in struct_fields
    assert "body" in struct_fields
    assert "html" in struct_fields
    assert "title" in struct_fields
    assert "author" in struct_fields


# ===========================================
# Integration Tests
# ===========================================


def test_markdown_collection_with_all_features(content_dir):
    """Integration test combining multiple features."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str
        author: str

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.md").write_text(
        """---
title: First Post
author: Alice
---

# Introduction

This is the **first** post.

## Section 1

Content here."""
    )

    posts = BlogPost.load()
    post = posts[0]

    # Test all markdown features work
    assert post.title == "First Post"
    assert post.author == "Alice"
    assert "# Introduction" in post.body
    assert "<h1>" in post.html
    assert "<strong>first</strong>" in post.html
    assert post.slug == "posts/post1"

    # Test headings
    assert len(post.headings) == 2
    assert post.headings[0].text == "Introduction"
    assert post.headings[1].text == "Section 1"

    # Test TOC
    toc = post.toc.nested()
    assert len(toc) == 1
    assert toc[0]["heading"].text == "Introduction"
    assert len(toc[0]["children"]) == 1

    # Test isinstance
    assert isinstance(post, MarkdownCollection)
    assert isinstance(post, pydantic.BaseModel)

    # Test issubclass
    assert issubclass(BlogPost, MarkdownCollection)


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_markdown_collection_msgspec_full_integration(content_dir):
    """Integration test with msgspec."""

    class BlogPost(MarkdownCollection, msgspec.Struct):
        title: str
        published: bool = False

        class Meta:
            pattern = "posts/*.md"

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.md").write_text(
        """---
title: Msgspec Post
published: true
---

# Msgspec Test

Testing **msgspec** integration."""
    )

    posts = BlogPost.load()
    post = posts[0]

    # Test msgspec + markdown features
    assert isinstance(post, msgspec.Struct)
    assert isinstance(post, MarkdownCollection)
    assert post.title == "Msgspec Post"
    assert post.published is True
    assert "# Msgspec Test" in post.body
    assert "<h1>" in post.html

    # Test struct fields include markdown fields
    assert "id" in BlogPost.__struct_fields__
    assert "body" in BlogPost.__struct_fields__
    assert "html" in BlogPost.__struct_fields__
    assert "title" in BlogPost.__struct_fields__


def test_markdown_singleton_auto_dataclass_basic(content_dir):
    """Test auto-dataclass with MarkdownSingleton (basic non-markdown features)."""

    class Config(MarkdownSingleton):
        theme: str
        version: str

        class Meta:
            pattern = "config.json"

    (content_dir / "config.json").write_text(
        """{"theme": "dark", "version": "1.0.0"}"""
    )

    config = Config.load()

    # Test it's a dataclass
    assert is_dataclass(Config)

    # Test singleton features
    assert isinstance(config, MarkdownSingleton)
    assert config.theme == "dark"
    assert config.version == "1.0.0"

    # Test dataclass features
    from dataclasses import fields as get_fields

    field_names = [f.name for f in get_fields(Config)]
    assert "theme" in field_names
    assert "version" in field_names


# ===========================================
# Edge Case Tests
# ===========================================


def test_multiple_markdown_classes_dont_interfere():
    """Test that multiple MarkdownCollection classes don't interfere."""

    class BlogPost(MarkdownCollection, pydantic.BaseModel):
        title: str

    class NewsPost(MarkdownCollection, pydantic.BaseModel):
        headline: str

    # Both should work independently
    assert issubclass(BlogPost, MarkdownCollection)
    assert issubclass(NewsPost, MarkdownCollection)
    assert BlogPost is not NewsPost


def test_markdown_collection_reusable():
    """Test that MarkdownCollection can be used multiple times."""

    # Create multiple classes using MarkdownCollection
    classes = []
    for i in range(5):

        class Post(MarkdownCollection, pydantic.BaseModel):
            title: str

        classes.append(Post)

    # All should be valid
    for cls in classes:
        assert issubclass(cls, MarkdownCollection)


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_with_custom_struct_config():
    """Test msgspec with custom Struct configuration."""

    class BlogPost(MarkdownCollection, msgspec.Struct, frozen=True):
        title: str

    # Should work with custom msgspec config
    assert BlogPost.__struct_config__.frozen is True
    assert "id" in BlogPost.__struct_fields__
    assert "body" in BlogPost.__struct_fields__
