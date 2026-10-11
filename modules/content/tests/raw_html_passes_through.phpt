--TEST--
raw HTML passes through untouched
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $markdown->render("<p class=\"colophon\">Measured.</p>\n");
var_dump(str_contains($html, '<p class="colophon">Measured.</p>'));
?>
--EXPECT--
bool(true)
