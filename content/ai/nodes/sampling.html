<p>The model only ever predicts one next token. Generating a reply is a loop
around that:</p>
<ol>
<li>Run the model on the current sequence and get a distribution for the next
token.</li>
<li>Choose one token from it.</li>
<li>Append that token to the sequence and go back to step 1.</li>
</ol>
<p>The loop stops when the model produces its end-of-sequence token or a length
limit is reached. This is why replies stream out token by token: each one
really does exist only after the step that chose it.</p>
<p><strong>Greedy</strong> decoding always takes the most probable token. <strong>Sampling</strong> draws a
token at random according to the probabilities, usually after reshaping them
with <a href="#sampling-controls">temperature, top-k or top-p</a>. <strong>Beam search</strong> keeps
several partial sequences and extends the best few; it is common in
translation and rarely used for chat models.</p>
<p>The decoding strategy is part of the inference software, not of the model’s
weights. The same model can produce noticeably different text under
different settings.</p>
