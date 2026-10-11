<?php
/**
 * The AI article's three views of the same knowledge nodes: Read
 * (/blog/ai/), Timeline (/blog/ai/timeline/) and Ask (/blog/ai/ask/).
 */
class Knowledge extends Trongate {

    /** The examples /blog/ai/ask/ offers; the first is also the placeholder. */
    const ASK_EXAMPLES = [
        "Why doesn't a larger context window replace RAG?",
        'What is stored in the KV cache?',
        'Is MCP the same thing as RAG?',
        'Does the model learn from my conversation?',
        'top-p',
    ];

    public function read(): void {
        $cv = $this->content->cv();
        $article = $this->content->article();
        $nodes_html = implode('', array_column($article['nodes'], 'body'));

        $data = [
            'view_module' => 'knowledge',
            'view_file' => 'read',
            'cv' => $cv,
            'article' => $article,
            'node_count' => count($article['nodes']),
            'views' => $this->views($article, 'read'),
            'nav' => 'blog',
            'syntax' => str_contains($nodes_html, '<pre class="hl-code">'),
            'lightbox' => str_contains($nodes_html, '<figure class="diagram"'),
            'meta' => [
                'title' => $article['title'] . ' — ' . $cv['site']['name'],
                'description' => $article['description'],
                'path' => $article['url'],
                'og_type' => 'article',
            ],
        ];
        $this->layout->page($data);
    }

    public function timeline(): void {
        $cv = $this->content->cv();
        $article = $this->content->article_outline();

        $data = [
            'view_module' => 'knowledge',
            'view_file' => 'timeline',
            'cv' => $cv,
            'article' => $article,
            'views' => $this->views($article, 'timeline'),
            'nav' => 'blog',
            'meta' => [
                'title' => 'Timeline: ' . $article['title'] . ' — ' . $cv['site']['name'],
                'description' => $article['description'] . ' One line per idea, each linking to its full section.',
                'path' => $article['url'] . 'timeline/',
                'og_type' => 'website',
            ],
        ];
        $this->layout->page($data);
    }

    public function ask(): void {
        $cv = $this->content->cv();
        $article = $this->content->article_outline();
        $search = $this->search->_assets();

        $data = [
            'view_module' => 'knowledge',
            'view_file' => 'ask',
            'cv' => $cv,
            'title' => $article['title'],
            'read_url' => $article['url'],
            'timeline_url' => $article['url'] . 'timeline/',
            'ask_url' => $article['url'] . 'ask/',
            'ai_url' => $article['url'],
            'scope' => 'ai',
            'examples' => self::ASK_EXAMPLES,
            'placeholder' => self::ASK_EXAMPLES[0],
            'index_url' => $search['index_url'],
            'model' => $search['model'],
            'script' => $this->search->_ask_script(),
            'nav' => 'blog',
            'meta' => [
                'title' => 'Ask: ' . $article['title'] . ' — ' . $cv['site']['name'],
                'description' => 'Ask a question and get the most relevant sections of the article, found by semantic search that runs entirely in your browser.',
                'path' => $article['url'] . 'ask/',
                'og_type' => 'website',
            ],
        ];
        $this->layout->page($data);
    }

    /**
     * The Read / Timeline / Ask switcher: plain links between three pages,
     * so every view works without JavaScript and has its own URL.
     */
    private function views(array $article, string $current): array {
        return [
            ['label' => 'Read', 'url' => $article['url'], 'current' => $current === 'read'],
            ['label' => 'Timeline', 'url' => $article['url'] . 'timeline/', 'current' => $current === 'timeline'],
            ['label' => 'Ask', 'url' => $article['url'] . 'ask/', 'current' => $current === 'ask'],
        ];
    }

}
