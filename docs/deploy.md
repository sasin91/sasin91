# Deploying sasin91.xyz

CI builds the site on Linux and ships it to the FreeBSD box that also runs
athletos.app. The site builder binary never runs on the server.

## How a deploy lands

Deploys do not mutate the directory Caddy is serving. Each one goes into a new
release directory and becomes live by moving a symlink:

```
/usr/local/www/sasin91.xyz/          <- zroot/www, its own dataset
  releases/r20260728-102606-931db83/
  releases/r20260727-181422-a1b2c3d/
  current -> releases/r20260728-102606-931db83
```

Caddy's `root` points at `current`. The swap is a rename, so there is no window
in which a visitor sees new HTML referencing a stylesheet that has not arrived
yet, and no reload is needed — verified: the swap takes effect within a second.

`rsync --link-dest` hardlinks unchanged files from the previous release, so the
2.4 MB demo video costs nothing on the wire or on disk after the first deploy.
Three releases are kept.

**Rolling back** is re-pointing the symlink, and needs no rebuild:

```sh
ssh deploy@athletos.app
cd /usr/local/www/sasin91.xyz
ls releases                       # pick the one you want
ln -sfn releases/<name> current.tmp && mv -h current.tmp current
```

`ln -sfn` alone is not atomic — it unlinks before it links. The temporary name
plus `mv -h` is, and `-h` moves the symlink rather than following it.

## Why not a jail

The site is static: no code runs to isolate. Caddy serves it from the host and
already serves athletos.app from there. A jail would either isolate nothing
(host Caddy still reads the files) or require a second web server and a proxy
hop for four HTML files. The AthletOS jails exist because they run application
code and need blue/green rollout; neither applies here.

## Why its own ZFS dataset

`/usr/local/www` is `zroot/www`, not part of `zroot/ROOT/default`. Web content
should not live inside a boot environment — a BE rollback to fix an OS problem
would otherwise rewind the site too. This matches how `pgdata`, `backups` and
`jails` are already separated. Compression is lz4, as elsewhere.

The workflow lives at `.github/workflows/deploy.yml`. It runs on every push
to `main` (and on demand via `workflow_dispatch`, restricted to the `main`
ref), and the `deploy` job `needs:` a `check` job that runs `cargo fmt
--check`, `cargo clippy -- -D warnings` and `cargo test` first — none of
those may fail. It also refuses to deploy if either build-time guard fails:
any `<script` tag found under `public/`, or any of the seven required URLs
missing from the build output.

## Repository secrets

Four secrets must exist under the repository's Settings -> Secrets and
variables -> Actions before the workflow can deploy:

- `DEPLOY_SSH_KEY` — a private key whose public half is in the deploy user's
  `authorized_keys` on the FreeBSD box.
- `DEPLOY_HOST` — the box's hostname or IP.
- `DEPLOY_USER` — the deploy user, e.g. `deploy`.
- `DEPLOY_KNOWN_HOSTS` — the output of `ssh-keyscan <host>`, so the workflow
  can verify the host key without prompting.

## Caddy

Add alongside the existing athletos.app block in `/usr/local/etc/caddy/Caddyfile`:

```caddy
sasin91.xyz, www.sasin91.xyz {
	encode zstd gzip
	root * /usr/local/www/sasin91.xyz

	# The old Laravel routes had no trailing slash, and the generator writes
	# <path>/index.html. This resolves /blog/trongate without a redirect.
	try_files {path} {path}/ {path}/index.html

	# Stylesheet filenames carry a hash of their own contents, so a given URL
	# can never change what it returns. That is what makes it safe to tell a
	# browser never to ask again -- and it is what stops the revalidation
	# round trip that made every click after a deploy feel slow.
	@immutable path_regexp \.[0-9a-f]{8,}\.css$
	header @immutable Cache-Control "public, max-age=31536000, immutable"

	# Everything else changes at the same URL on every deploy, so it must be
	# revalidated rather than cached blind. `no-cache` still allows a 304 off
	# the ETag, so a revalidation costs headers, not the body.
	header Cache-Control "public, no-cache"

	file_server
}
```

Measured against the live site before this rule existed: Caddy sent no
`Cache-Control` at all, only `ETag`, `Last-Modified` and `Vary`. With no
explicit freshness, browsers fall back to heuristic caching -- roughly 10% of
the age since `Last-Modified`. Immediately after a deploy that age is near
zero, so freshness is near zero, and every navigation revalidated every
stylesheet: a measured 135ms round trip, render-blocking, on every click.
Hours later the heuristic grows and the site quietly feels fast again --
which is why this read as "a bit of latency" that "went back to normal", and
why it would have returned on every subsequent deploy.

The two rules above split the site's assets into two lifetimes:

- **The hashed stylesheets** get `max-age=31536000` (one year) and
  `immutable`: a year because a hashed URL never needs revisiting once
  cached, and `immutable` so the browser does not even bother re-checking it
  on reload. This is only safe *because* the filename changes when the
  content does (see `hash_css` in `src/main.rs`) -- a long `max-age` on a
  fixed name like `site.css` would be a staleness trap, since a cached copy
  would then survive a deploy that changed the CSS underneath it.
- **Everything else** -- HTML, `rss.xml`, `sitemap.xml`, `cv.pdf` -- gets
  `no-cache`. That name is misleading: it does not mean "do not cache", it
  means "revalidate before reuse". The browser still stores the response and
  still sends the `ETag` on the next request, so a fresh visit costs a 304
  and a header round trip, not the full body. That is the right behaviour for
  these files specifically because they change at the same URL on every
  deploy -- `/`, `/cv.pdf`, `/rss.xml` never get a new name the way the
  stylesheets do, so blind caching them would show a stale page just as
  surely as a long `max-age` on `site.css` would.

Reload without dropping connections:

```sh
caddy reload --config /usr/local/etc/caddy/Caddyfile
```

## One-time setup

```sh
mkdir -p /usr/local/www/sasin91.xyz
chown deploy:deploy /usr/local/www/sasin91.xyz
```

## Verifying a deploy

Trailing-slash resolution is Caddy's behaviour and cannot be verified from
`public/` alone. After a deploy, check the live URLs:

```bash
for u in / /about/ /blog /blog/trongate /blog/trongate/mx-transition \
         /blog/freebsd-on-hetzner /blog/athletos-freebsd /rss.xml; do
  printf "%-40s %s\n" "$u" "$(curl -s -o /dev/null -w '%{http_code}' https://sasin91.xyz$u)"
done
```

Expected: `200` for every line, except `/blog` and `/blog/trongate` — those
two are deliberately requested *without* their trailing slash, and
`try_files {path} {path}/ {path}/index.html` matches them on the `{path}/`
candidate, i.e. as a directory. Caddy's `file_server` canonicalizes a
directory match by redirecting to the trailing-slash URL, so `308` is the
expected response there too (this follows from Caddy's documented
directory-canonicalization behaviour; it has not been exercised against a
running Caddy instance, so treat it as expected, not confirmed). A `308` is
fine to accept as a pass: browsers and search engines follow it
automatically, and the site's own internal links, RSS and sitemap all
already use trailing slashes, so only inbound legacy links (old bookmarks,
stale search results) ever hit the redirect. If a redirect-free response is
wanted instead, reorder the directive to
`try_files {path}/index.html {path} {path}/` so the index file is served
directly before the bare path is tried. A `404` on any line still means the
`try_files` directive is wrong or missing.
