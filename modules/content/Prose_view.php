<?php
require_once APPPATH . 'modules/prose/prose_helper.php';

/**
 * A post or a knowledge node: an ordinary PHP view, written with the prose
 * helpers (section_open(), code_open() …), whose file starts with the same
 * +++ TOML header the listing reads.
 *
 * The header is not PHP, so rendering the view outputs it as text first; it
 * is cut off here.
 */
class Prose_view {

    /**
     * The view's HTML, without its header.
     */
    public static function html(string $file): string {
        Modules::run('prose/reset_ids', []);
        ob_start();
        require $file;
        $output = ob_get_clean();

        $start = strpos($output, '+++');
        $end = strpos($output, '+++', $start + 3);
        return ltrim(substr($output, $end + 3));
    }

    /**
     * The text search embeds, read from rendered HTML: each top-level block's
     * text, in the section around it. Sections and callouts are opened up;
     * code, math, scripts and media are left out; an image or a diagram
     * reads as its description. A passage ending in a colon is joined to the
     * next one in the same section, so "Like this:" stays with what it
     * introduces.
     *
     * @return array<int, array{text: string, section: ?array{id: string, heading: string}}>
     */
    public static function passages(string $html): array {
        $dom = new DOMDocument();
        libxml_use_internal_errors(true);
        $dom->loadHTML('<?xml encoding="UTF-8"><body>' . $html . '</body>', LIBXML_NOERROR);
        libxml_clear_errors();

        $passages = [];
        $section = null;
        $body = $dom->getElementsByTagName('body')->item(0);
        self::collect($body, $section, $passages);
        return $passages;
    }

    private static function collect(DOMNode $parent, ?array &$section, array &$passages): void {
        foreach ($parent->childNodes as $node) {
            if (!$node instanceof DOMElement) {
                continue;
            }
            $tag = strtolower($node->tagName);

            if ($tag === 'section' || $tag === 'div') {
                if ($node->hasAttribute('id')) {
                    $section = ['id' => $node->getAttribute('id'), 'heading' => self::heading_text($node)];
                }
                self::collect($node, $section, $passages);
                continue;
            }
            if (preg_match('/^h[1-6]$/', $tag) === 1) {
                if ($node->hasAttribute('id')) {
                    $section = ['id' => $node->getAttribute('id'), 'heading' => self::text($node)];
                }
                continue;
            }

            $text = self::block_text($node);
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
    }

    /** A section's own heading: its first h1-h6 child. */
    private static function heading_text(DOMElement $section): string {
        foreach ($section->childNodes as $child) {
            if ($child instanceof DOMElement && preg_match('/^h[1-6]$/', strtolower($child->tagName)) === 1) {
                return self::text($child);
            }
        }
        return '';
    }

    private static function block_text(DOMElement $node): string {
        $tag = strtolower($node->tagName);
        if (in_array($tag, ['pre', 'script', 'style', 'video', 'math'], true)) {
            return '';
        }
        if ($tag === 'figure') {
            return self::collapse($node->getAttribute('aria-label'));
        }
        if ($node->getAttribute('class') === 'codeblock') {
            return '';
        }
        $copy = $node->cloneNode(true);
        foreach (['pre', 'math', 'script'] as $unsearchable) {
            $found = iterator_to_array($copy->getElementsByTagName($unsearchable));
            foreach ($found as $element) {
                $element->parentNode->removeChild($element);
            }
        }
        foreach (iterator_to_array($copy->getElementsByTagName('img')) as $image) {
            $image->parentNode->replaceChild($copy->ownerDocument->createTextNode($image->getAttribute('alt')), $image);
        }
        // A diagram reads as its description, not as the text inside the SVG.
        foreach (iterator_to_array($copy->getElementsByTagName('figure')) as $figure) {
            $figure->parentNode->replaceChild($copy->ownerDocument->createTextNode($figure->getAttribute('aria-label')), $figure);
        }
        return self::text($copy);
    }

    private static function text(DOMNode $node): string {
        return self::collapse($node->textContent);
    }

    private static function collapse(string $text): string {
        return implode(' ', preg_split('/\s+/u', trim($text), -1, PREG_SPLIT_NO_EMPTY));
    }

}
