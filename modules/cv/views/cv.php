  <article class="cv">
    <p class="cv-download"><a href="<?= $pdf_url ?>">Download as PDF</a></p>

    <header class="cv-header">
      <h1><?= out($cv['site']['name']) ?></h1>
      <p class="lede"><?= out($cv['site']['title']) ?></p>
      <?php /* Each item is one unbreakable unit carrying its own trailing
         "·", so the line can only wrap between items. */ ?>
      <p class="cv-contact sep-list">
        <span><?= out($cv['contact']['town']) ?>, <?= out($cv['contact']['postcode']) ?></span>
        <span><a href="tel:<?= out($cv['contact']['phone_href']) ?>"><?= out($cv['contact']['phone']) ?></a></span>
        <span><a href="mailto:<?= out($cv['contact']['email']) ?>"><?= out($cv['contact']['email']) ?></a></span>
        <span><a href="<?= out($cv['site']['links']['github']) ?>">GitHub</a></span>
        <span><a href="<?= out($cv['site']['links']['linkedin']) ?>">LinkedIn</a></span>
      </p>
    </header>

<?php foreach ($cv['intro'] as $paragraph) { ?>
      <p><?= out($paragraph) ?></p>
<?php } ?>

    <h2>Experience</h2>
<?php foreach ($cv['roles'] as $role) { ?>
      <article class="cv-role">
        <h3><?= out($role['title']) ?> · <?= out($role['company']) ?></h3>
        <p class="meta">
          <time datetime="<?= $role['start'] ?>"><?= $role['start_label'] ?></time>
<?php if ($role['end'] !== null) { ?>
            – <time datetime="<?= $role['end'] ?>"><?= $role['end_label'] ?></time>
<?php } else { ?>
            – present
<?php } ?>
          — <?= out($role['location']) ?>
        </p>
        <p><?= out($role['summary']) ?></p>
<?php if ($role['achievements'] !== []) { ?>
          <ul>
<?php foreach ($role['achievements_html'] as $achievement) { ?>
              <li><?= $achievement ?></li>
<?php } ?>
          </ul>
<?php } ?>
<?php if ($role['note'] !== null) { ?>
          <p class="cv-note"><?= out($role['note']) ?></p>
<?php } ?>
      </article>
<?php } ?>

    <h2>Skills</h2>
    <ul class="cv-skills">
<?php foreach ($cv['skills'] as $skill) { ?>
        <li><?= out($skill['name']) ?></li>
<?php } ?>
    </ul>

    <h2>Education</h2>
<?php if ($cv['education_note'] !== null) { ?>
      <p class="lede"><?= out($cv['education_note']) ?></p>
<?php } ?>
<?php foreach ($cv['education'] as $education) { ?>
      <article class="cv-role">
        <h3><?= out($education['title']) ?> · <?= out($education['school']) ?></h3>
        <p class="meta">
          <time datetime="<?= $education['start'] ?>"><?= $education['start_label'] ?></time>
<?php if ($education['end'] !== null) { ?>
            – <time datetime="<?= $education['end'] ?>"><?= $education['end_label'] ?></time>
<?php } else { ?>
            – present
<?php } ?>
          — <?= out($education['location']) ?>
        </p>
<?php if ($education['note'] !== null) { ?>
          <p><?= out($education['note']) ?></p>
<?php } ?>
      </article>
<?php } ?>
  </article>
