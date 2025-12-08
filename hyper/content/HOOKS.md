# Meta Hooks

Hyper Content provides three lifecycle hooks that allow you to customize how content is loaded, parsed, and processed. These hooks are defined as static methods on the `Meta` class within your `Singleton` or `Collection` models.

## Hook Lifecycle

When loading content, hooks are executed in this order:

```
1. before_parse  →  Raw bytes from file
2. [Parser]      →  Parses to dict/list
3. after_parse   →  Modify parsed dict
4. [Converter]   →  Convert to typed model
5. after_load    →  Modify final instance
```

## Available Hooks

### `before_parse(path: Path, content: bytes) -> bytes`

Modify raw file bytes before parsing. Useful for:
- Decryption or decompression
- Encoding normalization
- Find-and-replace in raw content
- Removing BOMs or unwanted prefixes

**Example: Decrypt encrypted content**
```python
from pathlib import Path
from hyper import Singleton

class Secrets(Singleton):
    api_key: str
    database_url: str

    class Meta:
        pattern = "secrets.json.encrypted"

        @staticmethod
        def before_parse(path: Path, content: bytes) -> bytes:
            return decrypt(content)  # Your decryption function
```

**Example: Remove draft markers**
```python
from hyper import Collection

class BlogPost(Collection):
    title: str
    body: str

    class Meta:
        pattern = "posts/*.md"

        @staticmethod
        def before_parse(path: Path, content: bytes) -> bytes:
            # Remove "DRAFT: " prefix from all titles
            return content.replace(b"DRAFT: ", b"")
```

### `after_parse(path: Path, data: dict) -> dict`

Modify the parsed dictionary before type conversion. Useful for:
- Computing derived fields
- Migrating legacy field names
- Filtering or redacting sensitive data
- Resolving references between files
- Injecting metadata from file path

**Example: Compute word count**
```python
from pathlib import Path
from hyper import Singleton

class Article(Singleton):
    title: str
    body: str
    word_count: int

    class Meta:
        pattern = "article.md"

        @staticmethod
        def after_parse(path: Path, data: dict) -> dict:
            data['word_count'] = len(data.get('body', '').split())
            return data
```

**Example: Migrate legacy field names**
```python
from hyper import Singleton

class Config(Singleton):
    new_field: str
    current_setting: int

    class Meta:
        pattern = "config.json"

        @staticmethod
        def after_parse(path: Path, data: dict) -> dict:
            # Support old field names
            if 'old_field' in data:
                data['new_field'] = data.pop('old_field')
            if 'deprecated_setting' in data:
                data['current_setting'] = data.pop('deprecated_setting')
            return data
```

**Example: Extract metadata from file path**
```python
from hyper import Collection

class BlogPost(Collection):
    title: str
    body: str
    year: int
    month: int

    class Meta:
        pattern = "blog/**/**.md"

        @staticmethod
        def after_parse(path: Path, data: dict) -> dict:
            # Extract year/month from path like "blog/2024/01/post.md"
            parts = path.parts
            data['year'] = int(parts[-3])
            data['month'] = int(parts[-2])
            return data
```

### `after_load(instance: Self) -> Self`

Modify the final typed instance after conversion. Useful for:
- Computing derived properties
- Establishing relationships between objects
- Initializing computed fields
- Post-processing validated data
- Triggering side effects (logging, caching)

**Example: Compute URL from components**
```python
from hyper import Singleton

class ServerConfig(Singleton):
    host: str
    port: int
    protocol: str = "http"
    url: str = ""

    class Meta:
        pattern = "server.json"

        @staticmethod
        def after_load(instance: 'ServerConfig') -> 'ServerConfig':
            instance.url = f"{instance.protocol}://{instance.host}:{instance.port}"
            return instance
```

**Example: Normalize text fields**
```python
from hyper import Collection

class Product(Collection):
    name: str
    description: str
    name_normalized: str = ""

    class Meta:
        pattern = "products/*.json"

        @staticmethod
        def after_load(instance: 'Product') -> 'Product':
            instance.name_normalized = instance.name.lower().strip()
            return instance
```

