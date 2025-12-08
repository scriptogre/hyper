# Real-World Examples

This document shows real examples using public APIs and GitHub data.

## URL-Based Loading

### Example 1: GitHub Repositories

Load your GitHub repositories directly from the API:

```python
from hyper import Collection, computed

class GitHubRepo(Collection):
    name: str
    description: str | None
    stargazers_count: int
    language: str | None
    html_url: str

    class Meta:
        url = "https://api.github.com/users/torvalds/repos"

    @computed
    def stars_formatted(self) -> str:
        if self.stargazers_count >= 1000:
            return f"{self.stargazers_count // 1000}k ⭐"
        return f"{self.stargazers_count} ⭐"

    @computed
    def display_name(self) -> str:
        desc = self.description or "No description"
        return f"{self.name}: {desc} ({self.stars_formatted})"

# Load Linus Torvalds' repos
repos = GitHubRepo.load()

for repo in repos[:5]:
    print(repo.display_name)
    print(f"  → {repo.html_url}")
```

**Output:**
```
linux: Linux kernel source tree (66k ⭐)
  → https://github.com/torvalds/linux
subsurface: Subsurface divelog (1k ⭐)
  → https://github.com/torvalds/subsurface
```

---

### Example 2: GitHub User Profile (Singleton)

Load a single user's profile:

```python
from hyper import Singleton, computed

class GitHubUser(Singleton):
    login: str
    name: str
    bio: str | None
    public_repos: int
    followers: int
    following: int
    html_url: str

    class Meta:
        url = "https://api.github.com/users/gvanrossum"

    @computed
    def follower_ratio(self) -> float:
        if self.following == 0:
            return float(self.followers)
        return self.followers / self.following

    @computed
    def profile_summary(self) -> str:
        return f"{self.name} (@{self.login}): {self.public_repos} repos, {self.followers} followers"

# Load Guido van Rossum's profile
guido = GitHubUser.load()
print(guido.profile_summary)
print(f"Bio: {guido.bio}")
print(f"Follower ratio: {guido.follower_ratio:.1f}x")
```

---

### Example 3: JSON Placeholder (Common Test API)

```python
from hyper import Collection, computed

class Post(Collection):
    userId: int
    id: int
    title: str
    body: str

    class Meta:
        url = "https://jsonplaceholder.typicode.com/posts"

    @computed
    def word_count(self) -> int:
        return len(self.body.split())

    @computed
    def reading_time(self) -> str:
        minutes = max(1, self.word_count // 200)
        return f"{minutes} min"

# Load all posts
posts = Post.load()

# Find longest post
longest = max(posts, key=lambda p: p.word_count)
print(f"Longest post: '{longest.title}'")
print(f"Words: {longest.word_count}, Reading time: {longest.reading_time}")
```

---

### Example 4: REST Countries API

```python
from hyper import Collection, computed

class Country(Collection):
    name: dict  # Contains common, official names
    cca3: str   # 3-letter code
    population: int
    area: float | None
    region: str
    capital: list[str] | None

    class Meta:
        url = "https://restcountries.com/v3.1/region/europe"

    @computed
    def common_name(self) -> str:
        return self.name.get("common", "Unknown")

    @computed
    def population_millions(self) -> float:
        return round(self.population / 1_000_000, 2)

    @computed
    def density(self) -> float | None:
        if self.area and self.area > 0:
            return round(self.population / self.area, 1)
        return None

# Load European countries
countries = Country.load()

# Find most populous
most_populous = max(countries, key=lambda c: c.population)
print(f"{most_populous.common_name}: {most_populous.population_millions}M people")

# Find most dense
dense_countries = [c for c in countries if c.density]
most_dense = max(dense_countries, key=lambda c: c.density or 0)
print(f"Most dense: {most_dense.common_name} ({most_dense.density} people/km²)")
```

---

## File-Based Loading

### Example 5: Blog with Markdown

```python
from hyper import MarkdownCollection, computed
from datetime import datetime

class BlogPost(MarkdownCollection):
    title: str
    date: str
    author: str
    tags: list[str]
    # Inherited from Markdown: body, html, slug

    class Meta:
        pattern = "content/blog/**/*.md"

    @computed
    def date_formatted(self) -> str:
        dt = datetime.strptime(self.date, "%Y-%m-%d")
        return dt.strftime("%B %d, %Y")

    @computed
    def reading_time(self) -> str:
        words = len(self.body.split())
        minutes = max(1, words // 200)
        return f"{minutes} min read"

# Load all blog posts
posts = BlogPost.load()

# Group by tag
from collections import defaultdict
by_tag = defaultdict(list)
for post in posts:
    for tag in post.tags:
        by_tag[tag].append(post)

print(f"Found {len(posts)} posts across {len(by_tag)} tags")
```

---

## Custom Loaders

### Example 6: Fetch from GitHub Raw Content

