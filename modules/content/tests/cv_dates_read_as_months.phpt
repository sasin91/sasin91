--TEST--
CV dates read as months, and a malformed one names the role it is in
--FILE--
<?php
require __DIR__ . '/setup.inc';
$cv = Cv_data::load(APPPATH . 'content/cv.toml');
echo $cv['roles'][0]['start_label'], "\n";
var_dump(str_contains(implode(' ', array_merge(...array_column($cv['roles'], 'achievements_html'))), '<span class="cv-jump cv-jump-six">6</span>'));

$dir = fixture_dir('cv');
$toml = str_replace('start = "2024-09"', 'start = "2024-13"', file_get_contents(APPPATH . 'content/cv.toml'));
file_put_contents("$dir/cv.toml", $toml);
try {
    Cv_data::load("$dir/cv.toml");
} catch (RuntimeException $error) {
    echo $error->getMessage(), "\n";
}
?>
--EXPECTF--
September 2024
bool(true)
%s: invalid date "2024-13" (want YYYY-MM)
