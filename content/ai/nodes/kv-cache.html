<p>A model generates text one token at a time, and each new token attends to
every token before it. Without a cache, every step would recompute the
<a href="#qkv">keys and values</a> for the entire prefix, again and again.</p>
<p>Because a token’s keys and values never change once computed, the inference
engine stores them: for every layer, and for every position processed so far.
Each new step computes the query, key and value for the one new token,
appends its key and value to the cache, and attends over the stored ones.</p>
<p>The cache grows linearly with the context. Its size is roughly</p>
<p><math display="block"><mn>2</mn><mo>×</mo><mtext>layers</mtext><mo>×</mo><mtext>kv heads</mtext><mo>×</mo><mtext>head size</mtext><mo>×</mo><mtext>tokens</mtext><mo>×</mo><mtext>bytes per number</mtext></math></p>
<p>which for long contexts can exceed the size of the model weights.
Grouped-query attention, quantised caches and paged memory management all
exist to keep it in check.</p>
<p>What the cache holds is precise: the attention state for the exact sequence
that was actually processed, meaning the prompt plus the tokens that were
actually chosen. Candidate tokens that were not sampled were never fed back
in, so no keys or values exist for them. The cache is not a store of
<a href="#discarded-candidates">alternatives</a>.</p>
<p>Nor is it learning. The cache lives for one request, or is kept briefly to
reuse a shared prefix such as a long system prompt across requests, which is
what “prompt caching” usually refers to. The model’s weights are untouched
throughout.</p>
