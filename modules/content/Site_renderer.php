<?php
require_once __DIR__ . '/Highlighter.php';

use Djot\Node\Block\CodeBlock;
use Djot\Node\Block\Heading;
use Djot\Node\Inline\Image;
use Djot\Node\Inline\Math;
use Djot\Renderer\HtmlRenderer;
use Latex2MathML\Converter as Latex2MathML;

/**
 * php-collective/djot's HTML renderer with the four things this site does
 * differently, each overriding the one method that renders that node.
 */
class Site_renderer extends HtmlRenderer {

    private string $assets_dir;
    private Highlighter $highlighter;

    public function __construct(string $assets_dir) {
        parent::__construct();
        $this->assets_dir = rtrim($assets_dir, '/');
        $this->highlighter = new Highlighter();
    }

    /** Ids already given to a heading in this document, against reuse. */
    private array $section_ids = [];

    /** The id each heading got, by object, so asking twice gives one answer. */
    private array $ids_by_heading = [];

    /**
     * A heading's id, as the section around it carries it.
     */
    public function section_id(Heading $node): string {
        return $this->getSectionId($node);
    }

    /**
     * The id the published pages were built with, so links into them keep
     * working: the heading's letters, digits, hyphens and underscores, with
     * each run of whitespace a hyphen and everything else dropped. "1. Create
     * the server" is 1-Create-the-server, "Built on trongate.cloud" is
     * Built-on-trongatecloud. A repeat gets -1, -2 after it. The library's
     * own scheme differs (s-1-Create-the-server, Built-on-trongate-cloud).
     * An id the author wrote ({#...}) is used as it is.
     */
    protected function getSectionId(Heading $node): string {
        $key = spl_object_id($node);
        if (isset($this->ids_by_heading[$key])) {
            return $this->ids_by_heading[$key];
        }

        if ($node->hasAttribute('id')) {
            $id = (string) $node->getAttribute('id');
        } else {
            $id = $this->unique_id($this->id_from_text($this->getPlainText($node)));
        }

        $this->section_ids[$id] = true;
        $this->ids_by_heading[$key] = $id;
        return $id;
    }

    private function id_from_text(string $text): string {
        $kept = preg_replace('/[^\p{L}\p{N}_\-\s]/u', '', $text);
        $words = preg_split('/\s+/u', trim($kept), -1, PREG_SPLIT_NO_EMPTY);
        return implode('-', $words);
    }

    private function unique_id(string $id): string {
        if (!isset($this->section_ids[$id])) {
            return $id;
        }
        $number = 1;
        while (isset($this->section_ids[$id . '-' . $number])) {
            $number++;
        }
        return $id . '-' . $number;
    }

    /**
     * Fenced code, highlighted, and framed with its file name when the block
     * has a title attribute: {title="4-remaster.sh"}.
     */
    protected function renderCodeBlock(CodeBlock $node): string {
        $language = $node->getLanguage() ?? '';
        $highlighted = $this->highlighter->to_html($node->getContent(), $language);

        $title = $node->getAttribute('title');
        if ($title === null) {
            return $highlighted . "\n";
        }

        return '<div class="codeblock"><div class="filename">' . $this->escape($title) . '</div>'
            . $highlighted . "</div>\n";
    }

    /**
     * Math as MathML, which every current browser renders natively, rather
     * than markup a script has to find and typeset later.
     */
    protected function renderMath(Math $node): string {
        $tex = $node->getContent();
        $display = $node->isDisplay() ? 'block' : 'inline';

        try {
            return Latex2MathML::convert($tex, $display);
        } catch (Throwable $error) {
            return '<code class="math-error" title="' . $this->escapeAttribute($error->getMessage()) . '">'
                . $this->escape($tex) . '</code>';
        }
    }

    /**
     * A local SVG is inlined in a figure, so its colours can follow the page's
     * theme. Any other image stays an <img>.
     */
    protected function renderImage(Image $node): string {
        $source = $node->getSource();
        if (!self::is_local_svg($source)) {
            return parent::renderImage($node);
        }

        $path = $this->assets_dir . '/' . ltrim($source, '/');
        if (!is_file($path)) {
            throw new RuntimeException('missing SVG referenced by post: ' . $source);
        }

        return self::svg_figure($node->getAlt(), file_get_contents($path));
    }

    public static function is_local_svg(string $source): bool {
        return str_starts_with($source, '/') && str_ends_with($source, '.svg');
    }

    public static function svg_figure(string $alt, string $svg): string {
        return '<figure class="diagram" role="img" aria-label="' . htmlspecialchars($alt, ENT_QUOTES) . '">'
            . $svg . '</figure>';
    }

}
