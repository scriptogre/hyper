"""
Ultra-minimal live counter component

Just module-level state + functions + top-level t-string.
Files in app/live/ are automatically stateful.
"""

# State: module-level variable
count = 0

# Handlers: module-level functions
def increment():
    global count
    count += 1

def decrement():
    global count
    count -= 1

def reset():
    global count
    count = 0

# Template: top-level t-string (unchanged from regular components!)
t"""
<div class="counter">
    <h2>Count: {count}</h2>
    <div class="controls">
        <button @click="decrement">−</button>
        <button @click="reset">Reset</button>
        <button @click="increment">+</button>
    </div>
</div>
"""
