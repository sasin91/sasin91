+++
id = "embeddings"
title = "Embeddings"
summary = "Token IDs select rows of a learned table of vectors: the model's starting point for each token, not its final word on meaning."
concepts = ["embedding", "embedding table", "vector", "static embedding", "embedding model"]
prerequisites = ["token-ids"]
related = ["contextual-representations", "rag", "retrieval-vs-attention"]
search_terms = ["word vectors", "vector representation", "semantic vectors", "embedding matrix", "meaning of words"]
learning_objectives = [
    "Describe what an embedding table is and how it is used",
    "Distinguish a token's input embedding from its contextual representation",
]
misconceptions = ["An embedding is a fixed definition of what a word means."]
+++
<p>The model’s first layer is a table with one row per vocabulary entry. Each
row is a vector of a few hundred to several thousand numbers, and a token ID
simply selects its row. The values in the table are learned during training,
along with everything else.</p>
<p>Training tends to place tokens that are used in similar ways near each other,
which is why embeddings are often described as encoding meaning. For this
table, that description goes too far. The input embedding for <code>bank</code> is the
same row whether the sentence is about a river or an account: it is a
context-free starting point. What a token means in a particular sentence is
only worked out as its vector is transformed by the layers above, using the
tokens around it. That is the subject of
<a href="#contextual-representations">contextual representations</a>.</p>
<p>The word <strong>embedding</strong> is also used for something related but separate: an
embedding <strong>model</strong> that turns a whole passage into one vector, so passages can
be compared by similarity. That is how <a href="#rag">retrieval</a> usually finds
relevant text.</p>
<p>The <a href="/blog/ai/ask/">Ask</a> view of this article uses the simplest kind: a static
embedding table, where a passage’s vector is the average of its tokens’ rows.
No context and no layers, so it runs in a browser without a machine-learning
runtime. It is also why Ask combines it with plain keyword matching: a static
table cannot tell which sense of a word you meant.</p>