```python
from hyper import Collection
import json
from urllib.request import urlopen

class Package(Collection):
    name: str
    version: str
    description: str

    @classmethod
    def load(cls) -> list['Package']:
        # Load package.json from a GitHub repo
        url = "https://raw.githubusercontent.com/nodejs/node/main/package.json"

        with urlopen(url) as response:
            data = json.loads(response.read().decode('utf-8'))

        # Transform single package.json into a list of dependencies
        deps = []
        for name, version in data.get("dependencies", {}).items():
            deps.append(cls(
                name=name,
                version=version,
                description=f"Dependency of Node.js"
            ))

        return deps

packages = Package.load()
print(f"Node.js has {len(packages)} dependencies")
```

---

### Example 7: Combine Multiple Sources

```python
from hyper import Collection

class Article(Collection):
    title: str
    source: str
    url: str

    @classmethod
    def load(cls) -> list['Article']:
        # Combine file-based + API-based content
        from hyper import load

        # Load from local files
        local_articles = []
        try:
            local_data = load("articles/*.json")
            for item in local_data:
                local_articles.append(cls(**item, source="local"))
        except FileNotFoundError:
            pass

        # Load from API
        from urllib.request import urlopen
        import json

        with urlopen("https://jsonplaceholder.typicode.com/posts") as response:
            api_data = json.loads(response.read().decode('utf-8'))

        api_articles = [
            cls(
                title=item["title"],
                source="api",
                url=f"https://example.com/posts/{item['id']}"
            )
            for item in api_data[:5]  # Just first 5
        ]

        return local_articles + api_articles

articles = Article.load()
print(f"Loaded {len(articles)} articles from multiple sources")
```

---

## Advanced Examples

### Example 8: GitHub Releases with Caching

```python
from hyper import Collection, computed
import hashlib

class GitHubRelease(Collection):
    tag_name: str
    name: str
    published_at: str
    assets: list[dict]

    class Meta:
        url = "https://api.github.com/repos/python/cpython/releases"

    @computed
    def version(self) -> str:
        # Clean up version tag (v3.12.0 -> 3.12.0)
        return self.tag_name.lstrip('v')

    @computed
    def asset_count(self) -> int:
        return len(self.assets)

    @computed
    def total_download_size(self) -> int:
        return sum(asset.get('size', 0) for asset in self.assets)

    @computed
    def size_mb(self) -> float:
        return round(self.total_download_size / (1024 * 1024), 1)

# Load Python releases
releases = GitHubRelease.load()

print(f"Latest Python releases:")
for release in releases[:5]:
    print(f"  {release.version}: {release.asset_count} assets ({release.size_mb} MB)")
```

---

### Example 9: Pydantic Validation with URL Loading

```python
from hyper import Collection
from pydantic import BaseModel, Field, field_validator

class GitHubRepo(Collection, BaseModel):
    name: str = Field(min_length=1)
    stargazers_count: int = Field(ge=0, alias="stargazers_count")
    html_url: str

    class Meta:
        url = "https://api.github.com/users/openai/repos"

    @field_validator('name')
    @classmethod
    def name_must_not_be_empty(cls, v: str) -> str:
        if not v.strip():
            raise ValueError('Repository name cannot be empty')
        return v

# Load with automatic Pydantic validation
repos = GitHubRepo.load()
print(f"Loaded {len(repos)} validated OpenAI repositories")
```

---

### Example 10: Real-Time Data with Hooks

```python
from hyper import Collection
from pathlib import Path
from datetime import datetime

class WeatherData(Collection):
    temp: float
    humidity: int
    timestamp: str = ""

    class Meta:
        url = "https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&current=temperature_2m,relative_humidity_2m"

    @classmethod
    def load(cls) -> list['WeatherData']:
        from urllib.request import urlopen
        import json

        with urlopen(cls.Meta.url) as response:
            data = json.loads(response.read().decode('utf-8'))

        current = data.get("current", {})

        return [cls(
            temp=current.get("temperature_2m", 0),
            humidity=current.get("relative_humidity_2m", 0),
            timestamp=datetime.now().isoformat()
        )]

# Load current weather
weather = WeatherData.load()
if weather:
    w = weather[0]
    print(f"Temperature: {w.temp}°C")
    print(f"Humidity: {w.humidity}%")
    print(f"Updated: {w.timestamp}")
```

---

## Quick Start

Try these examples right now:

```bash
# Install
pip install hyper-content

# Run examples
python examples.py
```

```python
# examples.py
from hyper import Collection

class Repo(Collection):
    name: str
    stargazers_count: int

    class Meta:
        url = "https://api.github.com/users/github/repos"

repos = Repo.load()
top_5 = sorted(repos, key=lambda r: r.stargazers_count, reverse=True)[:5]

print("Top 5 GitHub repos:")
for repo in top_5:
    print(f"  ⭐ {repo.stargazers_count:,} - {repo.name}")
```

**Output:**
```
Top 5 GitHub repos:
  ⭐ 50,000+ - gitignore
  ⭐ 40,000+ - semantic
  ⭐ 30,000+ - choosealicense.com
  ...
```
