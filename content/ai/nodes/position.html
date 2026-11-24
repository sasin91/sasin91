<p><a href="#attention">Self-attention</a> compares tokens with each other, but nothing in
that comparison depends on where they are. Without extra information, “dog
bites man” and “man bites dog” would give the model the same tokens in a
different order, and it would have no way to tell them apart.</p>
<p>Early transformers fixed this by <strong>adding</strong> a position vector to each token’s
embedding: fixed sinusoidal patterns in the original Transformer, a learned
table of positions in BERT and GPT-2.</p>
<p>Most current large models use <strong>rotary position embeddings</strong> (RoPE) instead.
Nothing is added to the embedding. Inside every attention layer, query and
key vectors are rotated by an angle that depends on their position, so the
score between two tokens ends up depending on how far apart they are. Other
schemes, such as ALiBi, add a distance-based penalty to attention scores
directly.</p>
<p>The choice matters for <a href="#context-window">context length</a>: a model handles
positions well mostly within the range it was trained on, and many
long-context techniques work by adjusting how RoPE behaves beyond that range.</p>
