<?php
//The main config file
// The address this request came in on, so the same code answers on localhost,
// a preview address and sasin91.xyz. Keep it on one line: the host may rewrite it.
define('BASE_URL', (($_SERVER['HTTPS'] ?? '') === 'on' ? 'https' : 'http') . '://' . ($_SERVER['HTTP_HOST'] ?? 'localhost') . '/');
// 'dev' opens Trongate's developer tools (setup wizard, code generators, SQL
// runner, admin auto-login), so it must never answer the internet. Set APP_ENV
// to choose; without it only the command line and a browser on this machine
// get 'dev', and a deployed site is 'prod'.
$app_env = (static function (): string {
    $env = getenv('APP_ENV') ?: ($_SERVER['APP_ENV'] ?? '');
    if ($env !== '') {
        return $env;
    }
    if (PHP_SAPI === 'cli') {
        return 'dev';
    }
    $local = in_array($_SERVER['REMOTE_ADDR'] ?? '', ['127.0.0.1', '::1'], true);
    $forwarded = ($_SERVER['HTTP_X_FORWARDED_FOR'] ?? $_SERVER['HTTP_X_REAL_IP'] ?? $_SERVER['HTTP_FORWARDED'] ?? '') !== '';
    return $local && !$forwarded ? 'dev' : 'prod';
})();
// Keep this define on one line: the host rewrites config lines one at a time.
define('ENV', $app_env);
define('DEFAULT_MODULE', 'site');
define('DEFAULT_METHOD', 'index');
define('MODULE_ASSETS_TRIGGER', '_module');
define('ERROR_404', 'site/not_found');

// The site's public address, for canonical links, og:url, the feed and the
// sitemap. Fixed rather than taken from BASE_URL: wherever the app is served
// from (a trongate.cloud preview address, localhost), the page it describes
// is the one on sasin91.xyz.
define('SITE_URL', 'https://sasin91.xyz');
