<p>A large language model is a stack of dozens of transformer blocks, often
somewhere between 30 and 100 or more. Each block reads the residual stream,
computes something, and adds its result back, so information accumulates
rather than being replaced.</p>
<p>Every block has two parts:</p>
<ul>
<li><strong>Attention</strong> moves information between positions. It is the only place
where one token’s representation can be influenced by another’s. See
<a href="#attention">self-attention</a>.</li>
<li>A <strong>feed-forward network</strong> (MLP) then transforms each position separately,
with the same weights at every position. Much of what a model has absorbed
about the world appears to be stored in these weights.</li>
</ul>
<p>Normalisation steps keep the numbers in a stable range between the two.</p>
<p>The weights of each block are fixed after training and shared across
positions, which is why one model can process a prompt of five tokens or five
thousand: the same computation is applied at every position. Chat models are
<strong>decoder-only</strong> transformers with a causal mask, meaning each position can
only attend to positions before it.</p>
<p>Many recent large models replace the single MLP with a <strong>mixture of experts</strong>:
several MLPs, of which a small router picks a few for each token. The model
has more parameters in total, but each token only uses a fraction of them.</p>
