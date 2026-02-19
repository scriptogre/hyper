from hyper import html


@html
def SelfClosing():
    yield """<br /><hr /><img src="photo.jpg" alt="Photo" /><input type="text" name="field" /><div /><span />"""
