--TEST--
tokenizes exactly like the reference Hugging Face tokenizer, as the browser does
--FILE--
<?php
require __DIR__ . '/setup.inc';
$cases = json_decode(file_get_contents(APPPATH . 'models/potion-base-4M/wordpiece-fixture.json'), true);
$wrong = 0;
foreach ($cases as $case) {
    if ($model->tokenize($case['text']) !== $case['tokens']) {
        echo 'differs for ', json_encode($case['text']), "\n";
        $wrong++;
    }
}
var_dump(count($cases) >= 10, $wrong);
?>
--EXPECT--
bool(true)
int(0)
