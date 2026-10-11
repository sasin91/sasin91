<?php
require_once APPPATH . 'packages/autoload.php';
require_once __DIR__ . '/Markdown_html.php';
require_once __DIR__ . '/Posts.php';
require_once __DIR__ . '/Cv_data.php';
require_once __DIR__ . '/Knowledge_article.php';

/**
 * Everything the pages show, read from content/: the posts, the CV and the
 * AI article. Other modules ask this one (`$this->content->posts()`); it
 * answers no URL itself.
 *
 * Nothing is built ahead of time. A post renders when its page is asked for,
 * and the files are the only source, so a push that changes content/ changes
 * the site with no build step in between.
 */
class Content extends Trongate {

    public function __construct(?string $module_name = null) {
        parent::__construct($module_name);
        block_url('content');
    }

    /**
     * Every post's header, newest first, without rendering a body.
     */
    public function posts(): array {
        $posts = new Posts(APPPATH . 'content/blog');
        return $posts->all();
    }

    /**
     * One post with its body rendered, or null when no post has this path.
     */
    public function post(string $path): ?array {
        foreach ($this->posts() as $post) {
            if ($post['path'] === $path) {
                return $this->rendered($post);
            }
        }
        return null;
    }

    public function cv(): array {
        return Cv_data::load(APPPATH . 'content/cv.toml');
    }

    /**
     * The AI article, every node rendered and the whole checked.
     */
    public function article(): array {
        $article = new Knowledge_article(APPPATH . 'content/ai', new Markdown_html());
        return $article->load();
    }

    /**
     * The AI article's structure and sources, without rendering any node, for
     * the search index and the listing.
     */
    public function article_outline(): array {
        $article = new Knowledge_article(APPPATH . 'content/ai', new Markdown_html());
        return $article->load(false);
    }

    /**
     * A post's body, and the hero above it: a local SVG hero is inlined the
     * same way a diagram in the body is, so it follows the theme toggle too.
     */
    private function rendered(array $post): array {
        $markdown = new Markdown_html();
        $post['body'] = $markdown->render($post['source']);

        $post['alt_text'] = $post['hero_alt'] ?? '';
        $post['hero_html'] = null;
        $hero = $post['hero'];
        if ($hero !== null && Markdown_html::is_local_svg($hero)) {
            $svg_file = APPPATH . 'public/' . ltrim($hero, '/');
            if (!is_file($svg_file)) {
                throw new RuntimeException('missing SVG referenced by post hero: ' . $hero);
            }
            $post['hero_html'] = Markdown_html::svg_figure($post['alt_text'], file_get_contents($svg_file));
        }

        $post['hero_is_img'] = $hero !== null && $post['hero_html'] === null;
        $post['has_syntax'] = str_contains($post['body'], '<pre class="hl-code">');
        $post['has_diagrams'] = str_contains($post['body'] . $post['hero_html'], '<figure class="diagram"');

        return $post;
    }

}
