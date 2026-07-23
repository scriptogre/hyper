"""Django integration for Hyper.

Add this app to your INSTALLED_APPS:

    INSTALLED_APPS = [
        ...,
        "hyperhtml.integrations.django",
    ]

Wire the template tags and context processor into the DTE backend so any
template can use ``{% hyper Component arg=value %}`` without ``{% load %}``:

    TEMPLATES = [{
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "OPTIONS": {
            "context_processors": [
                ...,
                "hyperhtml.integrations.django.context_processors.components",
            ],
            "builtins": [
                "hyperhtml.integrations.django.templatetags.hyper",
            ],
        },
    }]

At startup the app looks wherever Django looks for templates (every ``DIRS``
entry, plus each app's ``templates/`` when ``APP_DIRS`` is on) and imports the
compiled ``.py`` sibling of every ``.hyper`` file. The context processor then
hands those components to every template.
"""
