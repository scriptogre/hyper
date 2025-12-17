from playground.components import Button, Card

t"""
<div class="page">
    <h1>Welcome to the Index Page</h1>

    <{Card} title="Getting Started" color="blue" hx-get="/test">
        <p>This is a card with content</p>
        <{Button} variant="success">Get Started</{Button}>
    </{Card}>
</div>
"""