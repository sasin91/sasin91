--TEST--
heading ids are the ones the published pages were built with, so links into them keep working
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $markdown->render("## 1. Create the server\n\nx\n\n## What `cv_match` does\n\nx\n\n## Built on trongate.cloud\n\nx\n\n## Built on trongate.cloud\n\nx\n");
preg_match_all('/<section id="([^"]+)"/', $html, $ids);
echo implode("\n", $ids[1]), "\n";
?>
--EXPECT--
1-Create-the-server
What-cv_match-does
Built-on-trongatecloud
Built-on-trongatecloud-1
