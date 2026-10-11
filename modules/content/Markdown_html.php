<?php
require_once __DIR__ . '/Markdown_extensions.php';

use League\CommonMark\Environment\Environment;
use League\CommonMark\Extension\CommonMark\CommonMarkCoreExtension;
use League\CommonMark\Extension\CommonMark\Node\Block\FencedCode;
use League\CommonMark\Extension\CommonMark\Node\Block\Heading;
use League\CommonMark\Extension\CommonMark\Node\Block\HtmlBlock;
use League\CommonMark\Extension\CommonMark\Node\Block\IndentedCode;
use League\CommonMark\Extension\CommonMark\Node\Inline\Code;
use League\CommonMark\Extension\CommonMark\Node\Inline\HtmlInline;
use League\CommonMark\Extension\CommonMark\Node\Inline\Image;
use League\CommonMark\Extension\SmartPunct\SmartPunctExtension;
use League\CommonMark\Extension\Table\TableExtension;
use League\CommonMark\Node\Block\AbstractBlock;
use League\CommonMark\Node\Block\Document;
use League\CommonMark\Node\Inline\Newline;
use League\CommonMark\Node\Inline\Text;
use League\CommonMark\Node\Node;
use League\CommonMark\Parser\MarkdownParser;
use League\CommonMark\Renderer\HtmlRenderer;

/**
 * Markdown to HTML, and to the plain-text passages search embeds.
 *
 * CommonMark with GitHub's tables and typographic quotes and dashes, plus the
 * pieces in Markdown_extensions.php (math, highlighted code, inlined SVGs).
 *
 * Each document-level heading opens a <section> carrying its id, closed by
 * the next heading of the same or a higher level, so a heading and what
 * follows it can be styled and linked to as one. A heading inside a callout
 * (a <div class="note"> written in the Markdown) gets the id itself instead.
 */
class Markdown_html {

    private Environment $environment;

    /** Ids already given to a heading in the current document, against reuse. */
    private array $used_ids = [];

    public function __construct(?string $assets_dir = null) {
        $assets_dir = rtrim($assets_dir ?? APPPATH . 'public', '/');

        $this->environment = new Environment(['html_input' => 'allow']);
        $this->environment->addExtension(new CommonMarkCoreExtension());
        $this->environment->addExtension(new TableExtension());
        $this->environment->addExtension(new SmartPunctExtension());
        $this->environment->addInlineParser(new Math_parser(), 100);
        $this->environment->addRenderer(Math_node::class, new Math_renderer());
        $this->environment->addRenderer(FencedCode::class, new Fenced_code_renderer(), 10);
        $this->environment->addRenderer(Image::class, new Svg_image_renderer($assets_dir), 10);
    }

    /**
     * Renders a Markdown body to HTML.
     *
     * Throws when a local SVG the body references is missing, so a broken
     * diagram is an error rather than an empty figure.
     */
    public function render(string $source): string {
        $document = $this->parse($source);
        $renderer = new HtmlRenderer($this->environment);

        $html = '';
        $open_sections = [];
        $div_depth = 0;
        foreach ($document->children() as $block) {
            $div_depth += $this->div_change($block);

            if ($block instanceof Heading && $div_depth === 0) {
                $html .= $this->close_sections($open_sections, $block->getLevel());
                $html .= '<section id="' . htmlspecialchars($this->section_id($block), ENT_QUOTES) . "\">\n";
                $open_sections[] = $block->getLevel();
            } elseif ($block instanceof Heading) {
                $block->data->set('attributes/id', $this->section_id($block));
            }

            $html .= $renderer->renderNodes([$block]) . "\n";
        }
        $html .= $this->close_sections($open_sections, 1);

        return $html;
    }

    /**
     * True when the body has a heading anywhere, which a knowledge node may not.
     */
    public function has_heading(string $source): bool {
        foreach ($this->parse($source)->iterator() as $node) {
            if ($node instanceof Heading) {
                return true;
            }
        }
        return false;
    }

