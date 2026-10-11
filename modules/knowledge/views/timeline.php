  <h1><?= out($article['title']) ?></h1>
  <p class="lede">
    The whole article as a progression of ideas, one line each. Every step links
    to its full section.
  </p>

<?= $this->view('views_nav', ['view_module' => 'knowledge', 'views' => $views], true) ?>

  <?php /* Generated from the same nodes and the same order as the article.
     One ordered list per part, with `start` carrying the numbering across
     parts so screen readers announce the step numbers that are shown. */ ?>
<?php foreach ($article['parts'] as $part) { ?>
    <section class="timeline-part">
      <h2><?= out($part['title']) ?></h2>
      <ol class="timeline" start="<?= $part['nodes'][0]['order'] ?>">
<?php foreach ($part['nodes'] as $node) { ?>
          <li>
            <a href="<?= $article['url'] ?>#<?= $node['id'] ?>">
              <span class="timeline-title"><?= out($node['title']) ?></span>
              <span class="timeline-summary"><?= out($node['summary']) ?></span>
            </a>
          </li>
<?php } ?>
      </ol>
    </section>
<?php } ?>
