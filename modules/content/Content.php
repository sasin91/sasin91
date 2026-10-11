<?php
require_once APPPATH . 'packages/autoload.php';
require_once __DIR__ . '/Prose_view.php';
require_once __DIR__ . '/Posts.php';
require_once __DIR__ . '/Cv_data.php';
require_once __DIR__ . '/Knowledge_article.php';

/**
 * Everything the pages show: the posts (modules/blog/views/posts), the CV
 * (modules/cv/cv.toml) and the AI article (modules/knowledge). Other modules ask this one (`$this->content->posts()`); it
 * answers no URL itself.
 *
 * Nothing is built ahead of time. A post renders when its page is asked for,
 * and the files are the only source, so a push that changes them changes the
 * site with no build step in between.
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
        $posts = new Posts(APPPATH . 'modules/blog/views/posts');
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
        return Cv_data::load(APPPATH . 'modules/cv/cv.toml');
    }

    /**
     * The AI article, every node rendered and the whole checked.
     */
    public function article(): array {
        $article = new Knowledge_article(APPPATH . 'modules/knowledge');
        return $article->load();
    }

    /**
     * The AI article's structure and sources, without rendering any node, for
     * the search index and the listing.
     */
    public function article_outline(): array {
        $article = new Knowledge_article(APPPATH . 'modules/knowledge');
        return $article->load(false);
    }

    /**
     * A post's body, and the hero above it: a local SVG hero is inlined the
     * same way a diagram in the body is, so it follows the theme toggle too.
     */
    private function rendered(array $post): array {
        $post['body'] = Prose_view::html($post['file']);

        $post['alt_text'] = $post['hero_alt'] ?? '';
        $post['hero_html'] = null;
        $hero = $post['hero'];
        if ($hero !== null && str_starts_with($hero, '/') && str_ends_with($hero, '.svg')) {
            $post['hero_html'] = diagram($hero, $post['alt_text']);
        }

        $post['hero_is_img'] = $hero !== null && $post['hero_html'] === null;
        $post['has_syntax'] = str_contains($post['body'], '<pre class="hl-code">');
        $post['has_diagrams'] = str_contains($post['body'] . $post['hero_html'], '<figure class="diagram"');

        return $post;
    }

}
