<p>A model’s weights are numbers, billions of them, and each is normally stored
in 16 bits. <strong>Quantization</strong> stores each one in fewer bits instead: 8, 4,
sometimes less. The memory the weights need is close to</p>
<p><math display="block"><mtext>parameters</mtext><mo>×</mo><mfrac><mtext>bits per weight</mtext><mn>8</mn></mfrac><mtext>&nbsp;bytes</mtext></math></p>
<p>so an 8-billion-parameter model takes about 16 GB at 16 bits, 8 GB at 8 bits
and a little over 4 GB at 4 bits.</p>
<p>Fewer bits means fewer distinct values, so every weight is rounded to the
nearest one that can be represented. To keep that rounding small, weights
are grouped, by row or in blocks of a few dozen, and each group stores its
own <strong>scale</strong>: the group’s largest value maps to the top of the small
integer range and everything else is rounded relative to it. A large weight
in one group then cannot crush the precision of every other group.</p>
<p>It is the same model, rounded. No training happens and nothing new is
learned: a quantized model is an approximation of the original weights, not a
smaller model trained to imitate it. That second thing exists too, and is
called distillation. At 8 bits the difference is usually hard to measure; at
4 bits it is small for most tasks; below about 3 bits quality drops
noticeably, and smaller models suffer from it sooner than large ones.</p>
<p>Weights are not the only thing that can be quantized. The
<a href="#kv-cache">KV cache</a> can be stored in fewer bits too, which matters for
long contexts. The embedding model behind this article’s
<a href="/blog/ai/ask/">Ask</a> view is stored as 8-bit integers with one scale per
row; against the original 32-bit numbers, the average similarity between each
row and its rounded copy is 0.99997.</p>
