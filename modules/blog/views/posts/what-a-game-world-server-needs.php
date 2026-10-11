+++
path = "blog/what-a-game-world-server-needs"
title = "What a game world server actually needs"
date = 2026-08-04
series = "self-hosting"
part = 2
description = "Hosting a WoW emulator: the world daemon's tick runs on one thread, so clock speed buys you everything and core count buys you almost nothing. What that means for what you buy, why the gaming chip is the wrong pick, and the routing gotcha that catches everyone."
+++
<p>WoW emulation has always been interesting to me. Here is what actually matters
if you want to host one, starting with the CPU, because that is where the usual
advice goes wrong.</p>
<p>These projects exist to study game server architecture and network protocol
design, and that is the only use I am discussing. If you run anything, the
legality of it is your own responsibility.</p>
<?= section_open('Start with the CPU') ?>
<p>The world daemon’s tick runs on a single thread. Parallelism exists between maps
and instances but never inside one, and <code>MapManager</code> puts a barrier on every
tick, so the tick costs whatever the slowest individual map costs no matter how
many threads are idle behind it. TrinityCore ships <code>MapUpdate.Threads = 1</code>;
CMaNGOS ships 3.</p>
<p>So core count barely matters and clock speed matters a lot. It is also why a
crowded capital city pins one core while the rest of the machine sits idle.</p>
<p>This comes from the source and the shipped config files, not from profiling a
populated server. Treat it as the design rather than a measurement.</p>
<?= section_close() ?>

<?= section_open('What to buy') ?>
<p>Single-thread rating is the number. PassMark, read on 5 August 2026, and these
drift.</p>
<table>
<thead>
<tr>
<th>Tier</th>
<th>CPU</th>
<th>Single thread</th>
</tr>
</thead>
<tbody>
<tr>
<td>Budget</td>
<td>Ryzen 5 5600</td>
<td>3,254</td>
</tr>
<tr>
<td>Budget</td>
<td>Core i5-12400</td>
<td>3,463</td>
</tr>
<tr>
<td>Midrange</td>
<td>Core i5-13600K</td>
<td>4,112</td>
</tr>
<tr>
<td>Midrange</td>
<td>Ryzen 5 7600</td>
<td>4,130</td>
</tr>
<tr>
<td>Midrange</td>
<td>Ryzen 5 9600X</td>
<td>4,571</td>
</tr>
<tr>
<td>High end</td>
<td>Core i9-14900K</td>
<td>4,690</td>
</tr>
<tr>
<td>High end</td>
<td>Ryzen 9 9950X</td>
<td>4,727</td>
</tr>
<tr>
<td>High end</td>
<td>Core Ultra 9 285K</td>
<td>5,087</td>
</tr>
<tr>
<td>Server</td>
<td>EPYC 9965, 192 cores</td>
<td>3,176</td>
</tr>
</tbody>
</table>
<p>A six-core desktop part beats a 192-core server part here. The EPYC is built for
throughput across many tenants, which is not the problem you have.</p>
<p>The gaming chip is not the pick either. X3D parts trade clock for cache, so the
Ryzen 7 7800X3D scores 3,759 against the plain 7600’s 4,130, and the 5800X3D
3,233 against the 5600’s 3,254. Whether the map update loop wins that back from
the extra L3 is not something I have measured, and I have not found anyone who
has. On the number you can look up, the cheaper non-X3D part is ahead.</p>
<p>A midrange current-generation part is the sweet spot. A Ryzen 5 9600X at 4,571
is within about 10% of the top of that table.</p>
<p>If you would rather rent, Hetzner’s dedicated AX line fits. AX42 is a Ryzen 7
PRO 8700GE (3,864) with 64 GB DDR5 ECC, AX102 a Ryzen 9 7950X3D (4,144) with
128 GB. AX162 is a 48-core EPYC 9454P, which is more machine and less tick.</p>
<?= section_close() ?>

<?= section_open('The rest of the list') ?>
<ul>
<li>16 GB of RAM as the floor. AzerothCore’s memory page says maps are cached as
players explore them and never freed until a restart, ending past 11 GB, and
recommends starting at 16 GB.</li>
<li>Local NVMe rather than a network volume. The characters database writes at
every save interval and punishes remote storage.</li>
<li>Latency over bandwidth. Small packets, frequently, so distance and jitter
decide how the game feels.</li>
<li>Not a cheap shared VPS. I ran servers on those years ago and the symptom
matched the code above: fine most of the time, then not, with cores sitting
there that nothing could spend.</li>
</ul>
<?= section_close() ?>

<?= section_open('The routing gotcha') ?>
<p>The auth daemon and the world daemon are joined by a database table rather than
an API. TrinityCore’s auth database has a <code>realmlist</code> row per realm with
<code>address</code> and <code>port</code>
(<a href="https://github.com/TrinityCore/tc-wiki/blob/main/database/335/auth/realmlist.md">realmlist</a>),
documented as the world server’s public address. The client logs in, gets that
address, and opens a raw TCP connection straight to the world daemon.</p>
<p>So there is no front door doing the routing. The address can be a hostname
rather than an IP, but the client resolves it and opens a raw socket, and there
is no HTTP layer for an ingress to route on. Proxy it at layer 4 or not at all.</p>
<?= section_close() ?>

<?= section_open('Installing it') ?>
<p>CMaNGOS’s <a href="https://github.com/cmangos/issues/wiki/Installation-Instructions">installation
instructions</a>
cover Debian and Ubuntu end to end: the package list, the CMake build, the three
databases and the script that populates them, extracting maps and DBC files from
a game client, and the realm configuration that ties it together. There is no
point in me rewriting that.</p>
<p>Budget most of your time for the data rather than the build. The servers I ran
broke on data far more often than on code, with a world database half-migrated
by whoever was developing against it, DBC files mangled on extraction, or a
missing MPQ archive making the maps come out wrong.</p>
<?= section_close() ?>

<?= section_open('If you already run a cluster') ?>
<p><?= diagram('/images/what-a-game-world-server-needs/split.svg', 'The system split: behind one load balancer, a cluster holding replicated auth daemons and the web front end over a shared database of accounts, characters and guilds; outside it, each world daemon on its own machine with its own read-mostly world database, reaching back into the shared database for characters and guilds.') ?></p>
<p>Everything except the world daemon is ordinary stateless plumbing: the auth
daemon, the website and registration API, item shops and armory pages, Discord
bots, backups and content-database imports, and observability for all of it. That
is a normal small platform and it is the shape
<a href="/blog/k3s-platform-kit/">a k3s kit</a> generates.</p>
<p>The world daemon stays outside. Its state is in RAM and grows until restart, so
restarting a pod restarts the server for everyone logged in. A server that has
finished loading is not interchangeable with the one it replaced, so a rolling
update has nothing to roll to. And the connection is a long-lived raw TCP
session rather than a request.</p>
<p>None of that needs a cloud. k3s is a single binary and runs on hardware you own,
and a homelab suits this split unusually well: the world machine wants a fast
desktop-class core and a local NVMe, which is what a homelab has and what cloud
instances are worst at. Co-located, the cluster reaches the world daemon across
your own switch and the layer 4 passthrough is a port forward rather than a
rented load balancer.</p>
<?= section_close() ?>
