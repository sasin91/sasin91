<?php
/**
 * The page shell every HTML page shares: head, site header, footer and the
 * theme and search scripts. A page module hands it the view to put in <main>
 * and the page's meta, and the shell does the rest.
 */
class Layout extends Trongate {

    public function __construct(?string $module_name = null) {
        parent::__construct($module_name);
        block_url('layout');
    }

    /**
     * Renders a page.
     *
     * Expects in $data: view_module and view_file (the page's own view), cv,
     * nav ('', 'blog', 'about' or 'cv'), and meta: title, description, path
     * (the page's own path, for canonical and og:url), og_type, and optionally
     * image and image_alt. syntax and lightbox say whether the page needs
     * syntax.css and the diagram lightbox.
     */
    public function page(array $data): void {
        $data['content'] = $this->view($data['view_file'], $data, true);

        $data['meta']['url'] = SITE_URL . $data['meta']['path'];
        $data['meta']['image'] = $data['meta']['image'] ?? null;
        $data['meta']['image_alt'] = $data['meta']['image_alt'] ?? null;
        $data['syntax'] = $data['syntax'] ?? false;
        $data['lightbox'] = $data['lightbox'] ?? false;
        $data['site_css_url'] = $this->asset_url('css/site.css');
        $data['syntax_css_url'] = $this->asset_url('css/syntax.css');
        $data['year'] = date('Y');
        $data['nav_current'] = [
            'blog' => $data['nav'] === 'blog',
            'about' => $data['nav'] === 'about',
            'cv' => $data['nav'] === 'cv',
        ];

        unset($data['view_module']);
        $this->view('page', $data);
    }

    /**
     * A file under public/ with a hash of its bytes in the query string, so a
     * browser can cache it for as long as it likes and still never keeps an
     * old copy after the file changes.
     */
    private function asset_url(string $file): string {
        $hash = substr(md5_file(APPPATH . 'public/' . $file), 0, 16);
        return '/' . $file . '?v=' . $hash;
    }

}
