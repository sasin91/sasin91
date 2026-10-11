+++
id = "sampling-controls"
title = "Temperature, top-k and top-p"
summary = "Temperature reshapes the distribution; top-k and top-p cut off its unlikely tail before a token is drawn."
concepts = ["temperature", "top-k", "top-p", "nucleus sampling", "min-p"]
prerequisites = ["sampling"]
related = ["softmax"]
search_terms = ["top p", "top k", "nucleus", "creativity", "randomness", "deterministic output", "temperature 0"]
learning_objectives = ["Describe the effect of temperature, top-k and top-p on the next-token distribution"]
misconceptions = ["Raising the temperature makes the model think more creatively."]
+++
<p>These settings act on the final distribution, just before a token is drawn:</p>
<ul>
<li><strong>Temperature</strong> divides every logit by a number <?= math('T') ?> before
<a href="#softmax">softmax</a>. Below 1 the distribution gets sharper and the likely
tokens dominate; as <?= math('T') ?> approaches 0 it approaches greedy decoding. Above
1 it flattens, and unlikely tokens are chosen more often.</li>
<li><strong>Top-k</strong> keeps only the <?= math('k') ?> most probable tokens and renormalises.</li>
<li><strong>Top-p</strong>, or nucleus sampling, keeps the smallest set of tokens whose
probabilities add up to at least <?= math('p') ?>, so the cut adapts to how spread out
the distribution is.</li>
<li><strong>Min-p</strong> keeps tokens whose probability is at least some fraction of the
top token’s.</li>
</ul>
<p>None of these change what the model computed. They change which of its
candidates are eligible and how often each is picked. “More creative” output
at a high temperature is the same model drawing more often from the tail of
the same distribution.</p>
<p>Even at temperature 0, output is not always perfectly repeatable in practice,
because of how floating-point work is batched and parallelised on
accelerators.</p>
