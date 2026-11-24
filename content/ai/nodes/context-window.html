<p>The context window is a limit on sequence length: the largest number of
tokens the model can process in one pass. It is not a region of memory inside
the model, and not a slice of its embeddings. It is simply the input,
measured in tokens.</p>
<p>Everything the model can take into account has to be inside it: the system
prompt, the conversation so far, any documents or tool results that were
pasted in, and every token the model has generated in this response. When a
conversation outgrows it, something has to be dropped or summarised.</p>
<p>The limit comes from several places at once: the lengths the model was
trained on, how its <a href="#position">positional scheme</a> extends, the
<a href="#dense-attention">quadratic cost</a> of attention, and the memory needed for
the <a href="#kv-cache">KV cache</a>.</p>
<p>Windows have grown from a few thousand tokens to hundreds of thousands and
more. That helps, but it does not make retrieval obsolete. Models measurably
use information less reliably when it is buried in the middle of a very long
input. Every token in the window costs time and money on every request. And
most importantly, a window only holds what someone put into it: a document
collection of millions of pages still has to be searched, and choosing what
to place in the context is what <a href="#rag">RAG</a> is for.</p>
