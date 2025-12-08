#!/usr/bin/env python3
"""Real-world examples using public APIs."""

from hyper.content import Collection, Singleton, computed
from typing import Any


# Example 1: GitHub Repositories
print("=" * 60)
print("Example 1: Top 5 Starred GitHub Repos")
print("=" * 60)


class GitHubRepo(Collection):
    name: str
    stargazers_count: int
    html_url: str
    description: str | None

    class Meta:
        url = "https://api.github.com/users/github/repos"

    @computed
    def stars_formatted(self) -> str:
        if self.stargazers_count >= 1000:
            return f"{self.stargazers_count // 1000}k"
        return str(self.stargazers_count)

    @classmethod
    def load(cls) -> list['GitHubRepo']:
        from urllib.request import urlopen
        import json

        with urlopen(cls.Meta.url) as response:
            data = json.loads(response.read().decode('utf-8'))

        # Only extract fields we care about
        return [cls(
            name=item["name"],
            stargazers_count=item["stargazers_count"],
            html_url=item["html_url"],
            description=item.get("description")
        ) for item in data]


try:
    repos = GitHubRepo.load()
    top_5 = sorted(repos, key=lambda r: r.stargazers_count, reverse=True)[:5]

    for i, repo in enumerate(top_5, 1):
        print(f"{i}. ⭐ {repo.stars_formatted:>6} - {repo.name}")
        if repo.description:
            print(f"   {repo.description[:60]}")
except Exception as e:
    print(f"Error: {e}")

print()

# Example 2: GitHub User Profile
print("=" * 60)
print("Example 2: GitHub User Profile (Guido van Rossum)")
print("=" * 60)


class GitHubUser(Singleton):
    login: str
    name: str | None
    bio: str | None
    public_repos: int
    followers: int
    following: int

    class Meta:
        url = "https://api.github.com/users/gvanrossum"

    @computed
    def follower_ratio(self) -> float:
        if self.following == 0:
            return float(self.followers)
        return round(self.followers / self.following, 1)

    @classmethod
    def load(cls) -> 'GitHubUser':
        from urllib.request import urlopen
        import json

        with urlopen(cls.Meta.url) as response:
            data = json.loads(response.read().decode('utf-8'))

        # Only extract fields we care about
        return cls(
            login=data["login"],
            name=data.get("name"),
            bio=data.get("bio"),
            public_repos=data["public_repos"],
            followers=data["followers"],
            following=data["following"]
        )


try:
    user = GitHubUser.load()
    print(f"Name: {user.name}")
    print(f"Username: @{user.login}")
    print(f"Bio: {user.bio}")
    print(f"Public repos: {user.public_repos}")
    print(f"Followers: {user.followers:,}")
    print(f"Following: {user.following}")
    print(f"Follower ratio: {user.follower_ratio}x")
except Exception as e:
    print(f"Error: {e}")

print()

# Example 3: REST Countries
print("=" * 60)
print("Example 3: European Countries (Population)")
print("=" * 60)


class Country(Collection):
    name: dict
    population: int
    cca3: str

    class Meta:
        url = "https://restcountries.com/v3.1/region/europe"

    @computed
    def common_name(self) -> str:
        return self.name.get("common", "Unknown")

    @computed
    def population_millions(self) -> float:
        return round(self.population / 1_000_000, 1)

    @classmethod
    def load(cls) -> list['Country']:
        from urllib.request import urlopen
        import json

        with urlopen(cls.Meta.url) as response:
            data = json.loads(response.read().decode('utf-8'))

        # Only extract fields we care about
        return [cls(
            name=item["name"],
            population=item["population"],
            cca3=item["cca3"]
        ) for item in data]


try:
    countries = Country.load()

    # Top 5 most populous
    top_5 = sorted(countries, key=lambda c: c.population, reverse=True)[:5]

    for i, country in enumerate(top_5, 1):
        print(f"{i}. {country.common_name}: {country.population_millions}M people")
except Exception as e:
    print(f"Error: {e}")

print()

# Example 4: JSON Placeholder Posts
print("=" * 60)
print("Example 4: Blog Posts with Reading Time")
print("=" * 60)


class Post(Collection):
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

    @classmethod
    def load(cls) -> list['Post']:
        from urllib.request import urlopen
        import json

        with urlopen(cls.Meta.url) as response:
            data = json.loads(response.read().decode('utf-8'))

        # Only extract fields we care about
        return [cls(
            id=item["id"],
            title=item["title"],
            body=item["body"]
        ) for item in data]


try:
    posts = Post.load()

    # Show 3 random posts with longest reading time
    longest = sorted(posts, key=lambda p: p.word_count, reverse=True)[:3]

    for post in longest:
        print(f"Post #{post.id}: {post.title[:50]}...")
        print(f"  Words: {post.word_count}, Reading time: {post.reading_time}")
        print()
except Exception as e:
    print(f"Error: {e}")

print("=" * 60)
print("All examples completed!")
print("=" * 60)
