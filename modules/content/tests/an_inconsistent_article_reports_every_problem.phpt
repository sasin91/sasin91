--TEST--
an inconsistent article reports every problem at once
--FILE--
<?php
require __DIR__ . '/setup.inc';
$dir = fixture_dir('article');
mkdir("$dir/views/nodes", 0777, true);
file_put_contents("$dir/article.toml", "path = \"blog/ai\"\ntitle = \"t\"\ndescription = \"d\"\npublished = 2026-10-02\nupdated = 2026-10-02\n\n[[part]]\ntitle = \"P\"\nnodes = [\"a\", \"b\", \"ghost\"]\n");
file_put_contents("$dir/views/nodes/a.php", "+++\nid = \"a\"\ntitle = \"A\"\nsummary = \"s\"\nconcepts = [\"c\"]\nprerequisites = [\"b\"]\n+++\n<p>See <a href=\"#b\">b</a> and <a href=\"#nowhere\">nothing</a>.</p>\n");
file_put_contents("$dir/views/nodes/b.php", "+++\nid = \"b\"\ntitle = \"B\"\nsummary = \"s\"\nconcepts = []\n+++\n<?= section_open('A heading') ?>\n<?= section_close() ?>\n");
file_put_contents("$dir/views/nodes/c.php", "+++\nid = \"c\"\ntitle = \"C\"\nsummary = \"s\"\nconcepts = [\"c\"]\n+++\n<p>x</p>\n");
try {
    (new Knowledge_article($dir))->load();
} catch (RuntimeException $error) {
    echo $error->getMessage(), "\n";
}
?>
--EXPECT--
the AI article is invalid:
  - node "b": list at least one concept
  - node "b": the body contains a heading; the node title is the heading, so split the node instead
  - part "P" lists "ghost", which is not a node
  - node "c" is not listed in any part of article.toml
  - node "a": prerequisite "b" comes later in the article; move one of them, or make it `related` instead
  - node "a": links to #nowhere, which is not a node or part