**Example: With msgspec (immutable structs)**
```python
import msgspec
from hyper import Singleton

class Settings(Singleton, msgspec.Struct):
    theme: str
    version: int
    display_name: str = ""

    class Meta:
        pattern = "settings.json"

        @staticmethod
        def after_load(instance: 'Settings') -> 'Settings':
            # msgspec.Struct is immutable, use replace()
            return msgspec.structs.replace(
                instance,
                display_name=f"{instance.theme.upper()} v{instance.version}"
            )
```

## Combining Hooks

All three hooks can be used together. They execute in order: `before_parse` → `after_parse` → `after_load`.

```python
from pathlib import Path
from hyper import Singleton

class Data(Singleton):
    value: str
    count: int
    processed: str = ""

    class Meta:
        pattern = "data.json"

        @staticmethod
        def before_parse(path: Path, content: bytes) -> bytes:
            # Stage 1: Clean raw bytes
            return content.replace(b"OLD", b"NEW")

        @staticmethod
        def after_parse(path: Path, data: dict) -> dict:
            # Stage 2: Transform parsed data
            data['count'] = data['count'] * 2
            return data

        @staticmethod
        def after_load(instance: 'Data') -> 'Data':
            # Stage 3: Compute final fields
            instance.processed = f"{instance.value}-{instance.count}"
            return instance
```

## Collections vs Singletons

For **Collections**, hooks behave slightly differently:

- `before_parse`: Called **once per file** in the collection
- `after_parse`: Called **once per file** in the collection
- `after_load`: Called **once per item** in the final list

```python
from hyper import Collection

class Post(Collection):
    title: str
    views: int

    class Meta:
        pattern = "posts/*.json"

        @staticmethod
        def after_load(instance: 'Post') -> 'Post':
            # This runs for EACH post in the collection
            instance.title = instance.title.strip()
            return instance

# If you have 10 JSON files, after_load runs 10 times (once per post)
posts = Post.load()
```

## Use Cases

### Decryption/Decompression
```python
class Encrypted(Singleton):
    class Meta:
        pattern = "data.enc"

        @staticmethod
        def before_parse(path: Path, content: bytes) -> bytes:
            import gzip
            return gzip.decompress(content)
```

### Environment Variable Substitution
```python
import os

class Config(Singleton):
    class Meta:
        pattern = "config.json"

        @staticmethod
        def before_parse(path: Path, content: bytes) -> bytes:
            text = content.decode('utf-8')
            # Replace ${VAR} with environment variables
            for var in os.environ:
                text = text.replace(f"${{{var}}}", os.environ[var])
            return text.encode('utf-8')
```

### Reference Resolution
```python
from hyper import load

class Schema(Singleton):
    class Meta:
        pattern = "schema.json"

        @staticmethod
        def after_parse(path: Path, data: dict) -> dict:
            # Resolve $ref pointers
            if '$ref' in data:
                ref_path = path.parent / data['$ref']
                referenced = load(str(ref_path))
                data.update(referenced)
            return data
```

### Audit Logging
```python
import logging

class AuditedData(Singleton):
    class Meta:
        pattern = "data.json"

        @staticmethod
        def after_load(instance: 'AuditedData') -> 'AuditedData':
            logging.info(f"Loaded {instance.__class__.__name__}")
            return instance
```

## Important Notes

1. **Hooks are optional** - Models work fine without any hooks defined
2. **Type hints matter** - Hooks only run when loading with type hints (not raw `load()`)
3. **Return values required** - All hooks must return their modified input
4. **Path context** - All hooks receive the file `Path` for context-aware processing
5. **Order matters** - Hooks execute in a fixed order: before_parse → after_parse → after_load
6. **Collections** - For collections, `after_load` runs once per item in the final list
7. **Immutability** - For immutable types (msgspec.Struct), use `msgspec.structs.replace()` in `after_load`

## Performance Considerations

- **before_parse**: Minimal overhead, processes bytes before parsing
- **after_parse**: Runs before type validation, good for field transformations
- **after_load**: Runs after validation, safest for computed fields

The msgspec direct-parse optimization still works with hooks - if you're loading a single file into a `msgspec.Struct`, the parser will decode JSON directly to the struct, then `after_load` will run on the final instance.
