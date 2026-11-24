<p>Inside an attention head, each token’s current vector is multiplied by three
learned matrices, producing three new vectors:</p>
<ul>
<li>a <strong>query</strong>: what this position is looking for;</li>
<li>a <strong>key</strong>: what this position can be matched on;</li>
<li>a <strong>value</strong>: what this position contributes if it is attended to.</li>
</ul>
<p>The score between position <math><mi>i</mi></math> and position <math><mi>j</mi></math> is the dot product of
<math><mi>i</mi></math>’s query with <math><mi>j</mi></math>’s key, divided by the square root of the key length to
keep the numbers in a stable range. A <a href="#softmax">softmax</a> turns each row of
scores into weights that sum to one, and the output for <math><mi>i</mi></math> is the weighted
sum of the values:</p>
<p><math display="block"><mi>Attention</mi><mo>(</mo><mi>Q</mi><mo>,</mo><mi>K</mi><mo>,</mo><mi>V</mi><mo>)</mo><mo>=</mo><mi>softmax</mi><mrow><mo>(</mo><mfrac><mrow><mi>Q</mi><msup><mi>K</mi><mi>T</mi></msup></mrow><msqrt><msub><mi>d</mi><mi>k</mi></msub></msqrt></mfrac><mo>)</mo></mrow><mi>V</mi></math></p>
<p>The database vocabulary is only an analogy. Every key matches every query to
some degree, and the output is a blend, not a single selected record.</p>
<p>Keys and values depend only on a token and the tokens before it, never on
later ones, so once computed they can be reused. That is what the
<a href="#kv-cache">KV cache</a> stores. <strong>Grouped-query attention</strong> lets several query
heads share one set of keys and values, which shrinks that cache
considerably.</p>
