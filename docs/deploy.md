# Deploying sasin91.xyz

CI builds the site on Linux and rsyncs `public/` to
`/usr/local/www/sasin91.xyz/` on the FreeBSD box that also runs athletos.app.
The site builder binary never runs on the server.

The workflow lives at `.github/workflows/deploy.yml`. It runs on every push
to `main` (and on demand via `workflow_dispatch`), and refuses to deploy if
either guard fails: any `<script` tag found under `public/`, or any of the
seven required URLs missing from the build output.

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
	file_server
}
```

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

Expected: `200` for every line. A `404` on `/blog/trongate` means the
`try_files` directive is wrong or missing.
