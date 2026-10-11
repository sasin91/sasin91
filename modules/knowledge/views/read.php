  <article class="prose knowledge">
    <h1><?= out($article['title']) ?></h1>
    <p class="lede"><?= out($article['description']) ?></p>
    <div class="meta">
      <time datetime="<?= $article['published_iso'] ?>"><?= $article['published_long'] ?></time>
<?php if ($article['was_updated']) { ?>
        · updated <time datetime="<?= $article['updated_iso'] ?>"><?= $article['updated_long'] ?></time>
<?php } ?>
      · <?= $node_count ?> sections
    </div>

<?= $this->view('views_nav', ['view_module' => 'knowledge', 'views' => $views], true) ?>

<?php foreach ($article['parts'] as $part) { ?>
      <section class="knowledge-part">
        <h2 id="<?= $part['id'] ?>"><?= out($part['title']) ?></h2>

<?php foreach ($part['nodes'] as $node) { ?>
          <?php /* The id is on the whole section, so a deep link scrolls to
             the heading and :target can mark the node being linked to. */ ?>
          <section class="node" id="<?= $node['id'] ?>">
            <div class="node-head">
              <h3><?= out($node['title']) ?></h3>
              <a class="node-anchor" href="#<?= $node['id'] ?>">
                <span aria-hidden="true">#</span>
                <span class="visually-hidden">Link to this section: <?= out($node['title']) ?></span>
              </a>
            </div>

            <?= $node['body'] ?>

<?php if ($node['has_misconceptions']) { ?>
              <div class="node-misreadings">
                <p>Often misread as:</p>
                <ul>
<?php foreach ($node['misconceptions'] as $belief) { ?>
                    <li>“<?= out($belief) ?>”</li>
<?php } ?>
                </ul>
              </div>
<?php } ?>

<?php if ($node['has_links']) { ?>
              <p class="node-links">
<?php if ($node['has_builds_on']) { ?>
                  <span>Builds on
                    <?php foreach ($node['builds_on'] as $link) { ?><a href="#<?= $link['id'] ?>"><?= out($link['title']) ?></a><?= $link['separator'] ?><?php } ?></span>
<?php } ?>
<?php if ($node['has_see_also']) { ?>
                  <span>See also
                    <?php foreach ($node['see_also'] as $link) { ?><a href="#<?= $link['id'] ?>"><?= out($link['title']) ?></a><?= $link['separator'] ?><?php } ?></span>
<?php } ?>
              </p>
<?php } ?>
          </section>
<?php } ?>
      </section>
<?php } ?>

    <p class="knowledge-coda">
      Looking for one idea in particular? <a href="<?= $article['url'] ?>ask/">Ask this
      article</a> finds the closest sections using the same kind of retrieval
      described in <a href="#rag">Retrieval-augmented generation</a>, running on
      your device.
    </p>
  </article>
