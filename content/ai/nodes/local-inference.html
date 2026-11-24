<p>To run a model, a machine needs room for three things: the weights, the
<a href="#kv-cache">KV cache</a> for the context in use, and some working memory for
the runtime. The weights are fixed by the model and its
<a href="#quantization">quantization</a>; the cache grows with the context. A 4-bit
8-billion-parameter model fits comfortably in 8 GB; the same model with a
long context may not.</p>
<p>Inference has two phases with different limits. <strong>Prefill</strong> reads the
prompt, and all of its tokens can be processed at once, so it is limited by
how much arithmetic the hardware can do. <strong>Decode</strong> then generates one token
at a time, and every token needs a pass through all of the weights. For a
single user, that pass is mostly waiting for memory, not computing, so a
rough ceiling is</p>
<p><math display="block"><mtext>tokens per second</mtext><mo>≈</mo><mfrac><mtext>memory bandwidth</mtext><mtext>bytes of weights read per token</mtext></mfrac></math></p>
<p>A machine that reads 100 GB/s, running a 5 GB model, tops out near 20 tokens
a second however fast its cores are. This is why GPUs, with several times the
memory bandwidth of a desktop CPU, generate faster, why quantization speeds
generation up as well as shrinking the download, and why machines with fast
memory shared between CPU and GPU run large models well.</p>
<p>A <a href="#transformer-layers">mixture-of-experts</a> model bends the rule: all of
its experts must be in memory, but each token reads only the few the router
picks, so it generates faster than its total size suggests.</p>
<p>Running locally keeps every prompt on the machine and costs nothing per
token. What it gives up is size: the largest models need far more memory than
one machine usually has, and a provider can batch many users’ requests through
each read of the weights, which a single person cannot.</p>
