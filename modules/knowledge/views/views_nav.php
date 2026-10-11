<nav class="view-nav" aria-label="Views of this article">
<?php foreach ($views as $view) { ?>
  <a href="<?= out($view['url']) ?>"<?= $view['current'] ? ' aria-current="page"' : '' ?>><?= $view['label'] ?></a>
<?php } ?>
</nav>
