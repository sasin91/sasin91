--TEST--
passages are the top-level blocks as plain text, in their sections, without code; a diagram reads as its description; a colon joins the next
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = render_view(<<<'VIEW'
<p>First <strong>paragraph</strong> with <a href="#x">a link</a> and <code>code</code>.</p>
<?= section_open('A heading') ?>
<p>Like this:</p>
<ul>
<li>one</li>
<li>two</li>
</ul>
<?= code_open('bash') ?>
hidden
<?= code_close() ?>
<p><?= diagram('/images/freebsd-on-hetzner/header.svg', 'Alt words') ?></p>
<?= callout_open('note') ?>
<p>Inside a callout.</p>
<?= callout_close() ?>
<?= section_close() ?>
VIEW);
echo json_encode(Prose_view::passages($html), JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), "\n";
?>
--EXPECT--
[
    {
        "text": "First paragraph with a link and code.",
        "section": null
    },
    {
        "text": "Like this: one two",
        "section": {
            "id": "A-heading",
            "heading": "A heading"
        }
    },
    {
        "text": "Alt words",
        "section": {
            "id": "A-heading",
            "heading": "A heading"
        }
    },
    {
        "text": "Inside a callout.",
        "section": {
            "id": "A-heading",
            "heading": "A heading"
        }
    }
]
