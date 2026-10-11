<?php
require_once APPPATH . 'packages/autoload.php';
require_once APPPATH . 'modules/content/Markdown_html.php';
require_once __DIR__ . '/Search_index.php';

/**
 * Everything search needs under /search/: the model and its vocabulary, the
 * index of AI article nodes and blog posts, and the two scripts that use
 * them (the site-wide palette and the Ask form).
 *
 * The index is built from content/ the first time anyone asks for it after
 * the content changed, and kept on disk until it changes again: embedding
 * every passage takes a moment, and asking for the same answer twice should
 * not. Every URL carries a hash of what it serves, so a browser may cache it
 * for good and still never keep a stale copy.
 */
class Search extends Trongate {

    const MODEL_DIR = 'models/potion-base-4M';
    const MODEL_NAME = 'potion-base-4M';

    /**
     * The index URL and model description a page embeds in its Ask form.
     */
    public function _assets(): array {
        return [
            'index_url' => '/search/index.json?v=' . $this->content_hash(),
            'model' => $this->model_info(),
        ];
    }

    /** The Ask page's script, inlined into /blog/ai/ask/ as a module. */
    public function _ask_script(): string {
        return $this->script('ask.mjs');
    }

    public function index_json(): void {
        $file = $this->index_file();
        $this->send_immutable('application/json', $file);
    }

    /**
     * The palette (Ctrl+K on any page): the retrieval code and the palette's
     * own, with the index's current URL written in front.
     */
    public function palette_js(): void {
        $prefix = 'const SEARCH_INDEX_URL = ' . json_encode($this->_assets()['index_url']) . ";\n"
            . 'const AI_ARTICLE_URL = ' . json_encode('/blog/ai/') . ";\n";
        header('Content-Type: text/javascript; charset=utf-8');
        header('Cache-Control: no-cache');
        echo $prefix . $this->script('palette.mjs');
    }

    /** The Ask box at the top of every post, imported only on those pages. */
    public function ask_js(): void {
        header('Content-Type: text/javascript; charset=utf-8');
        header('Cache-Control: no-cache');
        echo $this->_ask_script();
    }

    public function model_bin(): void {
        $this->send_immutable('application/octet-stream', $this->model_dir() . '/embeddings.q8.bin');
    }

    public function vocab_txt(): void {
        $this->send_immutable('text/plain; charset=utf-8', $this->model_dir() . '/vocab.txt');
    }

    /**
     * The index on disk, building it first when the content has changed
     * since it was last built.
     */
    private function index_file(): string {
        $dir = sys_get_temp_dir() . '/sasin91-search';
        $file = $dir . '/index-' . $this->content_hash() . '.json';
        if (is_file($file)) {
            return $file;
        }

        $model = Static_model::from_dir($this->model_dir());
        $builder = new Search_index($model, new Markdown_html());
        $json = $builder->build($this->content->article_outline(), $this->content->posts(), $this->model_info());

        if (!is_dir($dir)) {
            mkdir($dir, 0775, true);
        }
        $partial = $file . '.' . getmypid();
        file_put_contents($partial, $json);
        rename($partial, $file);
        return $file;
    }

    private function model_info(): array {
        return Search_index::model_info($this->model_dir(), self::MODEL_NAME);
    }

    /**
     * A hash of every file the index is built from, so a changed post or
     * node means a new index and a new URL for it.
     */
    private function content_hash(): string {
        $files = array_merge(glob(APPPATH . 'content/blog/*.md'), glob(APPPATH . 'content/blog/*.phtml'), glob(APPPATH . 'content/ai/nodes/*.md'));
        $files[] = APPPATH . 'content/ai/article.toml';
        sort($files);

        $hash = hash_init('md5');
        foreach ($files as $file) {
            hash_update($hash, substr($file, strlen(APPPATH)));
            hash_update_file($hash, $file);
        }
        // The code that builds the index is part of what it is built from.
        foreach (['Search_index.php', 'Static_model.php', '../content/Markdown_html.php', '../content/Markdown_extensions.php', '../content/Post_body.php', '../prose/Prose.php'] as $code) {
            hash_update_file($hash, __DIR__ . '/' . $code);
        }
        // So is the model the passages are embedded with.
        hash_update($hash, $this->model_info()['weights_url']);
        return substr(hash_final($hash), 0, 16);
    }

    private function model_dir(): string {
        return APPPATH . self::MODEL_DIR;
    }

    /**
     * The retrieval code followed by one of the scripts that use it, as one
     * module, the way the browser gets it.
     */
    private function script(string $file): string {
        $dir = __DIR__ . '/js/';
        return file_get_contents($dir . 'retrieval.mjs') . "\n" . file_get_contents($dir . $file);
    }

    private function send_immutable(string $content_type, string $file): void {
        header('Content-Type: ' . $content_type);
        header('Content-Length: ' . filesize($file));
        header('Cache-Control: public, max-age=31536000, immutable');
        readfile($file);
    }

}
