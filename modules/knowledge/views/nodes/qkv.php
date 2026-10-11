+++
id = "qkv"
title = "Queries, keys and values"
summary = "Each token is projected into a query (what it looks for), a key (what it can be matched on) and a value (what it passes along)."
concepts = ["query", "key", "value", "qkv", "dot product", "scaled dot-product attention", "grouped-query attention"]
prerequisites = ["attention"]
related = ["kv-cache", "position", "softmax"]
search_terms = ["q k v", "query key value", "attention formula", "gqa", "multi-query attention", "attention scores"]
learning_objectives = [
    "Write down scaled dot-product attention and explain each term",
    "Explain why keys and values are what gets cached",
]
misconceptions = ["Attention selects one matching token, like a database lookup."]
+++
<p>Inside an attention head, each token’s current vector is multiplied by three
learned matrices, producing three new vectors:</p>
<ul>
<li>a <strong>query</strong>: what this position is looking for;</li>
<li>a <strong>key</strong>: what this position can be matched on;</li>
<li>a <strong>value</strong>: what this position contributes if it is attended to.</li>
</ul>
<p>The score between position <?= math('i') ?> and position <?= math('j') ?> is the dot product of
<?= math('i') ?>’s query with <?= math('j') ?>’s key, divided by the square root of the key length to
keep the numbers in a stable range. A <a href="#softmax">softmax</a> turns each row of
scores into weights that sum to one, and the output for <?= math('i') ?> is the weighted
sum of the values:</p>
<p><?= math('\\mathrm{Attention}(Q, K, V) = \\mathrm{softmax}\\left(\\frac{QK^T}{\\sqrt{d_k}}\\right)V', true) ?></p>
<p>The database vocabulary is only an analogy. Every key matches every query to
some degree, and the output is a blend, not a single selected record.</p>
<p>Keys and values depend only on a token and the tokens before it, never on
later ones, so once computed they can be reused. That is what the
<a href="#kv-cache">KV cache</a> stores. <strong>Grouped-query attention</strong> lets several query
heads share one set of keys and values, which shrinks that cache
considerably.</p>
