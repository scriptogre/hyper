from hyper import html


@html
def IfElse(*, is_admin: bool):

    yield "<nav>"

    if is_admin:
        yield """\
<a href="/admin">Admin</a>
    """
    else:
        yield """\
<a href="/account">Account</a>
    """

    yield "</nav>"
