## Background Tasks

Queue tasks to run after the response.

```python
# app/pages/contact.py
from hyper import BackgroundTasks, POST
from typing import Annotated

background_tasks: BackgroundTasks

if POST:
    email: Annotated[str, Form()]
    message: Annotated[str, Form()]
    
    background_tasks.add_task(send_email, email, message)
    
    t"""<h1>Message sent!</h1>"""
```

Response returns immediately. Email sends in background.

***

## Add Tasks

Use `add_task()` to queue function calls.

```python
background_tasks.add_task(function_name, arg1, arg2)
```

Tasks run after response sent to client.

***

## Multiple Tasks

Queue multiple tasks in sequence.

```python
if POST:
    background_tasks.add_task(send_email, email, message)
    background_tasks.add_task(log_submission, email)
    background_tasks.add_task(update_analytics, "contact_form")
    
    t"""<h1>Done!</h1>"""
```

Tasks execute in order added.

***

## Async Tasks

Supports both sync and async functions.

```python
async def send_webhook(url: str, data: dict):
    async with httpx.AsyncClient() as client:
        await client.post(url, json=data)

background_tasks.add_task(send_webhook, webhook_url, payload)
```

Hyper handles sync/async automatically.

***

## Pluggable Backends

TODO: Expand on pluggable queue implementations.

Default runs in-process like FastAPI. Configure alternative backends:

```python
# config.py
app = Hyper(
    background_queue=RedisQueue(redis_url="...")
    # or NATSQueue, RabbitMQQueue, etc.
)
```

Same `background_tasks` API works with any backend. Swap implementations without changing page code.

Advanced features (retries, priorities, delays) coming soon.

***

**Key Points:**
- Inject `background_tasks: BackgroundTasks`
- Call `add_task(func, *args)`
- Response returns immediately
- Tasks run after response sent
- Pluggable backends supported
