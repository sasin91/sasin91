# sasin91.xyz

A static site: a short CV, and posts about things I built.

Content is [Djot](https://djot.net/) under `content/`. A Rust binary renders it
into `public/`, which is disposable and regenerated on every build. The site
ships no JavaScript.

## Build

```sh
cargo run --release      # writes ./public
```

## Write

```sh
cargo install watchexec-cli               # once
watchexec -e dj,html,css,rs -- cargo run  # rebuild on change
cd public && python -m http.server 8000   # serve, in another shell
```

A post is one `.dj` file under `content/blog/` with a `+++` TOML header. The
`path` key is the URL, and is deliberately not derived from the filename.

## Deploy

Pushing to `main` builds the site in CI and rsyncs it to the FreeBSD box.
See `docs/deploy.md`.

## History

This replaced a Laravel + Inertia + React application. That repo is archived
separately; nothing here shares history with it.
