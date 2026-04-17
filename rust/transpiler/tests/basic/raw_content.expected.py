from hyper import html, escape


@html
def RawContent(*, theme_color: str = "red"):

    # Style tags are raw - CSS braces are literal
    yield """\
<style>
    .card {
        background: white;
        border-radius: 8px;
    }
    .card:hover {
        transform: scale(1.05);
    }
    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
    @media (max-width: 768px) {
        .card { padding: 1rem; }
    }
</style>"""

    # Script tags are raw - JS braces are literal
    yield """\
<script>
    const data = { name: "test", count: 0 };
    if (data.count > 0) {
        console.log(data.name);
    }
    for (let i = 0; i < 10; i++) {
        data.count++;
    }
</script>"""

    # Style with attributes
    yield """\
<style type="text/tailwindcss">
    @theme {
        --color-accent: #FE750F;
    }
</style>"""

    # Explicit raw block
    yield """\
<div>
    
        @decorator
        if something:
            for x in y:
                {not_an_expression}
            end
        end
    
</div>"""

    # Content after raw resumes normal parsing
    yield f"""<p>{escape(theme_color)}</p>"""
