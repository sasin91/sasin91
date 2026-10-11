<?php
require_once __DIR__ . '/Site_renderer.php';

use Djot\DjotConverter;
use Djot\Node\Block\BlockNode;
use Djot\Node\Block\CodeBlock;
use Djot\Node\Block\Div;
use Djot\Node\Block\Footnote;
use Djot\Node\Block\Heading;
use Djot\Node\Block\RawBlock;
use Djot\Node\Inline\Code;
use Djot\Node\Inline\EscapedText;
use Djot\Node\Inline\FootnoteRef;
use Djot\Node\Inline\HardBreak;
use Djot\Node\Inline\Image;
use Djot\Node\Inline\Math;
use Djot\Node\Inline\RawInline;
use Djot\Node\Inline\SoftBreak;
use Djot\Node\Inline\Symbol;
use Djot\Node\Inline\Text;
use Djot\Node\Node;

/**
 * Djot source to HTML, and to the plain-text passages search embeds.
 *
 * The HTML side is php-collective/djot with Site_renderer on top: fenced code
 * is highlighted, math becomes MathML, and a local SVG image is inlined so it
 * follows the theme toggle. The passages side walks the same parse tree.
 */
class Djot_html {

    private string $assets_dir;

    public function __construct(?string $assets_dir = null) {
        $this->assets_dir = $assets_dir ?? APPPATH . 'public';
    }

    /**
     * Renders a Djot body to HTML.
     *
     * Throws when a local SVG the body references is missing, so a broken
     * diagram is an error rather than an empty figure.
     */
    public function render(string $source): string {
        $renderer = new Site_renderer($this->assets_dir);
        $converter = new DjotConverter(renderer: $renderer);
        $document = $converter->parse($source);
        $this->fix_quotes($document);
        return $converter->render($document);
    }

    /**
     * Two places the library curls a straight quote the wrong way, both put
     * right here:
     *
     * - a double quote after an opening bracket: ("Needs 3 came out as
     *   (”Needs 3, and a closing quote can never follow an opening bracket;
     * - an apostrophe straight after inline math or code: $`q`'s came out
     *   with an opening quote, where it closes a word.
     */
    private function fix_quotes(Node $node): void {
        $previous = null;
        foreach ($node->getChildren() as $child) {
            if ($child instanceof Text) {
                $fixed = preg_replace('/([(\[{])\x{201D}/u', "$1\u{201C}", $child->getContent());
                $follows_word = $previous instanceof Math || $previous instanceof Code;
                if ($follows_word && str_starts_with($fixed, "\u{2018}")) {
                    $fixed = "\u{2019}" . substr($fixed, strlen("\u{2018}"));
                }
                $child->setContent($fixed);
            } else {
                $this->fix_quotes($child);
            }
            $previous = $child;
        }
    }

    /**
     * True when the body has a heading anywhere, which a knowledge node may not.
     */
    public function has_heading(string $source): bool {
        $document = (new DjotConverter())->parse($source);
        return $this->contains_heading($document);
    }

    /**
     * The body's top-level blocks as plain text, each with the section it sits
     * in: the most recent document-level heading above it, as its id and text.
     *
     * Code, raw HTML, math and footnotes are left out, since their words are
     * not prose a question would be phrased in. A passage ending in a colon is
     * joined to the next one in the same section, so "Like this:" stays with
     * what it introduces.
     *
     * @return array<int, array{text: string, section: ?array{id: string, heading: string}}>
     */
    public function passages(string $source): array {
        $renderer = new Site_renderer($this->assets_dir);
        $converter = new DjotConverter(renderer: $renderer);
        $document = $converter->parse($source);
        $this->fix_quotes($document);

        $passages = [];
        $section = null;
        foreach ($this->top_level_blocks($document) as $block) {
            if ($block instanceof Heading) {
                $section = [
                    'id' => $renderer->section_id($block),
                    'heading' => $this->collapse_whitespace($this->plain_text($block)),
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

    /**
     * The document's blocks, with divs opened up: a callout's paragraphs are
     * passages of their own, exactly as if the div were not there.
     *
     * @return Node[]
     */
    private function top_level_blocks(Node $parent): array {
        $blocks = [];
        foreach ($parent->getChildren() as $child) {
            if ($child instanceof Div) {
                $blocks = array_merge($blocks, $this->top_level_blocks($child));
                continue;
            }
            $blocks[] = $child;
        }
        return $blocks;
    }

    private function plain_text(Node $node): string {
        if ($this->is_unsearchable($node)) {
            return '';
        }
        if ($node instanceof Text || $node instanceof EscapedText || $node instanceof Code) {
            return $node->getContent();
        }
        if ($node instanceof Image) {
            return $node->getAlt();
        }
        if ($node instanceof Symbol) {
            return ':' . $node->getName() . ':';
        }
        if ($node instanceof SoftBreak || $node instanceof HardBreak) {
            return ' ';
        }

        $text = '';
        foreach ($node->getChildren() as $child) {
            $text .= $this->plain_text($child);
            if ($child instanceof BlockNode) {
                $text .= ' ';
            }
        }
        return $text;
    }

    private function is_unsearchable(Node $node): bool {
        return $node instanceof CodeBlock
            || $node instanceof RawBlock
            || $node instanceof RawInline
            || $node instanceof Math
            || $node instanceof Footnote
            || $node instanceof FootnoteRef;
    }

    private function contains_heading(Node $node): bool {
        foreach ($node->getChildren() as $child) {
            if ($child instanceof Heading) {
                return true;
            }
            if ($this->contains_heading($child)) {
                return true;
            }
        }
        return false;
    }

    private function collapse_whitespace(string $text): string {
        $words = preg_split('/\s+/u', trim($text), -1, PREG_SPLIT_NO_EMPTY);
        return implode(' ', $words);
    }

}
