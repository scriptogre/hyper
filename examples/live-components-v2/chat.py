"""
Live chat with shared state and broadcasting

Shows:
- Shared state across all connections
- Broadcasting to all clients
- Per-connection state (username)
- Lifecycle hooks (on_mount, on_unmount)
- System messages vs user messages
"""

from hyper import shared, broadcast
from datetime import datetime

# Shared state (across ALL connections)
messages = shared([])

# Per-connection state (injected from session/auth)
username: str

# Lifecycle hooks
def on_mount():
    """Called when user connects"""
    messages.append({
        "type": "system",
        "text": f"{username} joined",
        "time": datetime.now()
    })
    broadcast()  # Notify all connected clients

def on_unmount():
    """Called when user disconnects"""
    messages.append({
        "type": "system",
        "text": f"{username} left",
        "time": datetime.now()
    })
    broadcast()

# Event handlers
def send_message(text: str):
    if not text.strip():
        return

    messages.append({
        "type": "message",
        "user": username,
        "text": text.strip(),
        "time": datetime.now()
    })
    broadcast()  # Update all connected clients

# Template
t"""
<div class="chat-room">
    <div class="messages" id="messages">
        {% for msg in messages %}
        {% if msg.type == "system" %}
        <div class="system-message">
            <small>{msg.time.strftime("%H:%M")}</small>
            <span>{msg.text}</span>
        </div>
        {% else %}
        <div class="message {'own' if msg.user == username else ''}">
            <strong>{msg.user}:</strong>
            <span>{msg.text}</span>
            <small>{msg.time.strftime("%H:%M")}</small>
        </div>
        {% endif %}
        {% endfor %}
    </div>

    <form @submit.prevent="send_message(text)" class="message-input">
        <input
            name="text"
            placeholder="Type a message..."
            autofocus
            autocomplete="off"
        />
        <button type="submit">Send</button>
    </form>
</div>
"""
