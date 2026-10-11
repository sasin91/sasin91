--TEST--
a post needs a +++ header that is closed
--FILE--
<?php
require __DIR__ . '/setup.inc';
foreach (['just a body', "+++\ntitle = \"x\"\n"] as $source) {
    try {
        Front_matter::split($source, 'x.php');
    } catch (RuntimeException $error) {
        echo $error->getMessage(), "\n";
    }
}
?>
--EXPECT--
x.php: file must start with a +++ frontmatter block
x.php: frontmatter block is never closed
