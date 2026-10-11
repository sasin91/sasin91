--TEST--
a quote after an opening bracket opens, and an apostrophe after inline math closes
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $djot->render("a reason (\"Needs 3, has 2\").\n\nThe score of \$`i`'s query.\n");
var_dump(str_contains($html, "(\u{201C}Needs 3, has 2\u{201D})"));
var_dump(str_contains($html, "</math>\u{2019}s query"));
?>
--EXPECT--
bool(true)
bool(true)
