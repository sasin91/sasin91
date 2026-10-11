--TEST--
a code block with a title gets a file name bar, one without stays unframed, and a title is escaped
--FILE--
<?php
require __DIR__ . '/setup.inc';
$titled = $djot->render("{title=\"4-remaster.sh\"}\n```bash\necho hi\n```\n");
var_dump(str_contains($titled, '<div class="codeblock"><div class="filename">4-remaster.sh</div><pre class="hl-code">'));

$untitled = $djot->render("```bash\necho hi\n```\n");
var_dump(str_contains($untitled, 'codeblock'), str_contains($untitled, '<pre class="hl-code"><code>'));

$unknown = $djot->render("```caddy\nsite { }\n```\n");
var_dump(str_contains($unknown, '<pre class="hl-code"><code>site { }'));

$hostile = $djot->render("{title=\"<script>\"}\n```bash\nx\n```\n");
var_dump(str_contains($hostile, '<div class="filename"><script>'));
?>
--EXPECT--
bool(true)
bool(false)
bool(true)
bool(true)
bool(false)
