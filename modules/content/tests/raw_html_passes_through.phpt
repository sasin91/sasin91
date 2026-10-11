--TEST--
raw HTML passes through untouched
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $djot->render("``` =html\n<p class=\"colophon\">Measured.</p>\n```\n");
var_dump(str_contains($html, '<p class="colophon">Measured.</p>'));
?>
--EXPECT--
bool(true)
