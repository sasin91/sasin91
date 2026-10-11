<?php
require_once __DIR__ . '/Highlighter.php';

use League\CommonMark\Extension\CommonMark\Node\Block\FencedCode;
use League\CommonMark\Extension\CommonMark\Node\Inline\Image;
use League\CommonMark\Extension\CommonMark\Renderer\Inline\ImageRenderer;
use League\CommonMark\Node\Inline\AbstractInline;
use League\CommonMark\Node\Node;
use League\CommonMark\Parser\Inline\InlineParserInterface;
use League\CommonMark\Parser\Inline\InlineParserMatch;
use League\CommonMark\Parser\InlineParserContext;
use League\CommonMark\Renderer\ChildNodeRendererInterface;
use League\CommonMark\Renderer\NodeRendererInterface;
use League\Config\ConfigurationAwareInterface;
use League\Config\ConfigurationInterface;
use Latex2MathML\Converter as Latex2MathML;

/**
 * The three things this site's Markdown does beyond CommonMark and GitHub's
 * tables, each a small piece league/commonmark is built to take:
 *
 * - math between dollar signs, $x$ inline and $$x$$ on its own line;
 * - fenced code highlighted, with a file name bar when the info string
 *   carries a title: ```bash title="1-create-server.sh";
 * - a local SVG image inlined, so its colours follow the theme toggle.
 */

/** $tex$ or $$tex$$, kept as the TeX source until it is rendered. */
class Math_node extends AbstractInline {

    public function __construct(public readonly string $tex, public readonly bool $display) {
        parent::__construct();
    }

}

class Math_parser implements InlineParserInterface {

    public function getMatchDefinition(): InlineParserMatch {
        return InlineParserMatch::regex('\$\$([^$]+)\$\$|\$([^$\s][^$\n]*?)\$');
    }

    public function parse(InlineParserContext $context): bool {
        [$display_tex, $inline_tex] = array_pad($context->getSubMatches(), 2, '');
        $display = $display_tex !== '';
        $tex = $display ? $display_tex : $inline_tex;

        $context->getCursor()->advanceBy($context->getFullMatchLength());
        $context->getContainer()->appendChild(new Math_node($tex, $display));
        return true;
    }

}

/**
 * Math as MathML, which every current browser renders natively, rather than
 * markup a script has to find and typeset later.
 */
class Math_renderer implements NodeRendererInterface {

    public function render(Node $node, ChildNodeRendererInterface $childRenderer): string {
        $display = $node->display ? 'block' : 'inline';
        try {
            return Latex2MathML::convert($node->tex, $display);
        } catch (Throwable $error) {
            return '<code class="math-error" title="' . htmlspecialchars($error->getMessage(), ENT_QUOTES) . '">'
                . htmlspecialchars($node->tex, ENT_QUOTES) . '</code>';
        }
    }

}

class Fenced_code_renderer implements NodeRendererInterface {

    private Highlighter $highlighter;

    public function __construct() {
        $this->highlighter = new Highlighter();
    }

    public function render(Node $node, ChildNodeRendererInterface $childRenderer): string {
        $info = $node->getInfo() ?? '';
        $language = $node->getInfoWords()[0] ?? '';
        if (str_contains($language, '=')) {
            $language = '';
        }
        $highlighted = $this->highlighter->to_html($node->getLiteral(), $language);

        if (preg_match('/title="([^"]*)"/', $info, $title) !== 1) {
            return $highlighted;
        }
        return '<div class="codeblock"><div class="filename">' . htmlspecialchars($title[1], ENT_QUOTES) . '</div>'
            . $highlighted . '</div>';
    }

}

class Svg_image_renderer implements NodeRendererInterface, ConfigurationAwareInterface {

    private ImageRenderer $images;

    public function __construct(private string $assets_dir) {
        $this->images = new ImageRenderer();
    }

    public function setConfiguration(ConfigurationInterface $configuration): void {
        $this->images->setConfiguration($configuration);
    }

    public function render(Node $node, ChildNodeRendererInterface $childRenderer): Stringable|string {
        $source = $node->getUrl();
        if (!Markdown_html::is_local_svg($source)) {
            return $this->images->render($node, $childRenderer);
        }

        $path = $this->assets_dir . '/' . ltrim($source, '/');
        if (!is_file($path)) {
            throw new RuntimeException('missing SVG referenced by post: ' . $source);
        }
        return Markdown_html::svg_figure(Markdown_html::text_of($node), file_get_contents($path));
    }

}
