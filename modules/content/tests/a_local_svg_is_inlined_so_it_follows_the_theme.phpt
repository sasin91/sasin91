--TEST--
a local SVG is inlined so it follows the theme; remote and raster images stay images; a missing SVG is an error
--FILE--
<?php
require __DIR__ . '/setup.inc';
$dir = fixture_dir('svg');
mkdir("$dir/images");
file_put_contents("$dir/images/diagram.svg", '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>');
$assets = new Markdown_html($dir);

$html = $assets->render("![A diagram](/images/diagram.svg)\n");
var_dump(str_contains($html, '<figure class="diagram" role="img" aria-label="A diagram"><svg'));
var_dump(str_contains($html, '<img'));

$kept = $assets->render("![remote](https://example.com/x.svg)\n![raster](/images/y.png)\n");
var_dump(substr_count($kept, '<img'));

try {
    $assets->render("![gone](/images/nope.svg)\n");
    echo "rendered\n";
} catch (RuntimeException $error) {
    echo $error->getMessage(), "\n";
}
?>
--EXPECT--
bool(true)
bool(false)
int(2)
missing SVG referenced by post: /images/nope.svg
