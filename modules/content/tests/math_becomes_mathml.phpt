--TEST--
math becomes MathML in the page rather than markup a script typesets later
--FILE--
<?php
require __DIR__ . '/setup.inc';
$inline = $djot->render("Epley is \$`e = w(1 + r/30)`.\n");
var_dump(str_contains($inline, '<math'), str_contains($inline, '\('));
$display = $djot->render("\$\$`e = w(1 + r/30)`\n");
var_dump(str_contains($display, 'display="block"'));
?>
--EXPECT--
bool(true)
bool(false)
bool(true)
