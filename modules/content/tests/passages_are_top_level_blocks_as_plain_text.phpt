--TEST--
passages are the top-level blocks as plain text, in their sections, without code, and a colon joins the next
--FILE--
<?php
require __DIR__ . '/setup.inc';
$source = "First *paragraph* with [a link](#x) and `code`.\n\n"
    . "## A heading\n\nLike this:\n\n- one\n- two\n\n```bash\nhidden\n```\n\n![Alt words](/x.png)\n\n"
    . "::: note\nInside a div.\n:::\n";
echo json_encode($djot->passages($source), JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), "\n";
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
        "text": "Inside a div.",
        "section": {
            "id": "A-heading",
            "heading": "A heading"
        }
    }
]
