<?php
require_once __DIR__ . '/Pdf_metrics.php';

/**
 * A minimal PDF 1.4 writer: pages of positioned text in Helvetica and
 * Helvetica-Bold, and link annotations. Nothing else, which is all the CV
 * needs, and small enough to read end to end.
 *
 * The two fonts are among the standard 14 every PDF reader carries, so
 * nothing is embedded and the file stays a few kilobytes. Text is encoded as
 * WinAnsi, which covers Danish and the typographic punctuation the CV uses.
 */
class Pdf_writer {

    const POINTS_PER_MM = 72.0 / 25.4;

    const HELVETICA = 'Helvetica';
    const HELVETICA_BOLD = 'Helvetica-Bold';

    /** Unicode characters WinAnsi places in 0x80-0x9F, by code point. */
    const WINANSI_HIGH = [
        0x20AC => 0x80, 0x201A => 0x82, 0x0192 => 0x83, 0x201E => 0x84, 0x2026 => 0x85,
        0x2020 => 0x86, 0x2021 => 0x87, 0x02C6 => 0x88, 0x2030 => 0x89, 0x0160 => 0x8A,
        0x2039 => 0x8B, 0x0152 => 0x8C, 0x017D => 0x8E, 0x2018 => 0x91, 0x2019 => 0x92,
        0x201C => 0x93, 0x201D => 0x94, 0x2022 => 0x95, 0x2013 => 0x96, 0x2014 => 0x97,
        0x02DC => 0x98, 0x2122 => 0x99, 0x0161 => 0x9A, 0x203A => 0x9B, 0x0153 => 0x9C,
        0x017E => 0x9E, 0x0178 => 0x9F,
    ];

    /**
     * The width of $text set in $font at $size_pt, in points, kerning
     * included.
     */
    public static function width(string $font, string $text, float $size_pt): float {
        $widths = self::widths($font);
        $bytes = self::winansi_bytes($text);
        $count = count($bytes);

        $thousandths = 0;
        for ($i = 0; $i < $count; $i++) {
            $thousandths += $widths[$bytes[$i] - 32];
            if ($i + 1 < $count) {
                $thousandths += self::kern($font, $bytes[$i], $bytes[$i + 1]);
            }
        }
        return $thousandths / 1000 * $size_pt;
    }

    /**
     * The whole file.
     *
     * $pages is a list of pages, each a list of placements:
     * ['x_mm', 'y_mm', 'size_pt', 'font', 'text', 'link' (a URI or null)],
     * with y measured up from the bottom of the page, as PDF does.
     */
    public static function write(string $title, float $width_mm, float $height_mm, array $pages): string {
        if ($pages === []) {
            $pages = [[]];
        }

        $width = $width_mm * self::POINTS_PER_MM;
        $height = $height_mm * self::POINTS_PER_MM;

        // Objects 1-5 are fixed; each page is then a page object followed
        // by its content stream.
        $first_page_object = 6;
        $kids = [];
        foreach (array_keys($pages) as $index) {
            $kids[] = ($first_page_object + $index * 2) . ' 0 R';
        }

        $objects = [
            1 => '<< /Type /Catalog /Pages 2 0 R /Lang (en) >>',
            2 => '<< /Type /Pages /Kids [' . implode(' ', $kids) . '] /Count ' . count($pages) . ' >>',
            3 => '<< /Title ' . self::text_string($title) . ' /Producer (sasin91.xyz) >>',
            4 => '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
            5 => '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>',
        ];

        foreach ($pages as $index => $placements) {
            $page_object = $first_page_object + $index * 2;
            $contents_object = $page_object + 1;

            $content = '';
            $annots = [];
            foreach ($placements as $placement) {
                $content .= self::placement_operators($placement);
                if ($placement['link'] !== null) {
                    $annots[] = self::link_annotation($placement);
                }
            }

            $annots_entry = '';
            if ($annots !== []) {
                $annots_entry = ' /Annots [' . implode(' ', $annots) . ']';
            }

            $objects[$page_object] = sprintf(
                '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 %.2F %.2F] '
                . '/Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> '
                . '/Contents %d 0 R%s >>',
                $width, $height, $contents_object, $annots_entry
            );
            $objects[$contents_object] = '<< /Length ' . strlen($content) . " >>\nstream\n" . $content . "\nendstream";
        }

        return self::serialise($objects);
    }

    private static function placement_operators(array $placement): string {
        $font_name = $placement['font'] === self::HELVETICA_BOLD ? '/F2' : '/F1';
        $operators = sprintf(
            'BT %s %.2F Tf 1 0 0 1 %.2F %.2F Tm ',
            $font_name,
            $placement['size_pt'],
            $placement['x_mm'] * self::POINTS_PER_MM,
            $placement['y_mm'] * self::POINTS_PER_MM
        );
        return $operators . self::tj_array($placement['font'], $placement['text']) . " TJ ET\n";
    }

