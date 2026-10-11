--TEST--
the CV PDF is a well-formed PDF, the same bytes every time, linking back to the online CV
--FILE--
<?php
require __DIR__ . '/setup.inc';
$pdf = Cv_pdf::render($cv);
var_dump(str_starts_with($pdf, '%PDF-1.4'), str_ends_with($pdf, "%%EOF\n"));
var_dump($pdf === Cv_pdf::render($cv));
var_dump(str_contains($pdf, '/URI (https://sasin91.xyz/cv/)'));

// Every offset in the cross-reference table points at its object.
preg_match('/startxref\n(\d+)/', $pdf, $start);
preg_match_all('/^(\d{10}) 00000 n $/m', substr($pdf, (int) $start[1]), $offsets);
$bad = 0;
foreach ($offsets[1] as $index => $offset) {
    if (!str_starts_with(substr($pdf, (int) $offset), ($index + 1) . ' 0 obj')) {
        $bad++;
    }
}
var_dump(count($offsets[1]) > 5, $bad);
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
int(0)
