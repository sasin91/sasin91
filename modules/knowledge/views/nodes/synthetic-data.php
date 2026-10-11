+++
id = "synthetic-data"
title = "Self-training and synthetic data"
summary = "Model outputs that have been filtered, checked or ranked can become training data for a later model: a learning loop outside generation."
concepts = ["synthetic data", "self-training", "distillation", "rejection sampling", "model collapse"]
prerequisites = ["training", "search-and-verification"]
related = ["self-improvement"]
search_terms = ["training on model outputs", "ai generated training data", "self improvement loop", "bootstrapping", "data flywheel"]
learning_objectives = [
    "Describe a generate, verify and train loop",
    "Explain why verification quality limits self-training",
]
misconceptions = ["A model improves itself every time it generates and checks an answer."]
+++
<p>Generated text can be training data. Some common patterns:</p>
<ul>
<li>Generate many solutions to problems with checkable answers, keep the ones
that pass, and fine-tune on those.</li>
<li><strong>Distillation</strong>: train a smaller model on the outputs of a larger one.</li>
<li>Have a model critique or rank responses, and use the rankings as
preference data.</li>
</ul>
<p>All of these are an outer loop with a clear sequence: generate, using
inference; evaluate, with tests, verifiers or other models; select; then
train, which is the only step where weights change, producing a new model.
The <a href="#search-and-verification">multi-candidate search</a> of the inference step
and the learning of the training step are different stages, often run at
different times on different machines.</p>
<p>The loop is only as good as its filter. If verification is weak, errors get
trained in and reinforced. Repeatedly training on unfiltered generated text
has been shown to narrow a model’s output and lose the rarer parts of the
original data, sometimes called <strong>model collapse</strong>. Where answers can be
checked mechanically, as in mathematics and code, the loop works far better
than where they cannot.</p>
