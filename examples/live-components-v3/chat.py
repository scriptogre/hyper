"""
Real-time chat with shared state and server push

Shows:
- Shared state across all connections
- Server-initiated updates (broadcast)
- Lifecycle hooks
- Mixed client/server interaction
"""

from hyper import shared, broadcast
from datetime import datetime

# Shared state (across ALL connections)
messages = shared([])

# Per-connection state (from session/auth)
username: str

# Lifecycle hooks
def on_mount():
    """Called when user connects"""
    messages.append({
        "type": "system",
        "text": f"{username} joined",
        "time": datetime.now()
    })
    broadcast()

def on_unmount():
    """Called when user disconnects"""
    messages.append({
        "type": "system",
        "text": f"{username} left",
        "time": datetime.now()
    })
    broadcast()

# Handlers
def send_message(text: str):
    if not text.strip():
        return

    messages.append({
        "type": "message",
        "user": username,
        "text": text.strip(),
        "time": datetime.now()
    })
    broadcast()  # Push to all connected clients

# Template
t"""
<div class="chat-room">
    <div
        class="messages"
        id="messages"
        _="on update from server
            wait 10ms then
            set my scrollTop to my scrollHeight"
    >
        {% for msg in messages %}
        {% if msg.type == "system" %}
        <div class="system-message" _="on load show me with *fade-in">
            <small>{msg.time.strftime("%H:%M")}</small>
            <span>{msg.text}</span>
        </div>
        {% else %}
        <div
            class="message {'own' if msg.user == username else ''}"
            _="on load show me with *slide-in"
        >
            <strong>{msg.user}:</strong>
            <span>{msg.text}</span>
            <small>{msg.time.strftime("%H:%M")}</small>
        </div>
        {% endif %}
        {% endfor %}
    </div>

    <form class="message-input">
        <input
            name="text"
            placeholder="Type a message..."
            autofocus
            autocomplete="off"
        />

        <button _="
            on click
                {send_message(text)} then
                set the previous <input/>'s value to '' then
                focus() the previous <input/>
        ">
            Send
        </button>
    </form>
</div>

<style>
.chat-room {
    display: flex;
    flex-direction: column;
    height: 500px;
    border: 1px solid #ddd;
    border-radius: 8px;
}

.messages {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    scroll-behavior: smooth;
}

.system-message {
    text-align: center;
    color: #999;
    font-size: 0.875rem;
    margin: 8px 0;
}

.message {
    margin: 8px 0;
    padding: 8px 12px;
    background: #f5f5f5;
    border-radius: 8px;
    max-width: 70%;
}

.message.own {
    margin-left: auto;
    background: #007bff;
    color: white;
}

.message-input {
    display: flex;
    gap: 8px;
    padding: 16px;
    border-top: 1px solid #ddd;
}

.message-input input {
    flex: 1;
}

@keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes slide-in {
    from {
        opacity: 0;
        transform: translateY(4px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>
"""
