# Form Handling

> **🔮 SSR Mode** - This feature requires server-side rendering (planned). Forms are not available in SSG mode, which generates static HTML at build time.

Hyper uses FastAPI-style dependency injection for form handling. Use `Form()` with type hints to automatically extract form fields.

---

## Simple Form Handling

```python
# routes/contact.py
from typing import Annotated
from hyper import GET, POST, Form
from layouts import Layout

if GET:
    # Show the form
    t"""
    <{Layout} title="Contact Us">
        <h1>Contact Us</h1>
        <form method="POST" action="/contact">
            <div>
                <label for="name">Name:</label>
                <input type="text" id="name" name="name" required>
            </div>
            <div>
                <label for="email">Email:</label>
                <input type="email" id="email" name="email" required>
            </div>
            <div>
                <label for="message">Message:</label>
                <textarea id="message" name="message" required></textarea>
            </div>
            <button type="submit">Send Message</button>
        </form>
    </{Layout}>
    """

elif POST:
    # Form fields are automatically injected
    name: Annotated[str, Form()]
    email: Annotated[str, Form()]
    message: Annotated[str, Form()]

    send_contact_email(name, email, message)

    t"""
    <{Layout} title="Contact - Success">
        <div class="alert alert-success">
            <h2>Thank you, {name}!</h2>
            <p>We've received your message and will get back to you soon.</p>
        </div>
        <a href="/">Back to Home</a>
    </{Layout}>
    """
```

**Key points:**
- Use `GET` and `POST` helpers to differentiate request methods
- `Form()` fields only injected when `POST` is True
- Field names match form input `name` attributes
- Type conversion happens automatically

---

## Form Validation

Add validation to form fields:

```python
# routes/signup.py
from typing import Annotated
from hyper import GET, POST, Form
from layouts import Base

if GET:
    t"""
    <{Base} title="Sign Up">
        <form method="POST">
            <input name="username" required>
            <input name="email" type="email" required>
            <input name="age" type="number">
            <button>Sign Up</button>
        </form>
    </{Base}>
    """

elif POST:
    # With validation
    username: Annotated[str, Form(min_length=3, max_length=20)]
    email: Annotated[str, Form()]
    age: Annotated[int, Form(ge=13, le=120)]

    # Validation errors are automatically returned as 422 responses
    user = create_user(username, email, age)

    t"""
    <{Base} title="Welcome">
        <h1>Welcome, {user.username}!</h1>
    </{Base}>
    """
```

**Validation options:**
- `min_length`, `max_length` - String length
- `ge`, `le`, `gt`, `lt` - Numeric constraints
- `regex` - Pattern matching

---

## Form Models

Use Pydantic models for structured form handling:

```python
# routes/signup.py
from typing import Annotated
from pydantic import BaseModel, EmailStr, Field
from hyper import GET, POST, Form
from layouts import Base

class SignupForm(BaseModel):
    username: str = Field(min_length=3, max_length=20)
    email: EmailStr
    password: str = Field(min_length=8)
    age: int = Field(ge=13, le=120)

if GET:
    t"""
    <{Base} title="Sign Up">
        <form method="POST">
            <input name="username" required>
            <input name="email" type="email" required>
            <input name="password" type="password" required>
            <input name="age" type="number" required>
            <button>Sign Up</button>
        </form>
    </{Base}>
    """

elif POST:
    # Entire form parsed into model with validation
    form: Annotated[SignupForm, Form()]

    user = create_user(
        username=form.username,
        email=form.email,
        password=hash_password(form.password),
        age=form.age
    )

    t"""
    <{Base} title="Welcome">
        <h1>Welcome, {user.username}!</h1>
    </{Base}>
    """
```

**Benefits:**
- Centralized validation logic
- Type safety
- Reusable across routes
- Clear error messages

---

## HTMX Form with Validation

```python
# routes/users/create.py
from typing import Annotated
from hyper import GET, POST, Form
from components import AlertBox

if GET:
    # Show form
    t"""
    <form hx-post="/users/create" hx-target="#form-response" hx-swap="innerHTML">
        <div>
            <label>Name: <input type="text" name="name" required></label>
        </div>
        <div>
            <label>Email: <input type="email" name="email" required></label>
        </div>
        <button type="submit">Create User</button>
        <div id="form-response"></div>
    </form>
    """

elif POST:
    # Form fields are automatically injected
    name: Annotated[str, Form()]
    email: Annotated[str, Form()]

    # Validate
    errors = []
    if not name:
        errors.append("Name is required")
    if not email or "@" not in email:
        errors.append("Valid email is required")

    if errors:
        # Return errors for HTMX swap
        t"""
        <div id="form-errors">
            {[t'<{AlertBox} message={err} type="error" />' for err in errors]}
        </div>
        """
    else:
        # Success
        user = create_user(name, email)
        t"""
        <div id="form-success">
            <{AlertBox} message="User created successfully!" type="success" />
            <div class="user-card">
                <h3>{user.name}</h3>
                <p>{user.email}</p>
            </div>
        </div>
        """
```

