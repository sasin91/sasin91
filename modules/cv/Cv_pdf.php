<?php
require_once __DIR__ . '/Pdf_writer.php';

/**
 * The CV as an A4 PDF, laid out from the same content/cv.toml as /cv/, so
 * the two can never carry different content.
 *
 * Layout is a cursor moving down the page: each block of lines is placed at
 * the cursor, and a block that must stay together (a role, with its heading)
 * moves to a new page whole rather than splitting across two.
 */
class Cv_pdf {

    const PAGE_W_MM = 210.0;
    const PAGE_H_MM = 297.0;
    const MARGIN_X_MM = 16.0;
    const MARGIN_Y_MM = 18.0;
    const COLUMN_MM = self::PAGE_W_MM - 2 * self::MARGIN_X_MM;

    const BULLET_INDENT_MM = 5.0;
    const BULLET_HANGING_MM = 3.2;

    const ONLINE_CV = 'https://sasin91.xyz/cv/';

    /** @var array<int, array> placements per page */
    private array $pages = [[]];
    private float $y_mm = self::PAGE_H_MM - self::MARGIN_Y_MM;

    public static function render(array $cv): string {
        $layout = new self();
        $layout->lay_out($cv);
        $title = $cv['site']['name'] . " \u{2014} CV";
        return Pdf_writer::write($title, self::PAGE_W_MM, self::PAGE_H_MM, $layout->pages);
    }

    private function lay_out(array $cv): void {
        $this->place($this->lines($cv['site']['name'], Pdf_writer::HELVETICA_BOLD, 20, 24));
        $this->place($this->lines($cv['site']['title'], Pdf_writer::HELVETICA, 11, 15));

        $contact = $cv['contact']['town'] . ', ' . $cv['contact']['postcode']
            . " \u{b7} " . $cv['contact']['phone'] . " \u{b7} " . $cv['contact']['email'];
        $this->place($this->lines($contact, Pdf_writer::HELVETICA, 9, 13));

        $online_cv = $this->lines('Full CV online: sasin91.xyz/cv', Pdf_writer::HELVETICA, 9, 13);
        $online_cv[0]['link'] = self::ONLINE_CV;
        $this->place($online_cv);

        $links = $cv['site']['links']['github'] . " \u{b7} " . $cv['site']['links']['linkedin'];
        $this->place($this->lines($links, Pdf_writer::HELVETICA, 9, 13));
        $this->gap($this->pt(8));

        foreach ($cv['intro'] as $paragraph) {
            $this->place($this->lines($paragraph, Pdf_writer::HELVETICA, 10, 14));
            $this->gap($this->pt(4));
        }

        $this->experience($cv['roles']);
        $this->skills($cv['skills']);
        $this->education($cv['education'], $cv['education_note']);
    }

    private function experience(array $roles): void {
        $heading = $this->section_heading('Experience');
        if ($roles === []) {
            $this->place($heading);
            return;
        }

        foreach ($roles as $index => $role) {
            $block = $this->role_block($role);
            if ($index === 0) {
                $block = $this->keep_with_heading($heading, $this->pt(3), $block);
            }
            $this->place_together($block);
            $this->gap($this->pt(6));
        }
    }

    private function skills(array $skills): void {
        $heading = $this->section_heading('Skills');
        if ($skills === []) {
            $this->place($heading);
            return;
        }

        foreach ($skills as $index => $skill) {
            $block = $this->bullet($skill['name']);
            if ($index === 0) {
                $this->place_together($this->keep_with_heading($heading, $this->pt(3), $block));
                continue;
            }
            $this->place($block);
        }
    }

    private function education(array $educations, ?string $note): void {
        $heading = $this->section_heading('Education');

        if ($note !== null) {
            $note_block = $this->lines($note, Pdf_writer::HELVETICA, 10, 14);
            $this->place_together($this->keep_with_heading($heading, $this->pt(3), $note_block));
            $this->gap($this->pt(4));
            foreach ($educations as $education) {
                $this->place_together($this->education_block($education));
                $this->gap($this->pt(6));
            }
            return;
        }

        if ($educations === []) {
            $this->place($heading);
            return;
        }

        foreach ($educations as $index => $education) {
            $block = $this->education_block($education);
            if ($index === 0) {
                $block = $this->keep_with_heading($heading, $this->pt(3), $block);
            }
            $this->place_together($block);
            $this->gap($this->pt(6));
        }
    }

    private function role_block(array $role): array {
        return $this->entry(
            $role['title'] . " \u{b7} " . $role['company'],
            $this->meta($role['start_label'], $role['end_label'], $role['location']),
            $role['summary'],
            $role['achievements'],
            $role['note']
        );
    }

    private function education_block(array $education): array {
        return $this->entry(
            $education['title'] . " \u{b7} " . $education['school'],
            $this->meta($education['start_label'], $education['end_label'], $education['location']),
            $education['note'],
            [],
            null
        );
    }

    /** "September 2024 – February 2026 — Copenhagen" */
    private function meta(string $start, ?string $end, string $location): string {
        return $start . " \u{2013} " . ($end ?? 'present') . " \u{2014} " . $location;
    }

