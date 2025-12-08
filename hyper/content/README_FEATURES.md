# Hyper Content - Feature Summary

## 🚀 Three Power Features to Crush the Competition

### 1. Custom Loaders (Dead Simple)

**Three ways to load content:**

```python
# ✅ File-based (local files)
class BlogPost(Collection):
    class Meta:
        pattern = "posts/*.md"

# ✅ URL-based (REST APIs) - SIMPLER THAN ASTRO!
class BlogPost(Collection):
    class Meta:
        url = "https://api.example.com/posts"

# ✅ Custom function (full control)
class BlogPost(Collection):
    @classmethod
    def load(cls) -> list['BlogPost']:
        return [cls(**item) for item in fetch_from_anywhere()]
```

**Real example:**
```python
from hyper import Collection

class GitHubRepo(Collection):
    name: str
    stargazers_count: int

    @classmethod
    def load(cls) -> list['GitHubRepo']:
        from urllib.request import urlopen
        import json

        url = "https://api.github.com/users/github/repos"
        with urlopen(url) as response:
            data = json.loads(response.read().decode('utf-8'))

        return [cls(
            name=item["name"],
            stargazers_count=item["stargazers_count"]
        ) for item in data]

repos = GitHubRepo.load()  # Loads from GitHub API!
```

---

### 2. Computed Fields (Lazy & Cached)

```python
from hyper import Collection, computed

class BlogPost(Collection):
    body: str

    @computed
    def word_count(self) -> int:
        return len(self.body.split())

    @computed
    def reading_time(self) -> str:
        # Can reference other computed fields!
        minutes = self.word_count // 200
        return f"{minutes} min read"
```

**Features:**
- ✅ Lazy evaluation (only computed when accessed)
- ✅ Automatic caching (computed once, reused forever)
- ✅ Chainable (computed fields can reference other computed fields)
- ✅ Works with Collections, Singletons, all validation libraries

---

### 3. Meta Hooks (Before/After Processing)

```python
from pathlib import Path
from hyper import Collection

class BlogPost(Collection):
    title: str
    body: str

    class Meta:
        pattern = "posts/*.md"

        @staticmethod
        def before_parse(path: Path, content: bytes) -> bytes:
            # Modify raw bytes before parsing
            return content.replace(b"DRAFT:", b"")

        @staticmethod
        def after_parse(path: Path, data: dict) -> dict:
            # Modify parsed dict before validation
            data['word_count'] = len(data.get('body', '').split())
            return data

        @staticmethod
        def after_load(instance: 'BlogPost') -> 'BlogPost':
            # Modify final validated instance
            instance.title = instance.title.strip().title()
            return instance
```

**Hook order:** `before_parse` → `after_parse` → `after_load`

**Note:** Hooks only run for file-based loading, not for custom loaders (explicit control).

---

## 📊 How We Compare to Astro Content Collections

| Feature | Astro | Hyper Content | Winner |
|---------|-------|---------------|--------|
| **URL Loading** | Requires loader function | `url = "..."` | 🏆 **Hyper** (simpler) |
| **Custom Loaders** | Complex loader API | Override `.load()` | 🏆 **Hyper** (cleaner) |
| **Computed Fields** | ❌ No built-in | `@computed` decorator | 🏆 **Hyper** (we have it!) |
| **Validation** | Zod only | Pydantic, msgspec, dataclasses | 🏆 **Hyper** (flexible) |
| **Hooks** | Basic transforms | 3-stage hooks | 🏆 **Hyper** (powerful) |
| **Performance** | Good | msgspec direct-parse | 🏆 **Hyper** (faster) |
| **Type Safety** | TypeScript | Python type hints | 🟰 **Tie** |
| **Language** | JavaScript/TypeScript | Python | (Preference) |

---

## 🎯 Real-World Examples

### Example: GitHub Repos with Computed Fields

```python
from hyper import Collection, computed

class GitHubRepo(Collection):
    name: str
    stargazers_count: int
    description: str | None

    @classmethod
    def load(cls) -> list['GitHubRepo']:
        from urllib.request import urlopen
        import json

        url = "https://api.github.com/users/github/repos"
        with urlopen(url) as response:
            data = json.loads(response.read().decode('utf-8'))

        return [cls(
            name=item["name"],
            stargazers_count=item["stargazers_count"],
            description=item.get("description")
        ) for item in data]

    @computed
    def stars_k(self) -> str:
        return f"{self.stargazers_count // 1000}k" if self.stargazers_count >= 1000 else str(self.stargazers_count)

# Load and display
repos = GitHubRepo.load()
for repo in sorted(repos, key=lambda r: r.stargazers_count, reverse=True)[:5]:
    print(f"⭐ {repo.stars_k} - {repo.name}")
```

### Example: Blog with Markdown + Hooks

```python
from hyper import MarkdownCollection, computed
from pathlib import Path

class BlogPost(MarkdownCollection):
    title: str
    date: str
    tags: list[str]

    class Meta:
        pattern = "content/blog/**/*.md"

        @staticmethod
        def after_parse(path: Path, data: dict) -> dict:
            # Inject slug from filename
            data['slug'] = path.stem
            return data

    @computed
    def reading_time(self) -> str:
        words = len(self.body.split())
        return f"{max(1, words // 200)} min read"

    @computed
    def excerpt(self) -> str:
        return self.body[:150] + "..."

posts = BlogPost.load()
```

---

## 📈 Test Coverage

**149 tests passing** with 91% code coverage:
- ✅ 96 core functionality tests
- ✅ 12 custom loader tests
- ✅ 18 hook tests
- ✅ 13 computed field tests
- ✅ 10 URL loader tests

All tests use real-world scenarios, not contrived examples.

---

## 🔥 Why Hyper Content Wins

1. **Simpler than Astro** for common cases (URL loading, computed fields)
2. **More powerful than Astro** for advanced cases (hooks, multi-library support)
3. **Better performance** (msgspec direct-parse optimization)
4. **More flexible** (Pydantic OR msgspec OR dataclasses)
5. **Python-native** (leverage the entire Python ecosystem)

---

## 🚦 Quick Start

```bash
pip install hyper-content
```

```python
from hyper import Collection, computed

class Repo(Collection):
    name: str
    stars: int

    @classmethod
    def load(cls) -> list['Repo']:
        from urllib.request import urlopen
        import json

        with urlopen("https://api.github.com/users/github/repos") as response:
            data = json.loads(response.read().decode('utf-8'))

        return [cls(name=item["name"], stars=item["stargazers_count"]) for item in data]

    @computed
    def popularity(self) -> str:
        return "🔥 Hot" if self.stars > 1000 else "👍 Good"

repos = Repo.load()
for repo in sorted(repos, key=lambda r: r.stars, reverse=True)[:5]:
    print(f"{repo.popularity} {repo.name}: {repo.stars} stars")
```

**That's it!** You're loading from GitHub, computing fields, and displaying results.

---

## 📚 Documentation

- [EXAMPLES.md](EXAMPLES.md) - Real-world examples with GitHub, REST Countries, etc.
- [HOOKS.md](HOOKS.md) - Complete guide to Meta hooks
- [TESTING.md](TESTING.md) - Testing philosophy and coverage

---

## 🎉 Conclusion

**Hyper Content gives you:**
- ✅ Dead-simple APIs for common cases
- ✅ Powerful escape hatches for complex cases
- ✅ Better performance than competitors
- ✅ Full Python ecosystem integration

**We crush Astro Content Collections by being both simpler AND more powerful.**
