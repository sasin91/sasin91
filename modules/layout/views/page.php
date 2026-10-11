<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title><?= out($meta['title']) ?></title>
    <meta name="description" content="<?= out($meta['description']) ?>" />
    <link rel="canonical" href="<?= out($meta['url']) ?>" />

    <?php /* Without these a shared link renders as bare text. og:image is
       left out on any page with no suitable picture (an SVG is one: every
       crawler discards it), since a tag pointing at nothing looks handled
       but is not. */ ?>
    <meta property="og:type" content="<?= out($meta['og_type']) ?>" />
    <meta property="og:site_name" content="<?= out($cv['site']['name']) ?>" />
    <meta property="og:title" content="<?= out($meta['title']) ?>" />
    <meta property="og:description" content="<?= out($meta['description']) ?>" />
    <meta property="og:url" content="<?= out($meta['url']) ?>" />
    <meta property="og:locale" content="en_GB" />
<?php if ($meta['image'] !== null) { ?>
    <meta property="og:image" content="<?= out($meta['image']) ?>" />
<?php if ($meta['image_alt'] !== null) { ?>
    <meta property="og:image:alt" content="<?= out($meta['image_alt']) ?>" />
<?php } ?>
    <meta name="twitter:card" content="summary_large_image" />
<?php } else { ?>
    <meta name="twitter:card" content="summary" />
<?php } ?>
    <link rel="icon" href="/favicon.png" sizes="any">
    <link rel="apple-touch-icon" href="/apple-touch-icon.png">
    <script>
      (function () {
        try {
          var t = localStorage.getItem('theme');
          if (t === 'light' || t === 'dark') {
            document.documentElement.dataset.theme = t;
          }
        } catch (e) {}
      })();
    </script>
    <link rel="stylesheet" href="<?= $site_css_url ?>" />
<?php if ($syntax) { ?>
    <link rel="stylesheet" href="<?= $syntax_css_url ?>" />
<?php } ?>
    <link rel="alternate" type="application/rss+xml" title="<?= out($cv['site']['name']) ?>" href="/rss.xml" />
  </head>
  <body>
    <a class="skip-link" href="#content">Skip to content</a>
    <div class="wrap">
      <header class="site-head">
        <a class="brand" href="/"><?= out($cv['site']['name']) ?></a>
        <nav class="site-nav" aria-label="Main">
          <a href="/blog/"<?= $nav_current['blog'] ? ' aria-current="page"' : '' ?>>Writing</a>
          <a href="/about/"<?= $nav_current['about'] ? ' aria-current="page"' : '' ?>>About</a>
          <a href="/cv/"<?= $nav_current['cv'] ? ' aria-current="page"' : '' ?>>CV</a>
          <a href="<?= out($cv['site']['links']['github']) ?>">GitHub</a>
          <?php /* Opens the search palette. Hidden until the script at the end
             of the page reveals it, so a reader without JavaScript is never
             offered a control that cannot work. */ ?>
          <button type="button" class="site-search" data-search hidden
                  aria-haspopup="dialog" aria-keyshortcuts="Control+K Meta+K Control+Space Alt+Space">
            <?php /* Heroicons v2, 20/solid: magnifying-glass (MIT). */ ?>
            <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" focusable="false">
              <path fill-rule="evenodd" clip-rule="evenodd" d="M9 3.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11ZM2 9a7 7 0 1 1 12.452 4.391l3.328 3.329a.75.75 0 1 1-1.06 1.06l-3.329-3.328A7 7 0 0 1 2 9Z"/>
            </svg>
            <span class="site-search-label">Search</span>
            <kbd class="site-search-key" aria-hidden="true">Ctrl K</kbd>
          </button>
          <?php /* role="switch": this is an on/off state, and "Dark theme,
             switch, off" is what a screen reader should say. The knob and
             icons are pure CSS on the same theme selectors as the syntax
             palette, so the switch is right at first paint. */ ?>
          <button type="button" id="theme-toggle" class="theme-switch"
                  role="switch" aria-checked="false">
            <span class="visually-hidden">Dark theme</span>
            <span class="theme-switch-track" aria-hidden="true">
              <?php /* Heroicons v2, 24/outline (MIT). */ ?>
              <svg class="theme-switch-icon theme-switch-sun" viewBox="0 0 24 24" fill="none"
                   stroke="currentColor" stroke-width="1.7" focusable="false">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v2.25m6.364.386-1.591 1.591M21 12h-2.25m-.386 6.364-1.591-1.591M12 18.75V21m-4.773-4.227-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0Z"/>
              </svg>
              <svg class="theme-switch-icon theme-switch-moon" viewBox="0 0 24 24" fill="none"
                   stroke="currentColor" stroke-width="1.7" focusable="false">
                <path stroke-linecap="round" stroke-linejoin="round" d="M21.752 15.002A9.72 9.72 0 0 1 18 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 0 0 3 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 0 0 9.002-5.998Z"/>
              </svg>
              <span class="theme-switch-knob"></span>
            </span>
          </button>
        </nav>
      </header>

      <main id="content" tabindex="-1">
