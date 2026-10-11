  <h1>About</h1>
  <p class="lede"><?= out($cv['site']['title']) ?>. <?= out($cv['site']['available_note']) ?>.</p>

  <?php /* `about` is written for a reader arriving cold. `intro` is the CV's
     own wording and renders on /cv instead, so the two pages do not repeat
     each other: one is a person, the other is a record. */ ?>
<?php foreach ($cv['about'] as $paragraph) { ?>
    <p><?= out($paragraph) ?></p>
<?php } ?>

  <p><a href="/cv/">Read the full CV</a>, or take the <a href="/cv_jonas_hansen_software_developer.pdf">PDF</a>.</p>
