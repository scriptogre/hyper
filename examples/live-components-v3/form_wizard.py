"""
Multi-step form wizard with mixed client/server state

Shows:
- Client state: which step is visible (no need for server)
- Server state: form data and validation (needs persistence)
- Progressive disclosure
- Real-time validation
"""

from pydantic import BaseModel, EmailStr, field_validator

class SignupForm(BaseModel):
    email: EmailStr
    username: str
    password: str

    @field_validator('username')
    @classmethod
    def username_valid(cls, v: str) -> str:
        if len(v) < 3:
            raise ValueError('Username must be at least 3 characters')
        return v

    @field_validator('password')
    @classmethod
    def password_valid(cls, v: str) -> str:
        if len(v) < 8:
            raise ValueError('Password must be at least 8 characters')
        return v

# Server state (form data + validation)
email = ""
username = ""
password = ""
errors = {}
submitted = False

# Server handlers
def validate_email(value: str):
    global email, errors
    email = value
    errors.pop("email", None)

    try:
        SignupForm(email=email, username="test", password="password123")
    except Exception as e:
        if hasattr(e, "errors"):
            for error in e.errors():
                if error["loc"][0] == "email":
                    errors["email"] = error["msg"]

def validate_username(value: str):
    global username, errors
    username = value
    errors.pop("username", None)

    try:
        SignupForm(email="test@test.com", username=username, password="password123")
    except Exception as e:
        if hasattr(e, "errors"):
            for error in e.errors():
                if error["loc"][0] == "username":
                    errors["username"] = error["msg"]

def submit(pwd: str):
    global password, errors, submitted
    password = pwd
    errors = {}

    try:
        form = SignupForm(email=email, username=username, password=password)
        # Save to database
        submitted = True
    except Exception as e:
        if hasattr(e, "errors"):
            for error in e.errors():
                errors[error["loc"][0]] = error["msg"]

# Template
t"""
<div class="wizard">
    {% if submitted %}
    <div class="success" _="on load show me with *fade-in">
        <h2>✓ Welcome, {username}!</h2>
        <p>Check your email ({email}) to verify your account.</p>
    </div>
    {% else %}

    <!-- Step 1: Email (visible by default) -->
    <div class="step" id="step1">
        <h2>Step 1: Email</h2>

        <input
            name="email"
            type="email"
            value="{email}"
            placeholder="your@email.com"
            _="on input debounced at 300ms {validate_email(value)}"
        />

        {% if 'email' in errors %}
        <p class="error">{errors['email']}</p>
        {% endif %}

        <button
            _="on click
                if #email.value is not empty
                    hide #step1 with *fade-out then
                    show #step2 with *fade-in
                end"
        >
            Next →
        </button>
    </div>

    <!-- Step 2: Username (hidden by default) -->
    <div class="step" id="step2" style="display: none;">
        <h2>Step 2: Username</h2>

        <input
            name="username"
            value="{username}"
            placeholder="username"
            _="on input debounced at 300ms {validate_username(value)}"
        />

        {% if 'username' in errors %}
        <p class="error">{errors['username']}</p>
        {% endif %}

        <div class="buttons">
            <button
                _="on click
                    hide #step2 with *fade-out then
                    show #step1 with *fade-in"
            >
                ← Back
            </button>

            <button
                _="on click
                    if #username.value is not empty
                        hide #step2 with *fade-out then
                        show #step3 with *fade-in
                    end"
            >
                Next →
            </button>
        </div>
    </div>

    <!-- Step 3: Password (hidden by default) -->
    <div class="step" id="step3" style="display: none;">
        <h2>Step 3: Password</h2>

        <input
            name="password"
            type="password"
            placeholder="password"
        />

        {% if 'password' in errors %}
        <p class="error">{errors['password']}</p>
        {% endif %}

        <div class="buttons">
            <button
                _="on click
                    hide #step3 with *fade-out then
                    show #step2 with *fade-in"
            >
                ← Back
            </button>

            <button
                class="primary"
                _="on click
                    add .loading to me then
                    {submit(password)}"
            >
                Create Account
            </button>
        </div>
    </div>

    {% endif %}
</div>

<style>
.wizard {
    max-width: 400px;
    margin: 0 auto;
    padding: 24px;
}

.step {
    animation: fadeIn 300ms ease;
}

.step input {
    width: 100%;
    padding: 12px;
    margin: 8px 0;
    border: 1px solid #ddd;
    border-radius: 4px;
}

.error {
    color: #dc3545;
    font-size: 0.875rem;
    margin: 4px 0;
}

.buttons {
    display: flex;
    gap: 8px;
    margin-top: 16px;
}

.buttons button {
    flex: 1;
}

button.loading {
    opacity: 0.6;
    pointer-events: none;
}

button.loading::after {
    content: "...";
}

@keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes fade-out {
    from { opacity: 1; }
    to { opacity: 0; }
}
</style>
"""
