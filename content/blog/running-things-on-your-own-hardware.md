+++
path = "blog/running-things-on-your-own-hardware"
title = "Where to start with self-hosting"
date = 2026-08-04
series = "self-hosting"
part = 0
description = "I set up Kubernetes to learn it, then packaged the second cluster so it would launch in an afternoon. What the kit costs, what it uses before your own code runs, and where k3s is genuinely stronger than the simple options."
+++

I set up Kubernetes to learn it. The cluster itself took an afternoon, but
everything after it took the rest of the week: delivery, secrets, certificates,
isolation between tenants, observability, backups. I did not want to pay for
that week twice, so the second cluster became a repository you can fork.

- **[K3s starter kit on Hetzner cloud](/blog/k3s-platform-kit/).** What it costs
  (**€122.45 a month**, measured), what it uses before any of your own code runs
  (**4,526 Mi**, also measured), and where k3s is genuinely stronger than the
  simple options. Which, under about ten services, is nowhere.

- **[What a game world server actually
  needs](/blog/what-a-game-world-server-needs/).** A worked example of the same
  decision: what to buy for a WoW emulator, and which half of it a cluster is
  any use for.

The platform itself is at
[hcloud-k3s-platform-kit](https://github.com/sasin91/hcloud-k3s-platform-kit):
fork it, edit the values, run it. All decisions are documented clearly with
their trade-offs.

Choose your tools wisely, and don't reach for the complex solutions if you don't
need them. A single [€7 box](/blog/athletos-freebsd/) is likely adequate for
most apps.

Every number in these posts was measured on a live Hetzner Cloud cluster.
