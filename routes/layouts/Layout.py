from hyperprose import Slot

content: Slot
title: Slot

t"""
<!doctype html>
<html lang="en">
<head>
    <title>{title}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover" />

    <!-- Favicon -->
    <link rel="icon" type="image/png" href="" sizes="96x96" />
    <link rel="icon" type="image/svg+xml" href="" />
    <link rel="shortcut icon" href="" />
    <link rel="apple-touch-icon" sizes="180x180" href="" />
    <meta name="apple-mobile-web-app-title" content="Roast Roulette" />
</head>
<body>
<main>
    {content}
</main>
</body>
</html>
"""