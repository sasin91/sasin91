+++
id = "attention"
title = "Self-attention"
summary = "Each token's representation gathers information from other positions in the context, weighted by how relevant the model computes them to be."
concepts = ["self-attention", "attention", "attention head", "attention weights", "causal mask"]
prerequisites = ["transformer-layers"]
related = ["qkv", "retrieval-vs-attention", "context-window"]
search_terms = ["attention mechanism", "multi-head attention", "attention is all you need", "how does attention work"]
learning_objectives = [
    "Explain what attention computes for one position",
    "State the scope of what attention can reach",
]
misconceptions = ["Attention lets the model look things up beyond what is in its context."]
+++
<p>Attention is how a transformer moves information between positions. For each
position, it computes a weight for every position it is allowed to look at,
then takes a weighted average of information from those positions. In a
decoder-only model the allowed positions are the current one and everything
before it.</p>
<p>The weights are computed fresh for every input. That is the important
difference from the rest of the network: the MLP weights are fixed numbers,
but how much “it” draws from “the cat” versus “the mat” is decided at run
time, from the content itself. How those weights are computed is the subject
of <a href="#qkv">queries, keys and values</a>.</p>
<p>Each layer runs several attention <strong>heads</strong> in parallel, each with its own
learned projections, so different heads can track different relations:
the previous token, the subject of the sentence, a matching bracket.</p>
<p>Attention only ever operates over what is in the
<a href="#context-window">context window</a>. It has no access to documents, databases
or earlier conversations unless something has put them there as tokens. That
is a job for <a href="#retrieval-vs-attention">retrieval</a>, which is a different
mechanism.</p>
