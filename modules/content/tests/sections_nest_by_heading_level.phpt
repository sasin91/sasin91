--TEST--
a heading opens a section that the next heading of the same or a higher level closes
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = $markdown->render("Intro\n\n## A\n\na\n\n### A.1\n\na1\n\n## B\n\nb\n");
echo preg_replace('/\s+/', ' ', strip_tags($html, '<section>')), "\n";
?>
--EXPECT--
Intro <section id="A"> A a <section id="A1"> A.1 a1 </section> </section> <section id="B"> B b </section>
