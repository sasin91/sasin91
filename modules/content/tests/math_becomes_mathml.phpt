--TEST--
math between dollar signs becomes MathML in the page, and a dollar sign in code is left alone
--FILE--
<?php
require __DIR__ . '/setup.inc';
$inline = $markdown->render("Epley is \$e = w(1 + r/30)\$.\n");
var_dump(str_contains($inline, '<math'), str_contains($inline, 'display="inline"'), str_contains($inline, '$'));
$display = $markdown->render("\$\$e = w(1 + r/30)\$\$\n");
var_dump(str_contains($display, 'display="block"'));
$code = $markdown->render("Run `echo \$HOME and \$PATH`.\n");
var_dump(str_contains($code, '<code>echo $HOME and $PATH</code>'));
?>
--EXPECT--
bool(true)
bool(true)
bool(false)
bool(true)
bool(true)
