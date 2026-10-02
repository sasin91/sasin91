+++
id = "text"
title = "Text"
summary = "A language model never reads letters directly: before it can do anything, text has to become a sequence of integers."
concepts = ["text", "input", "unicode", "bytes", "preprocessing"]
related = ["tokenization"]
search_terms = ["characters", "letters", "utf-8", "how does an llm read text", "input text"]
learning_objectives = [
    "Explain why a neural network cannot consume text as written",
    "Name the steps between a string and the model's first computation",
]
misconceptions = ["A language model reads the letters of a prompt the way a person does."]
+++

A computer stores text as bytes: Unicode characters encoded, usually, as
UTF-8. A neural network does not compute with bytes or letters. It computes
with numbers arranged in vectors and matrices, and the only thing it can do
with an input is multiply and add it.

So between the prompt you type and the first calculation inside the model
there is a fixed, mechanical pipeline. The text is normalised, split into
[tokens](#tokenization), and each token is replaced by an integer
[ID](#token-ids). Those integers are the model's entire view of your text.

Nothing in that pipeline is learned while you use the model, and nothing in
it understands anything. It is closer to a dictionary lookup than to reading.

This has consequences that surprise people. A model that receives `straw`
and `berry` as two IDs has no direct view of the letters inside them, so
"how many r's are in strawberry" asks about something the model only knows
indirectly, from patterns it absorbed during training.
