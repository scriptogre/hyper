"""Django integration for Hyper.

Add this app to your INSTALLED_APPS:

    INSTALLED_APPS = [
        ...,
        "hyper.integrations.django",
    ]

Wire the template tags and context processor into the DTE backend so any
template can use ``{% hyper Component arg=value %}`` without ``{% load %}``:

    TEMPLATES = [{
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "OPTIONS": {
            "context_processors": [
                ...,
                "hyper.integrations.django.context_processors.components",
            ],
            "builtins": [
                "hyper.integrations.django.templatetags.hyper",
            ],
        },
    }]

The Django app's ``ready()`` walks every installed app for a ``components/``
subdirectory and imports the compiled ``.py`` siblings of ``.hyper`` files.
Discovered components are exposed to every DTE template via the context
processor.
"""

default_app_config = "hyper.integrations.django.apps.HyperConfig"
