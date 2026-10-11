+++
id = "sparse-attention"
title = "Sparse and sliding-window attention"
summary = "Some attention layers restrict each token to a window or pattern of positions, trading direct reach for lower cost. It all happens inside the model."
concepts = ["sparse attention", "sliding-window attention", "local attention", "attention pattern", "receptive field"]
prerequisites = ["dense-attention"]
related = ["retrieval-vs-attention", "context-window"]
search_terms = ["sliding window", "local attention", "longformer", "bigbird", "block sparse", "windowed attention"]
learning_objectives = [
    "Describe sliding-window attention and how information still travels beyond the window",
    "Explain why sparse attention is not a form of retrieval",
]
misconceptions = ["Sparse attention fetches the relevant parts of a long document, like search."]
+++
<p>Sparse attention changes the rule of <a href="#dense-attention">dense attention</a>
that every token may attend to every earlier one. In <strong>sliding-window</strong>
attention, each token attends only to the most recent few thousand
positions. Other designs combine a local window with a few global tokens,
use fixed strided or block patterns, or select positions dynamically.</p>
<p>A token outside the window is not lost. Each layer passes information
forward by up to one window, so stacking layers lets information travel much
further, indirectly. Several open models alternate local-window layers with
full-attention layers to get most of the saving while keeping some direct
long-range reach.</p>
<p>Sparse attention is entirely internal to the transformer. It decides which
positions <strong>already in the context</strong> each token may look at, and it is fixed
by the architecture. It does not search a document collection, and it does
not bring anything new into the context. That is what
<a href="#retrieval-vs-attention">retrieval</a> does, outside the model.</p>
