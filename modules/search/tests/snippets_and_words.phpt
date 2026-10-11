--TEST--
a snippet is cut on a word boundary near 160 characters, and words split like retrieval.mjs splits them
--FILE--
<?php
require __DIR__ . '/setup.inc';
$index = new Search_index($model, new Markdown_html());
echo $index->snippet('Short enough.'), "\n";
$long = str_repeat('word ', 40) . 'end.';
$snippet = $index->snippet($long);
var_dump(mb_strlen($snippet) <= 161, str_ends_with($snippet, "word\u{2026}"));
echo implode(' ', Search_index::lexical_words("KV-cache, the KV cache: Ærø's 2 caches")), "\n";
?>
--EXPECT--
Short enough.
bool(true)
bool(true)
2 cache caches kv s the ærø
