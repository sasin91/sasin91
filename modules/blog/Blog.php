<?php
/**
 * /blog/ and every post under it. A post's URL is the `path` in its header,
 * so nested URLs like /blog/trongate/mx-transition/ survive.
 *
 * config/custom_routing.php sends /blog/<anything> to blog/show; the AI
 * article's three pages under /blog/ai/ are routed to the knowledge module
 * ahead of that.
 */
class Blog extends Trongate {

    /** Written under the listing's date line, to say what the article is. */
    const ARTICLE_LABEL = 'Long read, with a timeline and Ask';

    public function index(): void {
        $cv = $this->content->cv();
        $posts = $this->content->posts();
        $article = $this->content->article_outline();

        $data = [
            'view_module' => 'blog',
            'view_file' => 'listing',
            'cv' => $cv,
            'entries' => $this->listing($posts, $article),
            'nav' => 'blog',
            'meta' => [
                'title' => 'Writing — ' . $cv['site']['name'],
                'description' => 'Notes on things I built and what broke on the way.',
                'path' => '/blog/',
                'og_type' => 'website',
            ],
        ];
        $this->layout->page($data);
    }

    public function show(): void {
        $path = 'blog/' . implode('/', array_slice(SEGMENTS, 3));
        $post = $this->content->post($path);
        if ($post === null) {
            $this->site->not_found();
            return;
        }

        $cv = $this->content->cv();
        $og_image = $this->og_image($post);
        $search = $this->search->_assets();

        $data = [
            'view_module' => 'blog',
            'view_file' => 'post',
            'cv' => $cv,
            'post' => $post,
            'has_examples' => $post['ask'] !== [],
            'index_url' => $search['index_url'],
            'model_bytes' => $search['model']['bytes'],
            'nav' => 'blog',
            'syntax' => $post['has_syntax'],
            'lightbox' => $post['has_diagrams'],
            'meta' => [
                'title' => $post['title'] . ' — ' . $cv['site']['name'],
                'description' => $post['description'],
                'path' => $post['url'],
                'og_type' => 'article',
                'image' => $og_image['image'],
                'image_alt' => $og_image['image_alt'],
            ],
        ];
        $this->layout->page($data);
    }

    /**
     * The posts in their listing order, with the AI article placed among them
     * by its publication date. A series is compared by the date it is listed
     * under, so the article never lands between two of its parts.
     */
    private function listing(array $posts, array $article): array {
        $entries = [];
        $article_placed = false;
        foreach ($posts as $post) {
            if (!$article_placed && $post['listed_date'] < $article['published']) {
                $entries[] = $this->article_entry($article);
                $article_placed = true;
            }
            $entries[] = $this->post_entry($post);
        }
        if (!$article_placed) {
            $entries[] = $this->article_entry($article);
        }
        return $entries;
    }

    private function post_entry(array $post): array {
        return [
            'url' => $post['url'],
            'title' => $post['title'],
            'date_iso' => $post['date_iso'],
            'date_long' => $post['date_long'],
            'description' => $post['description'],
            'label' => null,
        ];
    }

    private function article_entry(array $article): array {
        return [
            'url' => $article['url'],
            'title' => $article['title'],
            'date_iso' => $article['published_iso'],
            'date_long' => $article['published_long'],
            'description' => $article['description'],
            'label' => self::ARTICLE_LABEL,
        ];
    }

    /**
     * The link-share picture: the post's card if it has one, otherwise its
     * hero when that is a raster image, otherwise none. Crawlers refuse an
     * SVG, so an SVG hero shares as no image rather than a broken one, and a
     * card exists precisely to give such a post a picture to share.
     */
    private function og_image(array $post): array {
        $card = $post['card'];
        if ($card !== null) {
            if (!$this->is_raster($card)) {
                throw new RuntimeException("post \"{$post['path']}\": card = \"{$card}\" is not a raster image; crawlers never share an SVG");
            }
            if ($post['card_alt'] === null) {
                throw new RuntimeException("post \"{$post['path']}\": card = \"{$card}\" has no card_alt; the card is a different image from the hero");
            }
            return ['image' => SITE_URL . $card, 'image_alt' => $post['card_alt']];
        }

        $hero = $post['hero'];
        if ($hero === null || !$this->is_raster($hero)) {
            return ['image' => null, 'image_alt' => null];
        }
        return ['image' => SITE_URL . $hero, 'image_alt' => $post['hero_alt']];
    }

    private function is_raster(string $path): bool {
        foreach (['.png', '.jpg', '.jpeg', '.webp'] as $extension) {
            if (str_ends_with($path, $extension)) {
                return true;
            }
        }
        return false;
    }

}