<?= $content ?>
<?php if ($lightbox) { ?>
<?= $this->view('lightbox', [], true) ?>
<?php } ?>
      </main>

      <footer class="site-foot">
        <span>&copy; <?= $year ?> <?= out($cv['site']['name']) ?></span>
        <nav class="site-foot-links" aria-label="Projects">
          <a href="https://athletos.app">AthletOS</a>
          <a href="https://trongate.cloud">trongate.cloud</a>
        </nav>
        <a href="/rss.xml">RSS</a>
      </footer>
    </div>
    <script>
      (function () {
        var button = document.getElementById('theme-toggle');
        if (!button) return;
        var root = document.documentElement;

        function current() {
          return root.dataset.theme ||
            (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
        }

        function sync() {
          button.setAttribute('aria-checked', String(current() === 'dark'));
        }

        sync();

        button.addEventListener('click', function () {
          var next = current() === 'dark' ? 'light' : 'dark';

          function apply() {
            root.dataset.theme = next;
            try { localStorage.setItem('theme', next); } catch (e) {}
            sync();
          }

          // Same-document view transition: the crossfade of a page transition
          // without the reload, so scroll position needs no handling at all.
          if (document.startViewTransition &&
              !matchMedia('(prefers-reduced-motion: reduce)').matches) {
            document.startViewTransition(apply);
          } else {
            apply();
          }
        });
      })();
    </script>
    <script>
      // The search palette, on every page. This is only the trigger: the
      // palette itself (/search/palette.js) and its index and model are
      // fetched the first time a reader opens it, so reading costs nothing.
      (function () {
        // A post's Ask box. The module only wires the form up; its index and
        // model are fetched once the reader focuses the field.
        if (document.querySelector('form[data-post-ask]')) import('/search/ask.js');

        var button = document.querySelector('[data-search]');
        if (!button) return;
        if (/Mac|iPhone|iPad/.test(navigator.platform)) {
          button.querySelector('kbd').textContent = '⌘K';
        }
        button.hidden = false;

        var loading = null;
        function open() {
          loading = loading || import('/search/palette.js');
          loading.then(function (palette) { palette.open(); }, function () { loading = null; });
        }
        button.addEventListener('click', open);

        // Ctrl/⌘+K everywhere; Ctrl+Space and Alt+Space where the OS lets
        // the browser see them; "/" when not typing in a field.
        addEventListener('keydown', function (event) {
          var typing = /^(INPUT|TEXTAREA|SELECT)$/.test(event.target.tagName) ||
            event.target.isContentEditable;
          var wanted =
            ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') ||
            ((event.ctrlKey || event.altKey) && event.code === 'Space') ||
            (event.key === '/' && !typing && !event.ctrlKey && !event.metaKey && !event.altKey);
          if (!wanted || document.querySelector('dialog.palette[open]')) return;
          event.preventDefault();
          open();
        });
      })();
    </script>
  </body>
</html>