---

## Form with File Upload

```python
# routes/profile/avatar.py
from typing import Annotated
from hyper import GET, POST, Form, File, UploadFile
from layouts import Base

user = get_current_user()

if GET:
    t"""
    <{Base} title="Update Avatar">
        <form method="POST" enctype="multipart/form-data">
            <input type="file" name="avatar" accept="image/*" required>
            <input type="text" name="alt_text" placeholder="Avatar description">
            <button>Upload</button>
        </form>
    </{Base}>
    """

elif POST:
    # File and form data together
    avatar: Annotated[UploadFile, File()]
    alt_text: Annotated[str, Form()] = ""

    # Save file
    contents = await avatar.read()
    avatar_url = save_avatar(user.id, avatar.filename, contents)

    # Update user
    update_user(user.id, avatar_url=avatar_url, avatar_alt=alt_text)

    t"""
    <{Base} title="Avatar Updated">
        <h1>Avatar Updated!</h1>
        <img src="{avatar_url}" alt="{alt_text}">
    </{Base}>
    """
```

---

## Optional Form Fields

Use `None` as default for optional fields:

```python
# routes/profile/update.py
from typing import Annotated
from hyper import POST, Form

if POST:
    # Required field
    name: Annotated[str, Form()]

    # Optional fields (default to None if not provided)
    bio: Annotated[str | None, Form()] = None
    website: Annotated[str | None, Form()] = None
    age: Annotated[int | None, Form()] = None

    update_user(
        name=name,
        bio=bio,
        website=website,
        age=age
    )
```

---

## Form Lists

Handle multiple values for the same field:

```python
# routes/preferences.py
from typing import Annotated
from hyper import GET, POST, Form

if GET:
    t"""
    <form method="POST">
        <label><input type="checkbox" name="interests" value="coding"> Coding</label>
        <label><input type="checkbox" name="interests" value="design"> Design</label>
        <label><input type="checkbox" name="interests" value="writing"> Writing</label>
        <button>Save</button>
    </form>
    """

elif POST:
    # Multiple values from checkboxes
    interests: Annotated[list[str], Form()] = []

    save_user_interests(interests)

    t"""<h1>Saved {len(interests)} interests</h1>"""
```

---

## Custom Error Handling

Handle validation errors gracefully:

```python
# routes/signup.py
from typing import Annotated
from pydantic import BaseModel, EmailStr, ValidationError
from hyper import POST, Form

class SignupForm(BaseModel):
    username: str
    email: EmailStr
    password: str

if POST:
    try:
        form: Annotated[SignupForm, Form()]
        user = create_user(form.username, form.email, form.password)
        t"""<h1>Welcome, {user.username}!</h1>"""
    except ValidationError as e:
        # Custom error display
        errors = e.errors()
        t"""
        <div class="errors">
            <h2>Please fix the following errors:</h2>
            <ul>
                {[t'<li>{err["loc"][0]}: {err["msg"]}</li>' for err in errors]}
            </ul>
        </div>
        """
```

---

## Key Points

- **Use `GET` and `POST` helpers to differentiate request methods**
- **Use `Form()` with `Annotated` for form fields**
- **Fields injected only when `POST` is True**
- **Field names match form input `name` attributes**
- **Use Pydantic models for complex forms**
- **Validation happens automatically**
- **Works seamlessly with HTMX**
- **Combine with `File()` for uploads**
- **Use `| None` with defaults for optional fields**

---

## Related

- **[Dependency Injection](dependency-injection.md)** - Full injection patterns
- **FastAPI [Form Data](https://fastapi.tiangolo.com/tutorial/request-forms/) docs**
- **FastAPI [Form Models](https://fastapi.tiangolo.com/tutorial/request-form-models/) docs**
- **FastAPI [Request Files](https://fastapi.tiangolo.com/tutorial/request-files/) docs**

---

**[← Previous: Dependency Injection](dependency-injection.md)** | **[Next: Streaming →](streaming.md)**
