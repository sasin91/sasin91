<?php
/**
 * For `php -S 127.0.0.1:8000 -t public bin/dev-router.php`: files under
 * public/ are served as they are, everything else goes to Trongate, as
 * public/.htaccess does under Apache.
 */
$path = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);
if ($path !== '/' && is_file($_SERVER['DOCUMENT_ROOT'] . $path)) {
    return false;
}
chdir($_SERVER['DOCUMENT_ROOT']);
require $_SERVER['DOCUMENT_ROOT'] . '/index.php';
