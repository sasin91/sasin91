--TEST--
a post is a plain view built with the prose helpers: sections, callouts and code, ids in page order, its header cut off
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = render_view(<<<'VIEW'
<p>Intro.</p>
<?= section_open('Result') ?>
<?= callout_open('note', 'Result') ?>
<p>It worked: <?= math('x^2') ?></p>
<?= callout_close() ?>
<?= code_open('bash', 'run.sh') ?>
ssh root@<ip>
<?= code_close() ?>
<?= section_close() ?>
VIEW);
var_dump(str_starts_with($html, '<p>Intro.</p>'));
var_dump(str_contains($html, '<section id="Result">'), str_contains($html, '<h2 id="Result-1">Result</h2>'));
var_dump(str_contains($html, '<div class="codeblock"><div class="filename">run.sh</div><pre class="hl-code">'), str_contains($html, 'root@&lt;ip&gt;'));
var_dump(str_contains($html, '<math'));

try {
    callout_open('tip');
} catch (InvalidArgumentException $error) {
    echo $error->getMessage(), "\n";
}
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
a callout is a note or a warning, not "tip"
