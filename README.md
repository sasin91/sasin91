## Jonas Hansen

Software developer in Slagelse, Denmark. I have worked with PHP and Laravel
since 2015, and have helped build a ticketing agency, a video streaming
platform and a recruitment platform along the way. I like designing and
delivering solid, tailored solutions — and I do not mind being the one who
listens to what users are actually struggling with.

**Available for work** — [jonas.kerwin.hansen@gmail.com](mailto:jonas.kerwin.hansen@gmail.com)

Right now I am building [**AthletOS**](https://athletos.app): a training app
that prescribes the weights and records what you actually lifted next to what
it asked for. Rust backend, one FreeBSD box, two jails behind Caddy.

On the side, [**trongate.cloud**](https://trongate.cloud): hosting for
[Trongate](https://trongate.io) apps on Kubernetes. You push, we deploy. Still
a work in progress.

To dogfood it, [**Recruiter**](https://recruiter.trongate.dev): a Trongate app
running on trongate.cloud. Its core is `cv_match`: paste or upload a job post,
and it matches your CV against each requirement, first through a Danish/English
skills taxonomy and then a language model for what the taxonomy can't decide,
and drafts the application and a tailored résumé from the result.

### Writing

I write occasionally at [sasin91.xyz](https://sasin91.xyz), mostly about things
that broke and what fixed them.

- [FreeBSD on a Hetzner Cloud VPS](https://sasin91.xyz/blog/freebsd-on-hetzner) — installing 15.1 on root-on-ZFS unattended, from media you can verify, and the five things that went wrong
- [One box, two jails and a five second deploy](https://sasin91.xyz/blog/athletos-freebsd) — what the boring infrastructure choices bought, measured rather than estimated
- [trongate.cloud: you push, we deploy your Trongate app](https://sasin91.xyz/blog/trongate/benchmarks/) — hosting for Trongate apps, with the benchmarks behind it
- [Recruiter: see how your CV matches a job post](https://sasin91.xyz/blog/trongate/recruiter/) — matching a CV against every requirement in a job post, and the numbers behind it
- [Trongate PHP](https://sasin91.xyz/blog/trongate) — a framework that breaks with common standards, and why that turns out to be interesting

### Tools

PHP · Laravel · Symfony · Go · Rust · Nuxt · React · Kubernetes · Linux · FreeBSD

---

<sub>This repository is the source of [sasin91.xyz](https://sasin91.xyz) — a
[Trongate](https://trongate.io) app, every post a plain PHP view,
hosted on [trongate.cloud](https://trongate.cloud). See
[docs/development.md](docs/development.md) to run it.</sub>

<details>
<summary><sub>Developing the site</sub></summary>

PHP 8.4 with `intl` and `mbstring`, and Composer:

```sh
composer install
php -S 127.0.0.1:8000 -t public bin/dev-router.php
```

Checks, as CI runs them:

```sh
php bin/run-tests.php modules/content/tests modules/search/tests
php bin/search-index.php > build/search-index.json
node --test modules/search/js/   # Node 22+, nothing to install
```

Posts live in `modules/blog/views/posts/`, the CV in `modules/cv/cv.toml`, and
the AI article's knowledge nodes in `modules/knowledge/` — see
[docs/knowledge-nodes.md](docs/knowledge-nodes.md).

</details>

<sub>[sasin91.xyz/cv](https://sasin91.xyz/cv/) is itself the source of the
downloadable [cv_jonas_hansen_software_developer.pdf](https://sasin91.xyz/cv_jonas_hansen_software_developer.pdf):
both are rendered from the same `modules/cv/cv.toml`, so the content cannot
diverge. The CV is edited in `cv.toml`, never in a PDF editor.</sub>
