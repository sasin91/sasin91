<?php
define('APPPATH', str_replace('\\', '/', dirname(__DIR__, 3)) . '/');
require_once APPPATH . 'packages/autoload.php';
// The prose helpers reach their module through Trongate, as in a page.
define('BASE_URL', 'http://localhost/');
require_once APPPATH . 'engine/Trongate.php';
require_once APPPATH . 'engine/Modules.php';
require_once __DIR__ . '/../Prose_view.php';
require_once __DIR__ . '/../Posts.php';
require_once __DIR__ . '/../Cv_data.php';
require_once __DIR__ . '/../Knowledge_article.php';

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
    file_put_contents("$dir/$slug.php", "+++\npath = \"$path\"\ntitle = \"$slug\"\ndate = $date\ndescription = \"d\"\n$extra+++\n<p>body</p>\n");
}

/** Renders view source written with the prose helpers, as a page would. */
function render_view(string $source): string {
    $file = fixture_dir('view') . '/view.php';
    file_put_contents($file, "+++\n+++\n" . $source);
    return Prose_view::html($file);
}
