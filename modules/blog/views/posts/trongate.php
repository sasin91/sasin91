+++
path = "blog/trongate"
title = "Trongate PHP"
date = 2024-09-14
description = "Trongate is often misunderstood and gets a bad reputation because it breaks with common standards and takes a journey back to its roots. In this article, I will explore and highlight this rough diamond that deserves a spot in the limelight."
+++
<p><img src="/images/trongate/trongate_logo_trans_bg.png" alt="Trongate: The framework they don’t want you to know about" /></p>
<p>An exciting PHP framework that challenges the established standards and offers a simple yet effective approach to development.</p>
<p>I have worked with PHP since 2014, and one recurring thing is <em>unnecessary</em> code
that needs maintenance. Occam’s razor is clearly visible when all you want is to
tear <em>everything</em> out by the roots and start fresh, hoping to build something
more simple.</p>
<blockquote>
<p>This is where Trongate shines; you get a simple starting point with a solid starting architecture due to everything being divided into modules, including media and JavaScript files.</p>
</blockquote>
<p><img src="/images/trongate/trongate_lvim_mangos-account.png" alt="Trongate modul struktur" /></p>
<p><a href="https://trongate.io/docs/basic-concepts/truly-modular-architecture.html">Read more here</a></p>
<p>A criticism that could be made is that internationalization (i18n) hasn’t been considered, and there is actually no real language support. This is true and something I have <a href="https://github.com/trongate/trongate-framework/pull/189">previously</a> addressed. It’s on the roadmap, and I look forward to seeing how it gets resolved. :)</p>
<p><img src="/images/trongate/trongate_lvim_validation.png" alt="Trongate validation" /></p>
<p>Another feature that often flies under the radar is <em>callback</em> based validation.</p>
<p>Before I go, I just want to shout out:</p>
<?= section_open('Trongate MX') ?>
<p>A library based on <a href="https://htmx.org">HTMX</a> and in many ways has the same features, just using mx- instead of hx-.</p>
<p>And yet it offers so much more, with close integration, built-in modal support, <a href="https://trongate.io/docs/trongate-mx/managing-ui-during-requests.html">“optimistic” UI</a>, and <a href="https://trongate.io/docs/trongate-mx/after-swap-operations.html">more.</a></p>
<p>My hope with this article is to spark people’s interest and highlight a rough diamond that unfortunately has gained a somewhat bad reputation.</p>
<p>A couple of things I have built around it: <a href="https://github.com/sasin91/trongate-cors">trongate-cors</a> and <a href="https://github.com/sasin91/trongate-provision">trongate-provision</a>, an experiment in deploying Trongate apps.</p>
<p>// sasin91, 18.09.2024</p>
<?= section_close() ?>
