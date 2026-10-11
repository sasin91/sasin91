+++
id = "rag"
title = "Retrieval-augmented generation"
summary = "RAG searches an external collection at request time and places the most relevant passages in the context before the model answers."
concepts = ["retrieval-augmented generation", "rag", "retrieval", "vector search", "chunking", "reranking"]
prerequisites = ["context-window", "embeddings"]
related = ["retrieval-vs-attention", "external-memory", "code-retrieval"]
search_terms = ["retrieval augmented generation", "vector database", "semantic search", "knowledge base", "grounding", "citations", "document search"]
learning_objectives = [
    "Walk through the steps of a RAG pipeline",
    "Name the main ways a RAG system fails",
]
misconceptions = ["RAG teaches the model new facts by updating it."]
+++
<p>A model knows only what it absorbed in training plus what is in its
<a href="#context-window">context</a>. Retrieval-augmented generation adds a step
before generation that decides what else should be in that context:</p>
<ol>
<li>Ahead of time, split a document collection into passages (“chunks”) and
index them, usually by computing an <a href="#embeddings">embedding</a> for each and
often with a keyword index as well.</li>
<li>When a question arrives, embed it the same way and find the passages
whose vectors are most similar, optionally re-scoring the best few with a
more expensive model.</li>
<li>Insert those passages into the prompt, and let the model generate with
them in view.</li>
</ol>
<p>That gives a model access to private, recent or very large collections
without retraining anything, and lets it cite its sources. The model’s
weights are not changed at all.</p>
<p>RAG fails in recognisable ways. If retrieval misses the passage that
matters, the model answers without it, often fluently. If it retrieves
near-misses, they can distract or mislead. Most of the work in a good RAG
system is in the retrieval, not the generation.</p>
<p>The <a href="/blog/ai/ask/">Ask</a> view of this article is the retrieval half of this
pipeline, without the generating half: it embeds your question in your
browser and returns the closest sections, which you then read yourself.</p>
