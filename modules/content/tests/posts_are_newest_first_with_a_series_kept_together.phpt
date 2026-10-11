--TEST--
posts are newest first, ties break on path, nested paths survive, and a series reads in part order at its newest date
--FILE--
<?php
require __DIR__ . '/setup.inc';
$dir = fixture_dir('posts');
write_post($dir, 'later', 'blog/later', '2026-08-09');
write_post($dir, 'three', 'blog/three', '2026-08-04', "series = \"s\"\npart = 3\n");
write_post($dir, 'one', 'blog/one', '2026-08-01', "series = \"s\"\npart = 1\n");
write_post($dir, 'two', 'blog/two', '2026-08-02', "series = \"s\"\npart = 2\n");
write_post($dir, 'b-post', 'blog/b-post', '2026-07-26');
write_post($dir, 'a-post', 'blog/a-post', '2026-07-26');
write_post($dir, 'nested', 'blog/trongate/mx-transition', '2025-03-03');
file_put_contents("$dir/ignored.md", 'not a post');

$posts = (new Posts($dir))->all();
foreach ($posts as $post) {
    echo $post['url'], ' ', $post['date_long'], ' ', $post['date_rfc2822'], "\n";
}
?>
--EXPECT--
/blog/later/ August 9, 2026 Sun, 9 Aug 2026 00:00:00 +0000
/blog/one/ August 1, 2026 Sat, 1 Aug 2026 00:00:00 +0000
/blog/two/ August 2, 2026 Sun, 2 Aug 2026 00:00:00 +0000
/blog/three/ August 4, 2026 Tue, 4 Aug 2026 00:00:00 +0000
/blog/a-post/ July 26, 2026 Sun, 26 Jul 2026 00:00:00 +0000
/blog/b-post/ July 26, 2026 Sun, 26 Jul 2026 00:00:00 +0000
/blog/trongate/mx-transition/ March 3, 2025 Mon, 3 Mar 2025 00:00:00 +0000
