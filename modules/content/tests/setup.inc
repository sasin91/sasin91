<?php
define('APPPATH', str_replace('\\', '/', dirname(__DIR__, 3)) . '/');
require_once APPPATH . 'packages/autoload.php';
require_once __DIR__ . '/../Djot_html.php';
require_once __DIR__ . '/../Posts.php';
require_once __DIR__ . '/../Cv_data.php';
require_once __DIR__ . '/../Knowledge_article.php';

$djot = new Djot_html();

/** A directory of its own under the system temp dir, emptied first. */
function fixture_dir(string $name): string {
    $dir = str_replace('\\', '/', sys_get_temp_dir()) . '/sasin91-test-' . $name . '-' . getmypid();
    if (is_dir($dir)) {
        $walk = new RecursiveIteratorIterator(new RecursiveDirectoryIterator($dir, FilesystemIterator::SKIP_DOTS), RecursiveIteratorIterator::CHILD_FIRST);
        foreach ($walk as $entry) {
            $entry->isDir() ? rmdir($entry->getPathname()) : unlink($entry->getPathname());
        }
    }
    @mkdir($dir, 0777, true);
    return $dir;
}

function write_post(string $dir, string $slug, string $path, string $date, string $extra = ''): void {
    file_put_contents("$dir/$slug.dj", "+++\npath = \"$path\"\ntitle = \"$slug\"\ndate = $date\ndescription = \"d\"\n$extra+++\nbody\n");
}
