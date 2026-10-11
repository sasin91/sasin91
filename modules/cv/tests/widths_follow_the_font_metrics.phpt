--TEST--
text widths come from the AFM metrics, kerning included, and non-WinAnsi characters count as "?"
--FILE--
<?php
require __DIR__ . '/setup.inc';
// Helvetica "A" is 667/1000 em, "V" 667, and the AV pair kerns by -70.
echo Pdf_writer::width(Pdf_writer::HELVETICA, 'A', 10), "\n";
echo Pdf_writer::width(Pdf_writer::HELVETICA, 'AV', 10), "\n";
echo Pdf_writer::width(Pdf_writer::HELVETICA, "\u{4E2D}", 10) === Pdf_writer::width(Pdf_writer::HELVETICA, '?', 10) ? "same\n" : "differs\n";
?>
--EXPECT--
6.67
12.64
same