    /**
     * A clickable rectangle over the placed text: from the baseline's left
     * end to the text's width, and from a little below the baseline to the
     * top of the capitals.
     */
    private static function link_annotation(array $placement): string {
        $size = $placement['size_pt'];
        $x0 = $placement['x_mm'] * self::POINTS_PER_MM;
        $baseline = $placement['y_mm'] * self::POINTS_PER_MM;
        $x1 = $x0 + self::width($placement['font'], $placement['text'], $size);
        $y0 = $baseline - 0.25 * $size;
        $y1 = $baseline + 0.85 * $size;

        return sprintf(
            '<< /Type /Annot /Subtype /Link /Rect [%.2F %.2F %.2F %.2F] /Border [0 0 0] /A << /S /URI /URI (%s) >> >>',
            $x0, $y0, $x1, $y1, self::escape_literal($placement['link'])
        );
    }

    /**
     * Numbers the objects into the file, then writes the cross-reference
     * table that gives each one's byte offset.
     */
    private static function serialise(array $objects): string {
        ksort($objects);
        $out = "%PDF-1.4\n%\xE6\xF8\xE5\xB5\n";
        $size = max(array_keys($objects)) + 1;
        $offsets = array_fill(0, $size, 0);

        foreach ($objects as $number => $body) {
            $offsets[$number] = strlen($out);
            $out .= $number . " 0 obj\n" . $body . "\nendobj\n";
        }

        // The ID is a hash of everything before it, so the same CV gives the
        // same file, byte for byte.
        $id = strtoupper(hash('fnv1a64', $out));
        $xref_start = strlen($out);

        $out .= "xref\n0 {$size}\n";
        $out .= "0000000000 65535 f \n";
        for ($number = 1; $number < $size; $number++) {
            $out .= sprintf("%010d 00000 n \n", $offsets[$number]);
        }
        $out .= "trailer\n<< /Size {$size} /Root 1 0 R /Info 3 0 R /ID [<{$id}> <{$id}>] >>\n"
            . "startxref\n{$xref_start}\n%%EOF\n";
        return $out;
    }

    /**
     * Text as a TJ array, split wherever a kerning pair applies so the
     * adjustment can sit between the two characters: [(T) 74 (o)].
     */
    private static function tj_array(string $font, string $text): string {
        $bytes = self::winansi_bytes($text);
        $count = count($bytes);
        if ($count === 0) {
            return '[()]';
        }

        $parts = [];
        $chunk = '';
        for ($i = 0; $i < $count; $i++) {
            $chunk .= chr($bytes[$i]);
            $is_last = $i + 1 === $count;
            $kern = 0;
            if (!$is_last) {
                $kern = self::kern($font, $bytes[$i], $bytes[$i + 1]);
            }
            if ($is_last || $kern !== 0) {
                $parts[] = '(' . self::escape_literal($chunk) . ')';
                if ($kern !== 0) {
                    $parts[] = (string) -$kern;
                }
                $chunk = '';
            }
        }
        return '[' . implode(' ', $parts) . ']';
    }

    private static function widths(string $font): array {
        if ($font === self::HELVETICA_BOLD) {
            return Pdf_metrics::HELVETICA_BOLD;
        }
        return Pdf_metrics::HELVETICA;
    }

    private static function kern(string $font, int $left, int $right): int {
        $table = Pdf_metrics::HELVETICA_KERN;
        if ($font === self::HELVETICA_BOLD) {
            $table = Pdf_metrics::HELVETICA_BOLD_KERN;
        }
        return $table[($left << 8) | $right] ?? 0;
    }

    /**
     * UTF-8 text as WinAnsi bytes; a character WinAnsi lacks becomes "?".
     *
     * @return int[]
     */
    private static function winansi_bytes(string $text): array {
        $bytes = [];
        foreach (mb_str_split($text) as $char) {
            $bytes[] = self::winansi_byte(mb_ord($char));
        }
        return $bytes;
    }

    private static function winansi_byte(int $code): int {
        if (isset(self::WINANSI_HIGH[$code])) {
            return self::WINANSI_HIGH[$code];
        }
        if ($code >= 0x20 && $code < 0x7F) {
            return $code;
        }
        if ($code >= 0xA0 && $code < 0x100) {
            return $code;
        }
        return ord('?');
    }

    private static function escape_literal(string $bytes): string {
        return addcslashes($bytes, '()\\');
    }

    /**
     * A string for the document information dictionary: a literal when it is
     * ASCII, otherwise UTF-16BE with a byte order mark, as hex.
     */
    private static function text_string(string $value): string {
        if (mb_check_encoding($value, 'ASCII')) {
            $printable = preg_replace('/[\x00-\x1F\x7F]/', '', $value);
            return '(' . self::escape_literal($printable) . ')';
        }
        return '<FEFF' . strtoupper(bin2hex(mb_convert_encoding($value, 'UTF-16BE', 'UTF-8'))) . '>';
    }

}
