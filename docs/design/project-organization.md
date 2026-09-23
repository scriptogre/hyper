# Project organization

Start with one entrypoint:

```text
templates/
└── index.hyper
```

Add a shared page shell beside it when needed:

```text
templates/
├── Base.hyper
└── index.hyper
```

Group new features by domain:

```text
templates/
├── Base.hyper
├── index.hyper
└── chats/
    ├── chat.hyper
    └── chat_list.hyper
```

Lowercase files are entrypoints called by routes. PascalCase names are components injected into templates.

Keep small components in their entrypoint file:

```hyper
from hyper import Component, subcomponent

message: Message
---
@subcomponent
def ChatMessage(*, message: Message) -> Component:
    <article>{message.text}</article>

<{ChatMessage} message={message} />
```

The route can also render the exported component directly:

```python
chat.ChatMessage(message=message).render()
```

When an inline component outgrows its entrypoint, extract it beside the page:

```text
templates/chats/
├── chat.hyper
├── chat_list.hyper
└── ChatMessage.hyper
```

When several small components belong together, group them in a domain-local library instead:

```text
templates/chats/
├── chat.hyper
├── chat_list.hyper
└── components.hyper
```
