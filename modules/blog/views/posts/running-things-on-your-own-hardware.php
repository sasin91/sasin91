+++
path = "blog/running-things-on-your-own-hardware"
title = "Where to start with self-hosting"
date = 2026-08-04
series = "self-hosting"
part = 0
description = "I set up Kubernetes to learn it, then packaged the second cluster so it would launch in an afternoon. What the kit costs, what it uses before your own code runs, and where k3s is genuinely stronger than the simple options."
+++
<p>I set up Kubernetes to learn it. The cluster itself took an afternoon, but
everything after it took the rest of the week: delivery, secrets, certificates,
isolation between tenants, observability, backups. I did not want to pay for
that week twice, so the second cluster became a repository you can fork.</p>
<ul>
<li>
<p><strong><a href="/blog/k3s-platform-kit/">K3s starter kit on Hetzner cloud</a>.</strong> What it costs
(<strong>€122.45 a month</strong>, measured), what it uses before any of your own code runs
(<strong>4,526 Mi</strong>, also measured), and where k3s is genuinely stronger than the
simple options. Which, under about ten services, is nowhere.</p>
</li>
<li>
<p><strong><a href="/blog/what-a-game-world-server-needs/">What a game world server actually
needs</a>.</strong> A worked example of the same
decision: what to buy for a WoW emulator, and which half of it a cluster is
any use for.</p>
</li>
</ul>
<p>The platform itself is at
<a href="https://github.com/sasin91/hcloud-k3s-platform-kit">hcloud-k3s-platform-kit</a>:
fork it, edit the values, run it. All decisions are documented clearly with
their trade-offs.</p>
<p>Choose your tools wisely, and don’t reach for the complex solutions if you don’t
need them. A single <a href="/blog/athletos-freebsd/">€7 box</a> is likely adequate for
most apps.</p>
<p>Every number in these posts was measured on a live Hetzner Cloud cluster.</p>
