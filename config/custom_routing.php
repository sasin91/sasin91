<?php
// Every page answers at the same URL the static site used, with or without
// its trailing slash. The first matching pattern wins, so the AI article's
// pages come before the catch-all for posts. There is no tg-admin route: the
// site has no database, so no admin panel.
$routes = [
    'about/?' => 'site/about',
    'rss\.xml' => 'site/rss',
    'sitemap\.xml' => 'site/sitemap',
    'robots\.txt' => 'site/robots',

    'cv/?' => 'cv/index',
    'cv_jonas_hansen_software_developer\.pdf' => 'cv/pdf',

    'blog/ai/?' => 'knowledge/read',
    'blog/ai/timeline/?' => 'knowledge/timeline',
    'blog/ai/ask/?' => 'knowledge/ask',
    'blog/?' => 'blog/index',
    'blog/(:all)' => 'blog/show/$1',

    'search/index\.json' => 'search/index_json',
    'search/palette\.js' => 'search/palette_js',
    'search/ask\.js' => 'search/ask_js',
    'search/model\.bin' => 'search/model_bin',
    'search/vocab\.txt' => 'search/vocab_txt',
];
define('CUSTOM_ROUTES', $routes);
