count: int = 5

# This will cause a ZeroDivisionError at render time
result = 10 / count

t"""<div>Result: {result}</div>"""