    /**
     * One role or education: its heading, the dates line, the summary, the
     * achievements as bullets and a closing note set a little apart.
     */
    private function entry(string $heading, string $meta_line, ?string $body, array $bullets, ?string $note): array {
        $block = $this->lines($heading, Pdf_writer::HELVETICA_BOLD, 10.5, 14);
        $block = array_merge($block, $this->lines($meta_line, Pdf_writer::HELVETICA, 8.5, 12));
        if ($body !== null) {
            $block = array_merge($block, $this->lines($body, Pdf_writer::HELVETICA, 10, 14));
        }
        foreach ($bullets as $bullet) {
            $block = array_merge($block, $this->bullet($bullet));
        }
        if ($note !== null) {
            $note_lines = $this->lines($note, Pdf_writer::HELVETICA, 9, 13);
            if ($note_lines !== []) {
                $note_lines[0]['leading_mm'] += $this->pt(3);
            }
            $block = array_merge($block, $note_lines);
        }
        return $block;
    }

    private function bullet(string $text): array {
        return $this->lines("\u{2022}  " . $text, Pdf_writer::HELVETICA, 10, 14,
            self::BULLET_INDENT_MM, self::BULLET_HANGING_MM);
    }

    private function section_heading(string $title): array {
        $this->gap($this->pt(10));
        return $this->lines($title, Pdf_writer::HELVETICA_BOLD, 11, 16);
    }

    /**
     * A heading and the first block under it as one block, so a heading is
     * never left alone at the bottom of a page.
     */
    private function keep_with_heading(array $heading, float $gap_mm, array $block): array {
        if ($block !== []) {
            $block[0]['leading_mm'] += $gap_mm;
        }
        return array_merge($heading, $block);
    }

    /**
     * Text wrapped to the column, as lines: each carries its own leading (the
     * distance from the previous baseline) and indent. Continuation lines of
     * a bullet hang under its text rather than its bullet.
     */
    private function lines(string $text, string $font, float $size_pt, float $leading_pt,
                           float $indent_mm = 0.0, float $hanging_mm = 0.0): array {
        $wrapped = $this->wrap($text, $font, $size_pt, self::COLUMN_MM - $indent_mm - $hanging_mm);

        $lines = [];
        foreach ($wrapped as $index => $line_text) {
            $hang = $index === 0 ? 0.0 : $hanging_mm;
            $lines[] = [
                'text' => $line_text,
                'font' => $font,
                'size_pt' => $size_pt,
                'leading_mm' => $this->pt($leading_pt),
                'indent_mm' => $indent_mm + $hang,
                'link' => null,
            ];
        }
        return $lines;
    }

    /** Greedy word wrap against the font's real widths. */
    private function wrap(string $text, string $font, float $size_pt, float $width_mm): array {
        $limit = $width_mm * Pdf_writer::POINTS_PER_MM;
        $words = preg_split('/\s+/u', trim($text), -1, PREG_SPLIT_NO_EMPTY);

        $lines = [];
        $line = '';
        foreach ($words as $word) {
            if ($line === '') {
                $line = $word;
                continue;
            }
            $candidate = $line . ' ' . $word;
            if (Pdf_writer::width($font, $candidate, $size_pt) <= $limit) {
                $line = $candidate;
                continue;
            }
            $lines[] = $line;
            $line = $word;
        }
        if ($line !== '') {
            $lines[] = $line;
        }
        return $lines;
    }

    private function pt(float $points): float {
        return $points / Pdf_writer::POINTS_PER_MM;
    }

    private function remaining_mm(): float {
        return $this->y_mm - self::MARGIN_Y_MM;
    }

    private function break_page(): void {
        $this->pages[] = [];
        $this->y_mm = self::PAGE_H_MM - self::MARGIN_Y_MM;
    }

    /** Vertical space, unless it would run off the page anyway. */
    private function gap(float $mm): void {
        if ($this->remaining_mm() > $mm) {
            $this->y_mm -= $mm;
        }
    }

    /** Places lines one by one, breaking the page wherever one does not fit. */
    private function place(array $lines): void {
        foreach ($lines as $line) {
            if ($this->remaining_mm() < $line['leading_mm']) {
                $this->break_page();
            }
            $this->y_mm -= $line['leading_mm'];
            $this->pages[count($this->pages) - 1][] = [
                'x_mm' => self::MARGIN_X_MM + $line['indent_mm'],
                'y_mm' => $this->y_mm,
                'size_pt' => $line['size_pt'],
                'font' => $line['font'],
                'text' => $line['text'],
                'link' => $line['link'],
            ];
        }
    }

    /**
     * Places a block on one page: when it does not fit what is left, it
     * starts a new page, unless it would not fit a whole page either.
     */
    private function place_together(array $lines): void {
        $height = array_sum(array_column($lines, 'leading_mm'));
        $page_height = self::PAGE_H_MM - 2 * self::MARGIN_Y_MM;
        if ($height > $this->remaining_mm() && $height <= $page_height) {
            $this->break_page();
        }
        $this->place($lines);
    }

}
