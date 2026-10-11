+++
id = "mcp"
title = "Model Context Protocol (MCP)"
summary = "MCP is an open protocol for exposing tools, data and prompts to AI applications. It standardises the connection, not the retrieval."
concepts = ["model context protocol", "mcp", "protocol", "mcp server", "mcp client", "json-rpc"]
prerequisites = ["tool-use"]
related = ["rag", "code-retrieval"]
search_terms = ["mcp server", "what is mcp", "is mcp rag", "connectors", "integrations", "anthropic mcp"]
learning_objectives = [
    "Describe the parts of an MCP connection and what a server can expose",
    "Explain why MCP is not itself retrieval or memory",
]
misconceptions = ["MCP is a kind of RAG.", "MCP gives the model direct access to systems."]
+++
<p>The Model Context Protocol, introduced by Anthropic in late 2024 and now
implemented by many applications, is a standard way to connect AI
applications to outside capabilities. A <strong>host</strong> application, such as a chat
app or an editor, runs MCP <strong>clients</strong> that talk to MCP <strong>servers</strong>. A server can
offer:</p>
<ul>
<li><strong>tools</strong>: functions the model may ask to call;</li>
<li><strong>resources</strong>: data the application can read and place in context;</li>
<li><strong>prompts</strong>: reusable templates.</li>
</ul>
<p>Messages are JSON-RPC, carried over standard input and output for local
servers or over HTTP for remote ones. The benefit is reuse: an integration
written once as an MCP server works in every host that speaks the protocol.</p>
<p>MCP is plumbing. It is not a model, not a memory and not RAG. A server <strong>can</strong>
expose a search tool, and then a retrieval step happens through MCP; but the
protocol only carries the request and the result. Ranking, indexing and
relevance all belong to whatever the server does, and the model still only
sees what ends up in its context. Everything said about
<a href="#tool-use">tool use</a> still applies: the host decides what runs.</p>
