<?php
/**
 * The parts a post is written with, in the style of Trongate's form helpers:
 *
 *   <?= section_open('4. Remaster the ISO') ?>
 *   <p>Three edits …</p>
 *   <?= code_open('bash', '4-remaster.sh') ?>
 *   apt-get install -y xorriso qemu-system-x86
 *   <?= code_close() ?>
 *   <?= callout_open('warning', 'What went wrong') ?>
 *   <p>I replaced <code>loader.conf</code> …</p>
 *   <?= callout_close() ?>
 *   <?= section_close() ?>
 *
 * Each delegates to the prose module (modules/prose/Prose.php).
 */

/**
 * Opens a section: its heading, and a <section> carrying the heading's id.
 *
 * @param string $title The heading, HTML allowed.
 * @param int $level The heading level, h2 by default.
 * @return string The opening tags.
 */
function section_open(string $title, int $level = 2): string {
    return Modules::run('prose/section_open', ['title' => $title, 'level' => $level]);
}

/**
 * @return string The closing tag of the section.
 */
function section_close(): string {
    return Modules::run('prose/section_close', []);
}

/**
 * Opens a note or a warning set apart from the text.
 *
 * @param string $kind note or warning.
 * @param string|null $title An optional heading, linkable like a section's.
 * @return string The opening tags.
 */
function callout_open(string $kind, ?string $title = null): string {
    return Modules::run('prose/callout_open', ['kind' => $kind, 'title' => $title]);
}

/**
 * @return string The closing tag of the callout.
 */
function callout_close(): string {
    return Modules::run('prose/callout_close', []);
}

/**
 * Starts a code block. What follows, up to code_close(), is the code as it is
 * written; it is escaped and highlighted, never run.
 *
 * @param string $language The language to highlight as (bash, php, html …).
 * @param string|null $title An optional file name shown above the code.
 * @return string Nothing; the block is output by code_close().
 */
function code_open(string $language, ?string $title = null): string {
    return Modules::run('prose/code_open', ['language' => $language, 'title' => $title]);
}

/**
 * @return string The code block, highlighted.
 */
function code_close(): string {
    return Modules::run('prose/code_close', []);
}

/**
 * A code block given as a string, for code that contains PHP's own <?
 * tags, which code_open() would run instead of show:
 *
 *   <?= code('php', <<<'CODE'
 *   <?= anchor('account') ?>
 *   CODE, 'header.php') ?>
 *
 * @param string $language The language to highlight as.
 * @param string $code The code.
 * @param string|null $title An optional file name shown above the code.
 * @return string The code block, highlighted.
 */
function code(string $language, string $code, ?string $title = null): string {
    return Modules::run('prose/code', ['language' => $language, 'code' => $code, 'title' => $title]);
}

/**
 * A local SVG, inlined so its colours follow the theme toggle.
 *
 * @param string $src The file's path under public/, e.g. /images/x/y.svg.
 * @param string $alt What it shows, for a reader who cannot see it.
 * @return string The figure.
 */
function diagram(string $src, string $alt): string {
    return Modules::run('prose/diagram', ['src' => $src, 'alt' => $alt]);
}

/**
 * TeX as MathML.
 *
 * @param string $tex The formula.
 * @param bool $display True for a formula on its own line.
 * @return string The MathML.
 */
function math(string $tex, bool $display = false): string {
    return Modules::run('prose/math', ['tex' => $tex, 'display' => $display]);
}
