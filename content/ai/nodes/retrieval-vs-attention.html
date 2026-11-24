<p>Both mechanisms compare vectors and favour the most similar, which is why
they are easy to confuse. They do different jobs.</p>
<p><strong>Attention</strong> runs inside the model, in every layer, for every token. Its
queries and keys are the model’s own internal states, computed with weights
learned during training. Its scope is exactly the current context window:
nothing more, nothing less.</p>
<p><strong>Retrieval</strong> usually runs outside the model, before or between generation
steps. It is a separate system: a search index, an embedding model and a
store of documents. Its scope is whatever collection it indexes, which can
be far larger than any context window. Its output is text that gets placed
into the context, where attention can then use it.</p>
<p>So <a href="#sparse-attention">sparse attention</a> is not retrieval: it narrows what a
token may look at among tokens already present. And a bigger context window
does not replace retrieval: it raises the ceiling on how much can be placed
in context, but something still has to choose what goes there, and with a
large collection it cannot be everything.</p>
<p>A few research architectures build retrieval into the model itself, letting
it look up neighbours in a large external store during the forward pass.
Even there, the lookup over the store is a separate step from ordinary
attention over the context.</p>
