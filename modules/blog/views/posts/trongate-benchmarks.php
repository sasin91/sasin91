+++
path = "blog/trongate/benchmarks"
title = "trongate.cloud: you push, we deploy your Trongate app"
date = 2026-10-04
description = "Hosting for Trongate apps on Kubernetes. You push, we deploy: HTTPS, a database and room to grow, with nothing to configure. Tailored for Trongate first, proven by running our own control panel on it, and backed by benchmarks."
ask = [
    "How do I deploy my app?",
    "Do I need to manage servers?",
    "Does it scale with me?",
    "Is it built for Trongate?",
    "Is it a proven platform?",
    "What did the benchmarks find?",
    "What is the lazy session fix?",
]
+++
<?= callout_open('note') ?>
<p>trongate.cloud is still a work in progress at the time of writing, 4 October
2026.</p>
<?= callout_close() ?>
<p>You push, we deploy. <a href="https://trongate.cloud">trongate.cloud</a> hosts
<a href="/blog/trongate/">Trongate</a> apps on Kubernetes, with HTTPS, a database and
room to grow, and nothing to configure.</p>
<?= section_open('Simple, stable, and scales with you') ?>
<?= section_open('You push, we deploy', 3) ?>
<p>Connect your GitHub repository, then push. Your app is built and deployed,
live over HTTPS at <code>{slug}.trongate.dev</code>.</p>
<?= section_close() ?>

<?= section_open('Nothing to manage', 3) ?>
<p>No servers and no Dockerfile. Each project gets its own MariaDB database. Each team gets its own Kubernetes namespace with network
policies, so other tenants cannot reach in.</p>
<?= section_close() ?>

<?= section_open('Scales with you', 3) ?>
<p>No size to pick, and CPU is not capped. We watch what real apps use, and a
Trongate app’s throughput follows the CPU it gets almost one for one.</p>
<?= section_close() ?>

<?= section_close() ?>

<?= section_open('Built for Trongate') ?>
<p>Tailored for Trongate first. Trongate apps are detected and built without configuration, and <code>BASE_URL</code>
and <code>ENV</code> in <code>config.php</code> are set for you. The app you run locally is the app
that runs here.</p>
<p>Stability matters to Trongate developers; the framework’s own <a href="https://trongate.io/essays/trongates-new-versioning-system">essay on its
versioning system</a>
explains why. trongate.cloud is built with the same priority: few moving
parts, and Trongate measured as Trongate rather than as generic PHP.</p>
<?= section_close() ?>

<?= section_open('A proven platform: we run it ourselves') ?>
<p>The trongate.cloud control panel is a Trongate app, and it runs on
trongate.cloud, on the same platform as every customer app.</p>
<?= section_close() ?>

<?= section_open('Benchmarks') ?>
<p>The benchmarks use a stock Trongate v2 app, built exactly the way customer
apps are, measured on the platform on 2026-10-04:</p>
<ul>
<li><strong>2,582 requests a second</strong> over HTTPS through the public gateway, at 1 CPU.</li>
<li><strong>p99 of about 29 ms</strong> at 1 CPU.</li>
<li><strong>No evictions</strong> under the heaviest run, with the cluster at 78% CPU.</li>
<li><strong>Throughput follows CPU</strong>, from 1,347 requests a second on half a CPU to
7,765 on three.</li>
</ul>
<p><?= diagram('/images/trongate-benchmarks/cpu-limit.svg', 'Requests per second by CPU limit: raw PHP and Trongate both scale almost linearly from 0.5 to 3 CPU') ?></p>
<?= section_close() ?>

