"""
Live form with real-time validation using Pydantic

Shows:
- Per-field validation
- Pydantic model validation
- Error handling
- Debounced input validation
- Success state
"""

from pydantic import BaseModel, EmailStr, field_validator
from typing import Optional

class UserForm(BaseModel):
    """User registration form model"""
    username: str
    email: EmailStr
    age: int

    @field_validator('username')
    @classmethod
    def username_valid(cls, v: str) -> str:
        if len(v) < 3:
            raise ValueError('Username must be at least 3 characters')
        if not v.isalnum():
            raise ValueError('Username must be alphanumeric')
        return v

    @field_validator('age')
    @classmethod
    def age_valid(cls, v: int) -> int:
        if v < 13:
            raise ValueError('Must be at least 13 years old')
        if v > 120:
            raise ValueError('Invalid age')
        return v

# Form state
username = ""
email = ""
age: Optional[int] = None

# Error state
errors: dict[str, str] = {}
submitted = False

# Handlers
def validate_field(field: str, value: str):
    """Validate a single field as user types"""
    global username, email, age, errors

    # Update field value
    if field == "username":
        username = value
    elif field == "email":
        email = value
    elif field == "age":
        try:
            age = int(value) if value else None
        except ValueError:
            errors["age"] = "Must be a number"
            return

    # Clear previous error for this field
    errors.pop(field, None)

    # Validate using Pydantic (partial validation)
    try:
        # Create minimal valid model to test this field
        test_data = {
            "username": username if field == "username" else "test",
            "email": email if field == "email" else "test@example.com",
            "age": age if field == "age" else 18
        }

        # This will raise if field is invalid
        UserForm.model_validate(test_data)

    except Exception as e:
        if hasattr(e, "errors"):
            for error in e.errors():
                if error["loc"][0] == field:
                    errors[field] = error["msg"]

def submit():
    """Submit the form (validate all fields)"""
    global submitted, errors

    errors = {}  # Clear all errors

    try:
        # Validate entire form
        user = UserForm(
            username=username,
            email=email,
            age=age or 0
        )

        # Success!
        submitted = True
        # In real app: save to database

    except Exception as e:
        # Collect all validation errors
        if hasattr(e, "errors"):
            for error in e.errors():
                field = error["loc"][0]
                errors[field] = error["msg"]

# Computed: form is valid
is_valid = username and email and age and not errors

# Template
t"""
<div class="signup-form">
    {% if submitted %}
    <div class="success">
        <h2>✓ Welcome, {username}!</h2>
        <p>Check your email ({email}) to verify your account.</p>
    </div>
    {% else %}
    <form @submit.prevent="submit">
        <h2>Create Account</h2>

        <div class="field {'error' if 'username' in errors else ''}">
            <label for="username">Username</label>
            <input
                id="username"
                name="username"
                value="{username}"
                @input.debounce.300="validate_field('username', value)"
                required
            />
            {% if 'username' in errors %}
            <span class="error-message">{errors['username']}</span>
            {% endif %}
        </div>

        <div class="field {'error' if 'email' in errors else ''}">
            <label for="email">Email</label>
            <input
                id="email"
                name="email"
                type="email"
                value="{email}"
                @input.debounce.300="validate_field('email', value)"
                required
            />
            {% if 'email' in errors %}
            <span class="error-message">{errors['email']}</span>
            {% endif %}
        </div>

        <div class="field {'error' if 'age' in errors else ''}">
            <label for="age">Age</label>
            <input
                id="age"
                name="age"
                type="number"
                value="{age or ''}"
                @input.debounce.300="validate_field('age', value)"
                required
            />
            {% if 'age' in errors %}
            <span class="error-message">{errors['age']}</span>
            {% endif %}
        </div>

        <button
            type="submit"
            disabled={not is_valid}
        >
            Create Account
        </button>
    </form>
    {% endif %}
</div>
"""
