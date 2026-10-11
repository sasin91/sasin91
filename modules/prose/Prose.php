<?php
/**
 * Prose Module - the parts a post is written with.
 *
 * Reached through the helper functions in prose_helper.php, the way the form
 * helpers reach the form module: section_open() and section_close(),
 * callout_open() and callout_close(), code_open() and code_close(),
 * diagram() and math(). Every method takes one $data array, so none can be
 * called from a URL.
 */

if (!defined('BASE_URL')) {
    exit('No direct script access allowed');
}

require_once APPPATH . 'modules/content/Highlighter.php';

use Latex2MathML\Converter as Latex2MathML;

class Prose extends Trongate {

    /** Ids handed out in the page being rendered; see reset_ids(). */
    private static array $used_ids = [];

    /** Code blocks opened and not yet closed, innermost last. */
    private static array $open_code = [];

    /**
     * Forgets the ids handed out so far. Called before each page renders, so
     * ids repeat only within one page.
     */
    public function reset_ids(array $data): void {
        self::$used_ids = [];
    }

    /**
     * Opens a section: a heading and everything up to section_close(), in a
     * <section> carrying the heading's id so it can be linked to and styled
     * as one.
     *
     * @param array $data 'title' (HTML allowed), 'level' (2 by default).
     */
    public function section_open(array $data): string {
        $level = $data['level'] ?? 2;
        $id = $this->unique_id($data['title']);
        return '<section id="' . $id . '">' . "\n<h{$level}>" . $data['title'] . "</h{$level}>\n";
    }

    public function section_close(array $data): string {
        return "</section>\n";
    }

    /**
     * Opens an aside set apart from the text.
     *
     * @param array $data 'kind' (note or warning), and optionally 'title',
     *                    linkable like a section's.
     */
    public function callout_open(array $data): string {
        $kind = $data['kind'];
        if (!in_array($kind, ['note', 'warning'], true)) {
            throw new InvalidArgumentException("a callout is a note or a warning, not \"{$kind}\"");
        }

        $html = '<div class="' . $kind . "\">\n";
        $title = $data['title'] ?? null;
        if ($title !== null) {
            $html .= '<h2 id="' . $this->unique_id($title) . '">' . $title . "</h2>\n";
        }
        return $html;
    }

    public function callout_close(array $data): string {
        return "</div>\n";
    }

    /**
     * Starts capturing a code block: everything the view outputs until
     * code_close() is the code, written as it is.
     *
     * @param array $data 'language', and optionally 'title', shown as a file
     *                    name bar above the code.
     */
    public function code_open(array $data): string {
        self::$open_code[] = $data;
        ob_start();
        return '';
    }

    /**
     * The captured code, escaped and highlighted.
     */
    public function code_close(array $data): string {
        if (self::$open_code === []) {
            throw new LogicException('code_close() without a code_open()');
        }
        $block = array_pop(self::$open_code);

        // The code starts on the line after code_open() and ends on the line
        // before code_close(); those two line breaks are not part of it.
        $block['code'] = preg_replace('/^\r?\n/', '', ob_get_clean());
        return $this->code($block);
    }

    /**
     * A code block given whole, escaped and highlighted.
     *
     * @param array $data 'language', 'code', and optionally 'title'.
     */
    public function code(array $data): string {
        $code = rtrim($data['code']) . "\n";
        $highlighted = (new Highlighter())->to_html($code, $data['language']);
        $title = $data['title'] ?? null;
        if ($title === null) {
            return $highlighted . "\n";
        }
        return '<div class="codeblock"><div class="filename">' . htmlspecialchars($title, ENT_QUOTES) . '</div>'
            . $highlighted . "</div>\n";
    }

    /**
     * A local SVG, inlined so its colours follow the theme toggle.
     *
     * @param array $data 'src' (under public/) and 'alt'.
     */
    public function diagram(array $data): string {
        $file = APPPATH . 'public/' . ltrim($data['src'], '/');
        if (!is_file($file)) {
            throw new RuntimeException('missing SVG referenced by post: ' . $data['src']);
        }
        return '<figure class="diagram" role="img" aria-label="' . htmlspecialchars($data['alt'], ENT_QUOTES) . '">'
            . file_get_contents($file) . '</figure>';
    }

    /**
     * TeX as MathML, which browsers render natively. A formula that does not
     * convert shows as its source, marked, rather than breaking the page.
     *
     * @param array $data 'tex', and 'display' (true for a formula on its own line).
     */
    public function math(array $data): string {
        $display = ($data['display'] ?? false) ? 'block' : 'inline';
        try {
            return Latex2MathML::convert($data['tex'], $display);
        } catch (Throwable $error) {
            return '<code class="math-error" title="' . htmlspecialchars($error->getMessage(), ENT_QUOTES) . '">'
                . htmlspecialchars($data['tex'], ENT_QUOTES) . '</code>';
        }
    }

    /**
     * The id the published pages always had: the heading's letters, digits,
     * hyphens and underscores, each run of whitespace a hyphen, everything
     * else dropped. A repeat in the same page gets -1, -2 after it.
     */
    private function unique_id(string $title_html): string {
        $text = html_entity_decode(strip_tags($title_html));
        $kept = preg_replace('/[^\p{L}\p{N}_\-\s]/u', '', $text);
        $id = implode('-', preg_split('/\s+/u', trim($kept), -1, PREG_SPLIT_NO_EMPTY));

        $unique = $id;
        $number = 1;
        while (isset(self::$used_ids[$unique])) {
            $unique = $id . '-' . $number;
            $number++;
        }
        self::$used_ids[$unique] = true;
        return $unique;
    }

}
