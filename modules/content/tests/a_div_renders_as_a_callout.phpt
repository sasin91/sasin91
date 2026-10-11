--TEST--
a div renders as a callout, with its heading inside
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $djot->render("::: warning\n## What went wrong\n\nIt broke.\n:::\n");
var_dump(str_contains($html, '<div class="warning">'));
var_dump(str_contains($html, 'What went wrong'));
?>
--EXPECT--
bool(true)
bool(true)
