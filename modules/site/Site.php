<?php
/**
 * The pages that are about the person rather than a piece of writing: the
 * home page and /about/, plus the feeds and files crawlers read.
 */
class Site extends Trongate {

    public function index(): void {
        $cv = $this->content->cv();

        $data = [
            'view_module' => 'site',
            'view_file' => 'home',
            'cv' => $cv,
            'posts' => $this->content->posts(),
            'nav' => '',
            'meta' => [
                'title' => $cv['site']['name'] . ' — ' . $cv['site']['title'],
                'description' => $cv['site']['stack_line'],
                'path' => '/',
                'og_type' => 'website',
            ],
        ];
        $this->layout->page($data);
    }

    public function about(): void {
        $cv = $this->content->cv();

        $data = [
            'view_module' => 'site',
            'view_file' => 'about',
            'cv' => $cv,
            'nav' => 'about',
            'meta' => [
                'title' => 'About — ' . $cv['site']['name'],
                'description' => $cv['site']['title'] . ' in ' . $cv['contact']['town'] . '. ' . $cv['site']['available_note'] . '.',
                'path' => '/about/',
                'og_type' => 'website',
            ],
        ];
        $this->layout->page($data);
    }

    public function rss(): void {
        $posts = $this->content->posts();

        $last_build_date = '';
        if ($posts !== []) {
            $last_build_date = $posts[0]['date_rfc2822'];
        }

        $data = [
            'cv' => $this->content->cv(),
            'posts' => $posts,
            'base' => SITE_URL,
            'last_build_date' => $last_build_date,
        ];
        header('Content-Type: application/rss+xml; charset=utf-8');
        $this->view('rss', $data);
    }

    public function sitemap(): void {
        $data = [
            'posts' => $this->content->posts(),
            'article' => $this->content->article_outline(),
            'base' => SITE_URL,
        ];
        header('Content-Type: application/xml; charset=utf-8');
        $this->view('sitemap', $data);
    }

    public function robots(): void {
        header('Content-Type: text/plain; charset=utf-8');
        echo "User-agent: *\nAllow: /\nSitemap: " . SITE_URL . "/sitemap.xml\n";
    }

    /**
     * Any URL nothing else answers (ERROR_404 in config/config.php).
     */
    public function not_found(): void {
        http_response_code(404);
        $cv = $this->content->cv();

        $data = [
            'view_module' => 'site',
            'view_file' => 'not_found',
            'cv' => $cv,
            'nav' => '',
            'meta' => [
                'title' => 'Not found — ' . $cv['site']['name'],
                'description' => 'There is no page at this address.',
                'path' => '/',
                'og_type' => 'website',
            ],
        ];
        $this->layout->page($data);
    }

}
