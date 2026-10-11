  <section class="intro">
    <h1><?= out($cv['site']['name']) ?></h1>
    <p class="lede"><?= out($cv['site']['title']) ?></p>
    <?php /* The newline after each span is the only break opportunity in the
       line; .sep-list glues each generated "·" to the word before it. */ ?>
    <p class="stack sep-list">
      <?php foreach ($cv['site']['stack'] as $item) { ?><span><?= out($item) ?></span>
      <?php } ?>
    </p>
<?php if ($cv['site']['available']) { ?>
      <p class="available">
        <?= out($cv['site']['available_note']) ?> —
        <a href="<?= out($cv['site']['links']['email']) ?>">get in touch</a>
      </p>
<?php } ?>

    <div class="social">
      <a href="<?= out($cv['site']['links']['github']) ?>">GitHub</a>
      <a href="<?= out($cv['site']['links']['linkedin']) ?>">LinkedIn</a>
      <a href="<?= out($cv['site']['links']['email']) ?>">Email</a>
    </div>
  </section>

  <h2>Writing</h2>
  <ul class="post-list">
<?php foreach ($posts as $post) { ?>
      <li>
        <div class="meta">
          <time datetime="<?= $post['date_iso'] ?>"><?= $post['date_long'] ?></time>
        </div>
        <h3><a href="<?= out($post['url']) ?>"><?= out($post['title']) ?></a></h3>
        <p><?= out($post['description']) ?></p>
      </li>
<?php } ?>
  </ul>