    /**
     * The body's top-level blocks as plain text, each with the section it sits
     * in: the most recent heading above it, as its id and text.
     *
     * Code, raw HTML and math are left out, since their words are not prose a
     * question would be phrased in. A passage ending in a colon is joined to
     * the next one in the same section, so "Like this:" stays with what it
     * introduces.
     *
     * @return array<int, array{text: string, section: ?array{id: string, heading: string}}>
     */
    public function passages(string $source): array {
        $document = $this->parse($source);

        $passages = [];
        $section = null;
        foreach ($document->children() as $block) {
            if ($block instanceof Heading) {
                $section = [
                    'id' => $this->section_id($block),
                    'heading' => $this->collapse_whitespace(self::text_of($block)),
                ];
                continue;
            }

            $text = $this->collapse_whitespace($this->plain_text($block));
            if ($text === '') {
                continue;
            }

            $last = count($passages) - 1;
            $joins_previous = $last >= 0
                && str_ends_with($passages[$last]['text'], ':')
                && $passages[$last]['section'] === $section;
            if ($joins_previous) {
                $passages[$last]['text'] .= ' ' . $text;
                continue;
            }

            $passages[] = ['text' => $text, 'section' => $section];
        }

        return $passages;
    }

    public static function is_local_svg(string $source): bool {
        return str_starts_with($source, '/') && str_ends_with($source, '.svg');
    }

    public static function svg_figure(string $alt, string $svg): string {
        return '<figure class="diagram" role="img" aria-label="' . htmlspecialchars($alt, ENT_QUOTES) . '">'
            . $svg . '</figure>';
    }

    /**
     * The words of a node, as a reader would read them: text, inline code,
     * and an image's alt text.
     */
    public static function text_of(Node $node): string {
        if ($node instanceof Text || $node instanceof Code) {
            return $node->getLiteral();
        }
        if ($node instanceof Newline) {
            return ' ';
        }

        $text = '';
        foreach ($node->children() as $child) {
            $text .= self::text_of($child);
        }
        return $text;
    }

    private function parse(string $source): Document {
        $this->used_ids = [];
        $parser = new MarkdownParser($this->environment);
        return $parser->parse($source);
    }

    /**
     * +1 for a raw block that opens a <div>, -1 for one that closes it: how
     * the Markdown writes a callout around ordinary paragraphs.
     */
    private function div_change(Node $block): int {
        if (!$block instanceof HtmlBlock) {
            return 0;
        }
        $literal = $block->getLiteral();
        return substr_count($literal, '<div') - substr_count($literal, '</div>');
    }

    private function close_sections(array &$open_sections, int $level): string {
        $html = '';
        while ($open_sections !== [] && end($open_sections) >= $level) {
            array_pop($open_sections);
            $html .= "</section>\n";
        }
        return $html;
    }

    /**
     * The heading's id, the same as the published pages always had, so links
     * into them keep working: its letters, digits, hyphens and underscores,
     * each run of whitespace a hyphen and everything else dropped. "1. Create
     * the server" is 1-Create-the-server; "Built on trongate.cloud" is
     * Built-on-trongatecloud. A repeat gets -1, -2 after it.
     */
    private function section_id(Heading $heading): string {
        $known = $heading->data->get('section_id', null);
        if ($known !== null) {
            return $known;
        }

        $kept = preg_replace('/[^\p{L}\p{N}_\-\s]/u', '', self::text_of($heading));
        $id = implode('-', preg_split('/\s+/u', trim($kept), -1, PREG_SPLIT_NO_EMPTY));

        $unique = $id;
        $number = 1;
        while (isset($this->used_ids[$unique])) {
            $unique = $id . '-' . $number;
            $number++;
        }

        $this->used_ids[$unique] = true;
        $heading->data->set('section_id', $unique);
        return $unique;
    }

    private function plain_text(Node $node): string {
        $unsearchable = $node instanceof FencedCode
            || $node instanceof IndentedCode
            || $node instanceof HtmlBlock
            || $node instanceof HtmlInline
            || $node instanceof Math_node;
        if ($unsearchable) {
            return '';
        }
        if ($node instanceof Text || $node instanceof Code) {
            return $node->getLiteral();
        }
        if ($node instanceof Newline) {
            return ' ';
        }
        if ($node instanceof Image) {
            return self::text_of($node);
        }

        $text = '';
        foreach ($node->children() as $child) {
            $text .= $this->plain_text($child);
            if ($child instanceof AbstractBlock) {
                $text .= ' ';
            }
        }
        return $text;
    }

    private function collapse_whitespace(string $text): string {
        return implode(' ', preg_split('/\s+/u', trim($text), -1, PREG_SPLIT_NO_EMPTY));
    }

}
