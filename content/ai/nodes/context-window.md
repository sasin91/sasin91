+++
id = "context-window"
title = "Context windows"
summary = "The context window is the most tokens a model can process at once: prompt, documents, tool results and its own output together."
concepts = ["context window", "context length", "context", "long context", "prompt"]
prerequisites = ["attention", "tokenization"]
related = ["kv-cache", "rag", "retrieval-vs-attention", "external-memory"]
search_terms = ["context size", "token limit", "max tokens", "128k", "1 million tokens", "lost in the middle", "larger context window"]
learning_objectives = [
    "Define the context window precisely",
    "Explain why a larger context window does not remove the need for retrieval",
]
misconceptions = [
    "The context window is a slice of the model's embeddings.",
    "Everything inside the context window is used equally well.",
    "A large enough context window makes retrieval unnecessary.",
]
+++

The context window is a limit on sequence length: the largest number of
tokens the model can process in one pass. It is not a region of memory inside
the model, and not a slice of its embeddings. It is simply the input,
measured in tokens.

Everything the model can take into account has to be inside it: the system
prompt, the conversation so far, any documents or tool results that were
pasted in, and every token the model has generated in this response. When a
conversation outgrows it, something has to be dropped or summarised.

The limit comes from several places at once: the lengths the model was
trained on, how its [positional scheme](#position) extends, the
[quadratic cost](#dense-attention) of attention, and the memory needed for
the [KV cache](#kv-cache).

Windows have grown from a few thousand tokens to hundreds of thousands and
more. That helps, but it does not make retrieval obsolete. Models measurably
use information less reliably when it is buried in the middle of a very long
input. Every token in the window costs time and money on every request. And
most importantly, a window only holds what someone put into it: a document
collection of millions of pages still has to be searched, and choosing what
to place in the context is what [RAG](#rag) is for.
