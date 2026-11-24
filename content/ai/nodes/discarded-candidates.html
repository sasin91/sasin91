<p>At every step the model produces a probability for every token in its
vocabulary, and the decoding loop picks one. What happens to the rest is
simple: nothing. They are not fed back into the model, so no hidden states
and no <a href="#kv-cache">KV cache</a> entries are ever computed for them. The sequence
continues along the chosen path only.</p>
<p>An API can return the top few probabilities at each step, often called
logprobs, but that is a report about one distribution, not a set of
alternative continuations.</p>
<p>So “the model considered several answers and rejected them” is misleading for
ordinary generation. At each step there was a distribution; after it, there
is one token and one sequence. A model can only weigh alternatives in a way
that persists if it writes them out as text, which is what reasoning models
do when they try an approach, check it and back up, all within one generated
sequence.</p>
<p><strong>Speculative decoding</strong> looks like an exception but is not. A small draft
model proposes several tokens, and the large model verifies them in one
pass, keeping the ones it agrees with. Rejected drafts are thrown away, and
the result follows the large model’s distribution. It is a speed trick, not a
search over answers.</p>
<p>Exploring alternatives deliberately is possible, but it is done by the system
around the model: <a href="#search-and-verification">multiple samples and verification</a>.</p>
