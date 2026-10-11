--TEST--
heading ids are the ones the published pages were built with, so links into them keep working, and sections nest by level
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = render_view(<<<'VIEW'
<?= section_open('1. Create the server') ?>
<?= section_open('What <code>cv_match</code> does', 3) ?>
<?= section_close() ?>
<?= section_close() ?>
<?= section_open('Built on trongate.cloud') ?>
<?= section_close() ?>
<?= section_open('Built on trongate.cloud') ?>
<?= section_close() ?>
VIEW);
preg_match_all('/<section id="([^"]+)">\s*<(h\d)>/', $html, $ids, PREG_SET_ORDER);
foreach ($ids as [, $id, $tag]) {
    echo "$tag $id\n";
}
?>
--EXPECT--
h2 1-Create-the-server
h3 What-cv_match-does
h2 Built-on-trongatecloud
h2 Built-on-trongatecloud-1
