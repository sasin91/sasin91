+++
id = "self-improvement"
title = "Recursive self-improvement"
summary = "The idea that an AI system could improve its own capabilities, including its ability to improve itself. A research concept, not a current mechanism."
concepts = ["recursive self-improvement", "intelligence explosion", "automated ai research", "feedback loop"]
prerequisites = ["synthetic-data"]
related = ["agi", "training"]
search_terms = ["singularity", "self improving ai", "ai improving ai", "takeoff", "seed ai"]
learning_objectives = [
    "State the recursive self-improvement hypothesis",
    "Identify which parts of today's training loops resemble it, and which do not",
]
misconceptions = ["Current models rewrite their own weights to become smarter while they run."]
+++

The hypothesis is old: a system able to improve the system that produces it
could make each improvement come faster than the last. In 1965 the
statistician I. J. Good called the possible result an "intelligence
explosion".

Pieces of today's practice point in that direction. Models generate and
filter [their own training data](#synthetic-data), write code for machine
learning experiments, and help evaluate other models. But every one of those
loops still runs through training jobs, compute budgets and evaluations that
people design and launch, and no model changes its own weights while it
runs.

Whether such loops could become autonomous and compounding, or would run into
limits of data, compute, or the ability to verify that a change really is an
improvement, is an open research question. The honest summary is that the
concept is well defined and seriously studied, and the mechanism does not
exist in deployed systems today.
