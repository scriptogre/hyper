"""
Ultra-minimal counter with _hyperscript + server state

Shows:
- Server state (count)
- `{}` syntax for server calls
- Client-side animation with _hyperscript
"""

# Server state
count = 0

# Server handler
def increment():
    global count
    count += 1

def decrement():
    global count
    count -= 1

def reset():
    global count
    count = 0

# Template
t"""
<div class="counter">
    <h2>Count: {count}</h2>

    <div class="controls">
        <!-- Mixed: client animation + server call -->
        <button _="
            on click
                add .pulse to me then
                {decrement} then
                wait 100ms then
                remove .pulse from me
        ">−</button>

        <button _="on click {reset}">
            Reset
        </button>

        <button _="
            on click
                add .pulse to me then
                {increment} then
                wait 100ms then
                remove .pulse from me
        ">+</button>
    </div>
</div>

<style>
.pulse {
    animation: pulse 100ms ease;
}

@keyframes pulse {
    0% { transform: scale(1); }
    50% { transform: scale(1.1); }
    100% { transform: scale(1); }
}
</style>
"""
