# Deploying sasin91.xyz

The site runs on [trongate.cloud](https://trongate.cloud) as one Trongate app:
one container, built from this repository, with no database. trongate.cloud's
GitHub App builds and rolls out every push to `main`; there is no deploy
workflow here. `.github/workflows/pipeline.yml` checks a push is fit to go
(tests, every page answering, no stray scripts), and a red check is the signal
not to merge.

## What the build needs

- PHP 8.4 with `intl` and `mbstring`, both declared in `composer.json`, so the
  builder installs them.
- `composer install`, which puts the four content libraries in `packages/`
  (Trongate's convention for Composer's vendor directory).
- Nothing else. `content/`, `models/` and `public/` are read as they are in the
  repository; there is no build output.

## What happens at runtime

- Every page renders from `content/` on request. The slowest is `/blog/ai/`,
  which renders all its knowledge nodes, in under 0.1 s.
- The search index is built the first time it is asked for after a deploy
  changes the content, and kept in the container's temp directory. A new
  container builds it once, in well under a second, on its first search.
- `BASE_URL` follows the request's host (`config/config.php`), so the app
  answers on its preview address and on sasin91.xyz alike. Canonical links,
  `og:url`, the feed and the sitemap always name `https://sasin91.xyz`
  (`SITE_URL`), whichever address served them.
- `ENV` is `prod` unless `APP_ENV` says otherwise or the request comes from
  the machine itself, so Trongate's developer tools never answer the internet.

## Moving the domain

sasin91.xyz is still served by the static Rust build on the FreeBSD box (see
this file on `main`) until its DNS points at trongate.cloud. To move it, add
sasin91.xyz as a custom domain on the app in trongate.cloud, point the DNS
record where it says, and check the live URLs:

```sh
for path in / /blog/ /blog/trongate/ /blog/trongate/mx-transition/ \
            /blog/freebsd-on-hetzner/ /blog/athletos-freebsd/ /about/ /cv/ \
            /blog/ai/ /blog/ai/timeline/ /blog/ai/ask/ \
            /cv_jonas_hansen_software_developer.pdf /rss.xml; do
  printf '%-45s %s\n' "$path" "$(curl -sL -o /dev/null -w '%{http_code}' "https://sasin91.xyz$path")"
done
```

Every line should say 200. The pages answer with and without their trailing
slash; the canonical link names the trailing-slash form, as the static site's
redirects did.
