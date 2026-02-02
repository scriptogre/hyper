from hyper import component


@component
def DeeplyNested():
    yield """<div class="level-1"><div class="level-2"><div class="level-3"><div class="level-4"><div class="level-5"><span>Deep content</span></div></div></div></div></div>"""
