--TEST--
a .phtml post is a plain view built from components: sections, callouts and code, ids in page order, the header cut off
--FILE--
<?php
require __DIR__ . '/setup.inc';
require_once __DIR__ . '/../Post_body.php';
$dir = fixture_dir('phtml');
file_put_contents("$dir/post.phtml", <<<'PHTML'
+++
path = "blog/x"
+++
<p>Intro.</p>
<?php open_component('section', ['title' => 'Result']) ?>
<?php open_component('callout', ['kind' => 'note', 'title' => 'Result']) ?>
<p>It worked: <?= math('x^2') ?></p>
<?php close_component() ?>
<?php open_component('code', ['language' => 'bash', 'title' => 'run.sh']) ?>
ssh root@<ip>
<?php close_component() ?>
<?php close_component() ?>
PHTML);

$post = ['format' => 'phtml', 'file' => "$dir/post.phtml"];
$html = Post_body::html($post);
var_dump(str_starts_with($html, '<p>Intro.</p>'));
var_dump(str_contains($html, '<section id="Result">'), str_contains($html, '<h2 id="Result-1">Result</h2>'));
var_dump(str_contains($html, '<div class="filename">run.sh</div>'), str_contains($html, 'root@&lt;ip&gt;'));
var_dump(str_contains($html, '<math'));
echo json_encode(Post_body::passages($post)), "\n";

try {
    component('nope');
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
[{"text":"Intro.","section":null},{"text":"It worked:","section":{"id":"Result-1","heading":"Result"}}]
there is no component "nope" (modules/components/nope/)
