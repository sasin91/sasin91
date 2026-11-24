<p>In the standard design, every position attends to every position before it.
For a sequence of <math><mi>n</mi></math> tokens that is roughly <math><msup><mi>n</mi><mn>2</mn></msup><mo>/</mo><mn>2</mn></math> query-key scores per
head, per layer. Double the length of the input and the attention work
roughly quadruples, while the rest of the network’s work only doubles.</p>
<p>For short prompts this hardly matters. For hundreds of thousands of tokens it
dominates, and it is one of the reasons <a href="#context-window">context windows</a>
have limits at all.</p>
<p>A lot of engineering goes into making dense attention cheaper <strong>without
changing its result</strong>. FlashAttention, for example, computes exactly the same
output, but in tiles that stay in the GPU’s fast on-chip memory instead of
writing the whole score matrix out and reading it back. That is an
implementation detail, not an approximation.</p>
<p>Changing which positions may attend to which, so that less work is done in
the first place, is a different idea: <a href="#sparse-attention">sparse attention</a>.</p>
