+++
id = "code-retrieval"
title = "Codebase-aware retrieval"
summary = "Coding agents find the relevant parts of a repository with search tools, such as text search, symbol indexes, embeddings or code graphs, and read them into context."
concepts = ["code search", "codebase retrieval", "code knowledge graph", "symbol index", "language server", "grep"]
prerequisites = ["rag", "tool-use"]
related = ["mcp", "external-memory", "retrieval-vs-attention"]
search_terms = ["coding agent", "repository", "repo", "ripgrep", "code graph", "how do coding agents understand a codebase", "lsp"]
learning_objectives = [
    "List the main ways an agent searches a codebase and when each works best",
    "Explain why exact identifiers favour lexical search",
]
misconceptions = ["A coding agent reads and understands the whole repository at once."]
+++
<p>A repository rarely fits in a context window, and most of it is irrelevant
to any given task. So a coding agent does what a developer new to the code
would do: it searches, reads, and searches again. The usual instruments:</p>
<ul>
<li><strong>Text search</strong>, grep and its descendants: exact and fast, ideal for a known
identifier or error message.</li>
<li><strong>File listings</strong>: the shape of the project, from its directory structure.</li>
<li><strong>Symbol indexes</strong> and language servers: where something is defined, and
everything that refers to it.</li>
<li><strong>Embedding search</strong> over chunks of code and documentation, for questions
phrased in concepts rather than names.</li>
<li><strong>Code knowledge graphs</strong>: precomputed relations such as calls, imports and
ownership, queried as a tool and often exposed over <a href="#mcp">MCP</a>.</li>
</ul>
<p>The agent chooses which to use, reads the results into context, and lets
attention do the reasoning from there. Each tool is a different kind of
<a href="#rag">retrieval</a>, and they complement each other: an exact identifier such
as a function name is found reliably by text search and easily blurred by
semantic similarity. The <a href="/blog/ai/ask/">Ask</a> view of this article uses the same
idea, letting exact keyword matches outrank vague semantic ones.</p>
