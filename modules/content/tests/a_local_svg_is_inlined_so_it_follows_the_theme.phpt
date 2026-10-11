--TEST--
diagram() inlines a local SVG so it follows the theme, and a missing one is an error
--FILE--
<?php
require __DIR__ . '/setup.inc';
$html = diagram('/images/freebsd-on-hetzner/header.svg', 'A disk layout');
var_dump(str_starts_with($html, '<figure class="diagram" role="img" aria-label="A disk layout"><svg'));
try {
    diagram('/images/nope.svg', 'gone');
} catch (RuntimeException $error) {
    echo $error->getMessage(), "\n";
}
?>
--EXPECT--
bool(true)
missing SVG referenced by post: /images/nope.svg
