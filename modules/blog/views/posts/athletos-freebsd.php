+++
path = "blog/athletos-freebsd"
title = "One box, two jails and a five second deploy"
date = 2026-07-26
description = "AthletOS runs on a single €7 FreeBSD box. Two jails that are always up, Caddy handling certificates on its own, and a rolling deploy that never leaves zero healthy backends. What the boring choices bought, measured rather than estimated."
hero = "/images/athletos-freebsd/architecture.svg"
hero_alt = "One FreeBSD box: Caddy in front of two always-running jails, with Postgres on the host"
+++
<p>The essence is simple and efficient. That is exactly why I chose
<a href="/blog/freebsd-on-hetzner/">FreeBSD</a> as the foundation and
<a href="https://github.com/sasin91/athletos.app/tree/main/backend">Rust</a> as the base.</p>
<blockquote>
<p>AthletOS picks a program, prescribes the weights, and records what you
actually lifted next to what it asked for.</p>
<p>The gap between those two numbers is the entire product. I do not need a
motivational chart, I need to be told 100 kg and not talk myself into 120.</p>
</blockquote>
<p>One Hetzner CX23, about €7 a month. FreeBSD 15.1, root-on-ZFS, two jails, Caddy,
Postgres.</p>
<?= section_open('Why not Docker') ?>
<p>While Docker is great, well supported and convenient, it is also additional
complexity and overhead that I do not need. With
<a href="/blog/freebsd-on-hetzner/">FreeBSD</a> I have everything I need to ensure smooth
rollouts and consistent performance with <em>minimal</em> overhead and tooling.</p>
<?= section_close() ?>

<?= section_open('Rollouts') ?>
<p>Two jails, blue and green, both running all the time and both registered with
Caddy. A deploy takes one out, swaps a symlink, waits for it to answer, and only
then touches the other.</p>
<p><?= diagram('/images/athletos-freebsd/deploy-timeline.svg', 'Rolling deploy timeline: five seconds, always at least one healthy backend') ?></p>
<p>Five seconds, and never a moment without a healthy backend. The same machinery
covers a crash at 3am: Caddy marks the jail down and the other one serves until
I look.</p>
<?= callout_open('note', 'Migrations') ?>
<p>Two releases are briefly live against one database, so a migration has to be
backward compatible with the one before it.</p>
<?= callout_close() ?>
<?= section_close() ?>

<?= section_open('Certificates') ?>
<p>Certificate lifetimes are dropping to <strong>47 days</strong>, which turns renewal into eight
chances a year for a hook to fail quietly. Caddy just does it. No certbot, no
timer, nothing to forget.</p>
<?= code_open('caddy', 'Caddyfile') ?>
{$APP_DOMAIN} {
	encode zstd gzip
	reverse_proxy 10.0.0.2:3000 10.0.0.3:3000 {
		lb_policy       least_conn
		health_uri      /login
		health_interval 5s
	}
}
<?= code_close() ?>
<?= section_close() ?>

<?= section_open('Efficiency') ?>
<p><?= diagram('/images/athletos-freebsd/footprint.svg', 'Memory footprint: 2.19 GB free of 3.86 GB with everything running') ?></p>
<p>The efficiency speaks for itself. Plenty of room — likely more than I will ever
need for this app.</p>
<p>The API answers in under a millisecond. Everything else in the numbers below is
the distance to Helsinki and a fresh handshake per sample.</p>
<?= code_open('bash', 'latency, from my desk in Denmark') ?>
api, loopback inside the box      p50  0.0 ms   p95  0.6 ms
https://api.athletos.app/health   p50  136 ms   p95  143 ms
https://athletos.app/login        p50  136 ms   p95  247 ms
<?= code_close() ?>
<?= section_close() ?>

<?= section_open('Backups') ?>
<p>The backup story is refreshingly stable and efficient, with
<a href="https://github.com/jimsalterjrs/sanoid">sanoid</a> taking snapshots on a timer and
syncoid piggybacking on ZFS to move them offsite. Jails ensure a reproducible
and reliable runtime environment. There is little left for me to desire at this
scale.</p>
<?= code_open('bash', 'zfs list -t snapshot') ?>
zroot/data/pgdata@autosnap_2026-07-26_17:24:00_hourly
zroot/data/pgdata@autosnap_2026-07-26_17:24:00_daily
zroot/data/pgdata@autosnap_2026-07-26_17:24:00_monthly
<?= code_close() ?>
<?= section_close() ?>

<?= section_open('Was it worth it') ?>
<p>Absolutely. Stable, reliable and predictable. I can trust this to keep running.</p>
<?= callout_open('warning', 'What broke on first contact') ?>
<p>Five things broke on a real box. Four were ordinary. The fifth was worth
remembering: <code>${name}_user</code> is a reserved <code>rc.subr</code> variable, and <code>rc.subr</code>
wraps your command in <code>su -m</code> before it runs. My rc script named a variable
<code>athletos_api_user</code> thinking it was mine, so <code>daemon(8)</code> was already running as
that user when it tried to drop privileges to that user. Two privilege drops
fighting, and <code>-r</code> retried it forever. The error it prints is “failed to set
user environment”, which sounds like <code>login.conf</code> and is not.</p>
<?= callout_close() ?>
<?= callout_open('note', 'Building on the target') ?>
<p>CI builds inside a FreeBSD VM and runs the test suite there before producing a
binary. Rust is a Tier 2 platform on FreeBSD — the project builds it but does
not test it — so building and testing on the release you actually deploy to is
doing work nobody upstream does for you.</p>
<?= callout_close() ?>
<?= callout_open('warning', 'ARC eats your RAM') ?>
<p>ARC takes half your RAM unless told otherwise. On a 4 GB box that is 2 GB it
does not need, and it shows up later looking like a database problem. Put
<code>vfs.zfs.arc_max=&quot;512M&quot;</code> in <code>loader.conf</code>, on day one.</p>
<?= callout_close() ?>
<p class="colophon">Every number above was measured on the box, not estimated.</p>
<?= section_close() ?>
