--TEST--
straight quotes curl and double hyphens become dashes, as the published pages had them
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $markdown->render("a reason (\"Needs 3, has 2\"), it's -- roughly --- done...\n");
echo strip_tags($html);
?>
--EXPECT--
a reason (“Needs 3, has 2”), it’s – roughly — done…
