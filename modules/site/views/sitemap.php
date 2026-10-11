<?= '<?xml version="1.0" encoding="UTF-8"?>' ?>

<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc><?= $base ?>/</loc></url>
  <url><loc><?= $base ?>/about/</loc></url>
  <url><loc><?= $base ?>/cv/</loc></url>
  <url><loc><?= $base ?>/blog/</loc></url>
  <url>
    <loc><?= $base . $article['url'] ?></loc>
    <lastmod><?= $article['updated_iso'] ?></lastmod>
  </url>
  <url>
    <loc><?= $base . $article['url'] ?>timeline/</loc>
    <lastmod><?= $article['updated_iso'] ?></lastmod>
  </url>
<?php foreach ($posts as $post) { ?>
  <url>
    <loc><?= $base . out($post['url'], 'xml') ?></loc>
    <lastmod><?= $post['date_iso'] ?></lastmod>
  </url>
<?php } ?>
</urlset>
