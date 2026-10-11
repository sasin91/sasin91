--TEST--
a div written in the Markdown holds a callout, Markdown inside it renders, and its heading gets the id
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $markdown->render("## Before\n\nx\n\n<div class=\"warning\">\n\n## What went wrong\n\nIt *broke*.\n\n</div>\n\n## After\n");
var_dump(str_contains($html, '<div class="warning">'));
var_dump(str_contains($html, '<h2 id="What-went-wrong">What went wrong</h2>'));
var_dump(str_contains($html, '<p>It <em>broke</em>.</p>'));
// The callout sits inside the section before it, and the next heading
// opens a section of its own.
var_dump(preg_match('~<section id="Before">.*<div class="warning">.*</div>\s*</section>\s*<section id="After">~s', $html));
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
int(1)
