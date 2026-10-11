  <h1>Writing</h1>
  <p class="lede">Notes on things I built and what broke on the way.</p>

  <ul class="post-list">
<?php foreach ($entries as $entry) { ?>
      <li>
        <div class="meta">
          <time datetime="<?= $entry['date_iso'] ?>"><?= $entry['date_long'] ?></time>
          <?php if ($entry['label'] !== null) { ?> · <?= out($entry['label']) ?><?php } ?>

        </div>
        <h2><a href="<?= out($entry['url']) ?>"><?= out($entry['title']) ?></a></h2>
        <p><?= out($entry['description']) ?></p>
      </li>
<?php } ?>
  </ul>