<?= section_open('For the Trongate team: the session finding') ?>
<p>Measuring the platform also turned up one thing in Trongate itself, and the
data below supports a lazy session pull request to
<a href="https://github.com/trongate/trongate-framework">trongate-framework</a>, pending
review.</p>
<p>Stock <code>engine/ignition.php</code> calls <code>session_start()</code> on every request. On a
public host, every bot, health check and first visit writes a session file,
and PHP’s session garbage collection then scans a directory that only grows.
Stock p99 climbed to about 1 s after roughly 100,000 session files, and a
returning visitors run on the same pod right after dropped to 1,089 requests
a second because of the pile-up. The lazy session runs used a fresh pod.</p>
<p>The lazy session fix starts a session only when the visitor already has a
session cookie, or when the request writes to <code>$_SESSION</code>:</p>
<?= code_open('php', 'ignition.lazy.php') ?>
if (isset($_COOKIE[session_name()])) {
    session_start();
} else {
    $_SESSION = [];
    ob_start();
    register_shutdown_function(function (): void {
        if (empty($_SESSION) || session_status() === PHP_SESSION_ACTIVE) return;
        if (headers_sent()) return; // logged in the real patch
        $data = $_SESSION;
        session_start();
        $_SESSION = array_merge($_SESSION, $data);
    });
}
<?= code_close() ?>
<table>
<thead>
<tr>
<th>1 CPU</th>
<th>Stock Trongate</th>
<th>Lazy sessions</th>
</tr>
</thead>
<tbody>
<tr>
<td>New visitors</td>
<td>1,378, p99 up to about 1 s</td>
<td>3,464, p99 26 ms, flat</td>
</tr>
<tr>
<td>Returning visitors</td>
<td>1,089, after the pile-up</td>
<td>2,995</td>
</tr>
</tbody>
</table>
<p><?= diagram('/images/trongate-benchmarks/sessions.svg', 'Sessions before and after, at 1 CPU: new visitors from 1,378 to 3,464 requests a second, returning visitors from 1,089 to 2,995') ?></p>
<p>12 behaviour checks pass with and without the fix: cookies, flashdata across
a redirect, CSRF, a write after 100 KB of output, and no leaked locks.
Starting the session from <code>header_register_callback</code> instead leaked the
session lock under FrankenPHP, so that version was dropped.</p>
<p>Session stores, with lazy sessions at 1 CPU, returning visitors: 2,995
requests a second with files, 1,542 with Valkey (phpredis, persistent) and
913 with Memcached (extension defaults). On Kubernetes, files live with one
pod and are lost when it is replaced, which logs visitors out; a network store
keeps sessions across redeploys and replicas. Without phpredis’
<code>?persistent=1</code>, Valkey dropped to about 250, a new TCP connection per
request.</p>
<p>Requests a second by CPU, for reference:</p>
<table>
<thead>
<tr>
<th>CPU</th>
<th>Raw PHP</th>
<th>Trongate, returning visitors</th>
</tr>
</thead>
<tbody>
<tr>
<td>0.5</td>
<td>2,098</td>
<td>1,347</td>
</tr>
<tr>
<td>1</td>
<td>4,520</td>
<td>3,031</td>
</tr>
<tr>
<td>2</td>
<td>8,710</td>
<td>5,935</td>
</tr>
<tr>
<td>3</td>
<td>11,446</td>
<td>7,765</td>
</tr>
</tbody>
</table>
<p>Trongate did 62 to 68% of raw PHP here. The Trongate team’s <a href="https://trongate.io/benchmarks">published
benchmark</a> puts it at 89% (13,965 against
15,772), but its tool, hardware and concurrency are not published, so the two
are not comparable.</p>
<p>Method: Trongate v2 at
<a href="https://github.com/trongate/trongate-framework">cc968cf</a> with a hello-world
module, built with Railpack 0.40.1 for FrankenPHP with PHP 8.4.26 and OPcache
on, plus a raw PHP file with the same response. <code>wrk</code> with a 5 s warm-up, then
3 runs of 20 s: on the cluster 3 threads and 48 connections from the other
node, locally 2 threads and 32 connections pinned to their own core. “New
visitors” send no cookie, as <code>wrk</code> and <code>ab</code> do by default; “returning
visitors” rotate 1000 session cookies with a Lua script. CPU figures are pod
CPU limits set for the runs. At half a CPU, CFS throttling added a p99 tail
of 55 to 70 ms. Railpack’s Caddyfile logs at <code>DEBUG</code>; <code>INFO</code> gave 23% more
requests a second locally and 7% on the cluster at 0.5 CPU. Reproduce it all
with the <a href="https://github.com/sasin91/trongate-bench">benchmark
kit</a>.</p>
<?= section_close() ?>

<?= section_open('Try it') ?>
<p>Push a Trongate app to <a href="https://trongate.cloud">trongate.cloud</a>.</p>
<p>trongate.cloud is an independent project and is not affiliated with or
endorsed by the Trongate team.</p>
<?= section_close() ?>
