from hyper import html, escape, render_attr, spread_attrs


@html
def AllAttributeKinds(*, url: str, disabled: bool, props: dict):
    yield f"""\
<a href="/page" class="link-{escape(url)}" data-url="{escape(url)}"{render_attr("disabled", disabled)}{spread_attrs(props)} target="_blank">
    Link
</a>"""
