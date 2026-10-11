--TEST--
code containing PHP's own tags is given to code() as a string and shown, not run; a block without a title stays unframed
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = render_view(<<<'VIEW'
<?= code('php', <<<'CODE'
<?= anchor('account') ?>
CODE) ?>
VIEW);
var_dump(str_contains($html, 'codeblock'), str_contains($html, '&lt;?'), str_contains($html, 'anchor'));
?>
--EXPECT--
bool(false)
bool(true)
bool(true)
