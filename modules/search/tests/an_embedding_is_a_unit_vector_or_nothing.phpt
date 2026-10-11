--TEST--
an embedding is a unit vector of the model's dimension, unknown tokens change nothing, and text with no known token has none
--FILE--
<?php
require __DIR__ . '/setup.inc';
$vector = $model->encode('What is stored in the KV cache?');
var_dump(count($vector) === $model->dim());
var_dump(abs(sqrt(array_sum(array_map(fn ($v) => $v * $v, $vector))) - 1) < 1e-9);
var_dump($model->encode("\u{1F600}\u{1F600}"));

$cosine = fn ($a, $b) => array_sum(array_map(fn ($x, $y) => $x * $y, $a, $b));
$with_unknown = $model->encode("What is stored in the KV cache? \u{1F600}");
var_dump(abs($cosine($vector, $with_unknown) - 1) < 1e-9);

$related = $cosine($model->encode('key value cache for attention'), $model->encode('the KV cache stores keys and values'));
$unrelated = $cosine($model->encode('key value cache for attention'), $model->encode('FreeBSD jails on a Hetzner box'));
var_dump($related > $unrelated);
?>
--EXPECT--
bool(true)
bool(true)
NULL
bool(true)
bool(true)
