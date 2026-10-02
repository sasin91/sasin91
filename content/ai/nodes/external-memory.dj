+++
id = "external-memory"
title = "External memory"
summary = "Anything a system should remember beyond one context window has to be stored outside the model and read back in as tokens."
concepts = ["memory", "external memory", "long-term memory", "persistence", "agent memory"]
prerequisites = ["context-window", "rag"]
related = ["kv-cache", "training", "tool-use"]
search_terms = ["does the model remember", "chat memory", "remember previous conversations", "notes", "scratchpad", "vector store"]
learning_objectives = [
    "Explain why a model has no memory across requests by itself",
    "Describe how memory features are built from storage and retrieval",
]
misconceptions = ["A chat assistant remembers you because the model learned from your earlier conversations."]
+++

During ordinary use a model's weights do not change, and its context is
discarded when a request ends. Left to itself, it remembers nothing from one
conversation to the next.

Memory features are built around the model instead. The application saves
something, such as a summary of a conversation, a note the model was asked to
keep, or the files an agent wrote, and later retrieves the relevant parts and
places them back into the context. The model "remembers" only by reading.

The storage can be anything: a text file, a database table, a vector index.
The hard parts are the same as in [RAG](#rag): deciding what is worth
saving, finding the right piece later, and keeping stored facts accurate as
things change.

This is different from the [KV cache](#kv-cache), which only lasts for one
request, and from [training](#training), which changes weights in a separate
process. Coding agents rely on external memory constantly: task lists,
notes, and the repository itself, re-read through
[tools](#tool-use) whenever needed.
