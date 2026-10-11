+++
id = "search-and-verification"
title = "Multiple samples, search and verification"
summary = "Systems can explore alternatives deliberately: generate several candidates, search over partial solutions, and check the results. It is an outer loop around generation."
concepts = ["best-of-n", "self-consistency", "verification", "tree search", "test-time compute", "reward model"]
prerequisites = ["discarded-candidates"]
related = ["synthetic-data", "tool-use"]
search_terms = ["multiple samples", "majority vote", "verifier", "reasoning models", "thinking", "test time compute", "generate and test"]
learning_objectives = [
    "Describe best-of-n sampling and self-consistency",
    "Explain why multi-candidate search is an outer loop rather than part of token generation",
]
misconceptions = ["Running more candidates at inference time trains the model."]
+++
<p>If one sample can be wrong, generate several. That simple idea has many
forms:</p>
<ul>
<li><strong>Best-of-n</strong>: sample <?= math('n') ?> complete answers and pick one with a scoring model
or a check.</li>
<li><strong>Self-consistency</strong>: sample several reasoning paths and take the answer most
of them agree on.</li>
<li><strong>Search</strong>: treat partial solutions as nodes in a tree, extend the promising
ones and abandon the rest.</li>
<li><strong>Verification</strong>: run the unit tests, check the proof, execute the query, and
keep what passes.</li>
</ul>
<p>Each candidate is a separate generation, with its own context and its own
KV cache, although a shared prompt prefix can be cached once. The
alternatives exist because the inference system asked for them, not because
the model kept its <a href="#discarded-candidates">discarded candidates</a>. Spending
more computation this way, at answer time rather than in training, is often
called <strong>test-time compute</strong>.</p>
<p>None of this changes the model. Candidates that pass a verifier can later be
collected and used to train a better model, but that happens afterwards, in a
separate process: see <a href="#synthetic-data">synthetic data</a>.</p>
