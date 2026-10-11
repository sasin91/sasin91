<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title><?= out($title) ?></title>
<style>
  @page { margin: 18mm 16mm; }
  body { font-family: Helvetica, Arial, sans-serif; font-size: 10pt; line-height: 1.4; color: #111; }
  h1 { font-size: 20pt; margin: 0; }
  .title { font-size: 11pt; margin: 0 0 4pt; }
  .contact { font-size: 9pt; margin: 0 0 8pt; }
  .contact a { color: inherit; text-decoration: none; }
  h2 { font-size: 11pt; margin: 14pt 0 3pt; page-break-after: avoid; }
  .entry { margin-bottom: 6pt; page-break-inside: avoid; }
  h3 { font-size: 10.5pt; margin: 0; }
  .meta { font-size: 8.5pt; margin: 0 0 2pt; }
  p { margin: 0 0 4pt; }
  ul { margin: 0; padding-left: 5mm; }
  .note { font-size: 9pt; margin-top: 3pt; }
</style>
</head>
<body>
  <h1><?= out($cv['site']['name']) ?></h1>
  <p class="title"><?= out($cv['site']['title']) ?></p>
  <p class="contact">
    <?= out($cv['contact']['town']) ?>, <?= out($cv['contact']['postcode']) ?> · <?= out($cv['contact']['phone']) ?> · <?= out($cv['contact']['email']) ?><br>
    Full CV online: <a href="https://sasin91.xyz/cv/">sasin91.xyz/cv</a><br>
    <?= out($cv['site']['links']['github']) ?> · <?= out($cv['site']['links']['linkedin']) ?>
  </p>

<?php foreach ($cv['intro'] as $paragraph) { ?>
  <p><?= out($paragraph) ?></p>
<?php } ?>

  <h2>Experience</h2>
<?php foreach ($cv['roles'] as $role) { ?>
  <div class="entry">
    <h3><?= out($role['title']) ?> · <?= out($role['company']) ?></h3>
    <p class="meta"><?= $role['start_label'] ?> – <?= $role['end_text'] ?> — <?= out($role['location']) ?></p>
    <p><?= out($role['summary']) ?></p>
<?php if ($role['achievements'] !== []) { ?>
    <ul>
<?php foreach ($role['achievements'] as $achievement) { ?>
      <li><?= out($achievement) ?></li>
<?php } ?>
    </ul>
<?php } ?>
<?php if ($role['note'] !== null) { ?>
    <p class="note"><?= out($role['note']) ?></p>
<?php } ?>
  </div>
<?php } ?>

  <h2>Skills</h2>
  <ul>
<?php foreach ($cv['skills'] as $skill) { ?>
    <li><?= out($skill['name']) ?></li>
<?php } ?>
  </ul>

  <h2>Education</h2>
<?php if ($cv['education_note'] !== null) { ?>
  <p><?= out($cv['education_note']) ?></p>
<?php } ?>
<?php foreach ($cv['education'] as $education) { ?>
  <div class="entry">
    <h3><?= out($education['title']) ?> · <?= out($education['school']) ?></h3>
    <p class="meta"><?= $education['start_label'] ?> – <?= $education['end_text'] ?> — <?= out($education['location']) ?></p>
<?php if ($education['note'] !== null) { ?>
    <p><?= out($education['note']) ?></p>
<?php } ?>
  </div>
<?php } ?>
</body>
</html>
