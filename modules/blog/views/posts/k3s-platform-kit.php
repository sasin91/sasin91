+++
path = "blog/k3s-platform-kit"
title = "K3s starter kit on Hetzner cloud"
date = 2026-08-04
hero = "/images/k3s-platform-kit/architecture.svg"
hero_alt = "Inside a k3s cluster: a git fork reconciled by Flux, a visitor request arriving through Traefik to a tenant application, and telemetry from both flowing to a collector and on to VictoriaMetrics, VictoriaLogs and Tempo"
description = "A fork-and-edit k3s platform for Hetzner Cloud: GitOps delivery, encrypted secrets, certificates, per-tenant isolation and observability. What is in it, what it costs to run, and the one thing it does that three FreeBSD boxes or Docker Swarm cannot."
series = "self-hosting"
part = 1
+++
<p>Creating a Kubernetes cluster is a solved problem. What comes after it is not:
delivery, secrets, certificates, isolation between tenants, observability,
backups. That is where the week goes.</p>
<p>So the second cluster is a repository you fork:
<a href="https://github.com/sasin91/hcloud-k3s-platform-kit">hcloud-k3s-platform-kit</a>.
Every number below was measured over four rounds of building and tearing down a
live cluster on Hetzner Cloud: three nodes in Falkenstein, one cpx22 control
plane and two cpx32 agents, behind an lb11 load balancer.</p>
<?= callout_open('note') ?>
<p>Part of <a href="/blog/running-things-on-your-own-hardware/">Where to start with
self-hosting</a>.</p>
<?= callout_close() ?>
<?= section_open('What it is') ?>
<ul>
<li><strong>OpenTofu</strong> for the machines, with
<a href="https://github.com/mysticaltech/terraform-hcloud-kube-hetzner">terraform-hcloud-kube-hetzner</a>
as a pinned dependency. Not a fork of it. That project solves cluster creation
itself, and none of it is rewritten here.</li>
<li><strong>Flux</strong> for delivery, with the controller manifests committed rather than
bootstrapped. No personal access token, and nothing gets written back to your
repo.</li>
<li><strong>SOPS and age</strong> for secrets. Exactly one secret in the whole system is
applied by hand and cannot be encrypted.</li>
<li><strong>Gateway API</strong> for routing, where the platform owns its Gateway and tenants
own their Routes.</li>
<li><strong>VictoriaMetrics, VictoriaLogs and Tempo</strong> behind an OpenTelemetry collector.
The collector is what makes traces worth having, because it stamps
<code>k8s.pod.name</code>, <code>k8s.namespace.name</code> and <code>k8s.node.name</code> onto every span. An
application knows what it is. It does not know where the scheduler put it.</li>
<li><strong>A namespace per tenant</strong>, generated with its quota, its limits, its network
policy, its disruption budget, its backup job, its pod security level and its
RBAC.</li>
</ul>
<?= section_close() ?>

<?= section_open('What it costs to run') ?>
<p><strong>€122.45 a month gross</strong>, including 25% Danish VAT, for the three nodes and the
load balancer above. The kit’s default shape is three control planes and two
agents so there is real quorum, which is €171.17.</p>
<p>The number that matters more is what the platform uses before any of your own
code runs: <strong>4,526 Mi of memory and 194m of CPU across 43 pods</strong>, two thirds of
it observability, against <strong>8 Mi</strong> for the static site it was serving. CPU is a
rounding error at about 2% of allocatable. Memory is what you are paying for,
and the same 4.5 GiB carries both the second tenant and the tenth.</p>
<p>The per-namespace breakdown, the 161,507 active time series, and the two
control-plane histograms that are 27% of them with nobody querying them, are in
<a href="https://github.com/sasin91/hcloud-k3s-platform-kit/blob/main/docs/verification-stats-2026-08-04.md">the repository’s verification
notes</a>.</p>
<?= section_close() ?>

<?= section_open('Where k3s is genuinely stronger') ?>
<p><?= diagram('/images/k3s-platform-kit/spectrum.svg', 'Three options across a spectrum, three FreeBSD boxes, Docker Swarm, and the k3s platform kit, beneath two axes running in opposite directions: components acting on your behalf increase to the right while things you must write yourself decrease, and Swarm sits where they cross.') ?></p>
<p>The FreeBSD end is not bare. Jails are the isolation primitive, bhyve runs a
full guest kernel when a workload needs one, and bastille or cbsd will manage
either. What is missing is the layer above them: nothing schedules across the
three machines, and nothing reconciles them back to a written-down state. One
caveat if you are planning around bhyve on this provider, though. It needs
hardware virtualisation, and Hetzner Cloud exposes none, which is why
<a href="/blog/freebsd-on-hetzner/">installing FreeBSD there</a> runs under TCG emulation.
On these machines you get jails, not virtual machines.</p>
<p>I also run <a href="/blog/athletos-freebsd/">a production app on one FreeBSD
machine</a> for about €7 a month, and I have done the
arithmetic on doing the same with three. Put those machines on the same instance
type and the price difference disappears: three cpx32 is €133.08, more than the
cluster, because the cluster’s control plane is a smaller machine. <strong>The saving
is not in the operating system. It is that the FreeBSD machines fit in 4 GB and
the k3s machines do not.</strong> So the cost argument against Kubernetes is really an
observability argument in disguise. If you would build metrics, logs and traces
anyway, you have already paid.</p>
<p>Docker Swarm fits in the middle: one team running their own apps, which talk to
each other freely. If one thing needs isolating, put it outside the swarm. If
several do, that is where Kubernetes starts to be the better choice, because the
isolation is generated with each namespace and checked when it is applied,
rather than remembered.</p>
<?= section_close() ?>
