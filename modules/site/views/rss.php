<?= '<?xml version="1.0" encoding="UTF-8"?>' ?>

<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title><?= out($cv['site']['name'], 'xml') ?></title>
    <link><?= $base ?></link>
    <description><?= out($cv['site']['title'], 'xml') ?></description>
    <language>en</language>
    <lastBuildDate><?= $last_build_date ?></lastBuildDate>
    <atom:link href="<?= $base ?>/rss.xml" rel="self" type="application/rss+xml" />
<?php foreach ($posts as $post) { ?>
    <item>
      <title><?= out($post['title'], 'xml') ?></title>
      <link><?= $base . out($post['url'], 'xml') ?></link>
      <guid><?= $base . out($post['url'], 'xml') ?></guid>
      <pubDate><?= $post['date_rfc2822'] ?></pubDate>
      <description><?= out($post['description'], 'xml') ?></description>
    </item>
<?php } ?>
  </channel>
</rss>
