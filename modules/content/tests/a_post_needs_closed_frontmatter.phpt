--TEST--
a post needs a +++ header that is closed
--FILE--
<?php
require __DIR__ . '/setup.inc';
foreach (['just a body', "+++\ntitle = \"x\"\n"] as $source) {
    try {
        Front_matter::split($source, 'x.md');
    } catch (RuntimeException $error) {
        echo $error->getMessage(), "\n";
    }
}
?>
--EXPECT--
x.md: file must start with a +++ frontmatter block
x.md: frontmatter block is never closed
