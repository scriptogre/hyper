count: int = 5

# This will cause a ZeroDivisionError at render time
result = 10 / count

t"""
<!--@ fragment {'image'} -->
<img src="/test.png" {'fragment'} />l
<!--@ end -->

<div>Result: {result}</div>
"""
