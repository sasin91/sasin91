<?php
require_once __DIR__ . '/Static_model.php';
require_once APPPATH . 'modules/content/Prose_view.php';

/**
 * The search index behind the palette, /blog/ai/ask/ and every post's Ask
 * box: the AI article's knowledge nodes and every post, each with its
 * metadata, the words of its body for keyword matching, and one embedding per
 * passage. The browser downloads it only when a reader starts searching, and
 * ranks it with modules/search/js/retrieval.mjs.
 *
 * The index carries no full text, which is already on the pages: just a
 * short snippet per passage, so a result can show which part matched.
 */
class Search_index {

    /** Passage snippets are cut to about this many characters. */
    const SNIPPET_CHARS = 160;

    /** Four decimals is plenty for cosine similarity and halves the JSON. */
    const VECTOR_DECIMALS = 4;

    private Static_model $model;
    public function __construct(Static_model $model) {
        $this->model = $model;
    }

    /**
     * Every node of the article in reading order, then every post in listing
     * order, as the JSON the browser reads.
     */
    public function build(array $article, array $posts, array $model_info): string {
        $documents = [];

        foreach ($article['nodes'] as $node) {
            $body = Prose_view::passages(Prose_view::html($node['file']));
            $passages = $this->passage_list([
                $this->embedded($this->node_head($node), $node['summary'], null),
            ]);
            $passages = array_merge($passages, $this->body_passages($body));

            $documents[] = [
                'kind' => 'ai',
                'context' => $article['title'],
                'id' => $node['id'],
                'order' => $node['order'],
                'url' => $article['url'] . '#' . $node['id'],
                'title' => $node['title'],
                'summary' => $node['summary'],
                'concepts' => $node['concepts'],
                'search_terms' => $node['search_terms'],
                'words' => $this->body_words($body),
                'passages' => $passages,
            ];
        }

        $node_count = count($article['nodes']);
        foreach ($posts as $index => $post) {
            $body = Prose_view::passages(Prose_view::html($post['file']));
            $head = $post['title'] . '. ' . $post['description'];
            $passages = $this->passage_list([
                $this->embedded($head, $post['description'], null),
            ]);
            $passages = array_merge($passages, $this->body_passages($body));

            $document = [
                'kind' => 'post',
                'context' => 'Writing',
                'id' => $post['path'],
                'order' => $node_count + $index + 1,
                'url' => $post['url'],
                'title' => $post['title'],
                'summary' => $post['description'],
                'concepts' => [],
                'search_terms' => [],
                'words' => $this->body_words($body),
                'passages' => $passages,
            ];
            $sections = $this->post_sections($post, $body);
            if ($sections !== []) {
                $document['sections'] = $sections;
            }
            $documents[] = $document;
        }

        $index = [
            'version' => 2,
            'model' => $model_info,
            'nodes' => $documents,
        ];
        return json_encode($index, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES | JSON_THROW_ON_ERROR);
    }

    /**
     * A post's sections as documents of their own, in page order, so the Ask
     * box at the top of a post can rank them the way /blog/ai/ask/ ranks
     * nodes: the text above the first heading as "Introduction", then one per
     * heading. Passages group by their innermost heading.
     */
    private function post_sections(array $post, array $body): array {
        $groups = [];
        foreach ($body as $passage) {
            $last = count($groups) - 1;
            if ($last >= 0 && $groups[$last]['section'] === $passage['section']) {
                $groups[$last]['passages'][] = $passage;
                continue;
            }
            $groups[] = ['section' => $passage['section'], 'passages' => [$passage]];
        }

        $sections = [];
        foreach ($groups as $index => $group) {
            $section = $group['section'];
            $first_text = $group['passages'][0]['text'];
            $summary = $this->snippet($first_text);

            if ($section === null) {
                // Not the post's title: a question about anything in the post
                // shares words with it, so the opening would outrank the
                // section that answers.
                $id = 'top';
                $title = 'Introduction';
                $url = $post['url'];
                $passages = [];
            } else {
                $id = $section['id'];
                $title = $section['heading'];
                $url = $post['url'] . '#' . $section['id'];
                // The heading embedded with the section's opening, so a
                // question phrased like the heading is close in meaning too.
                $head = $title . '. ' . $first_text;
                $passages = $this->passage_list([$this->embedded($head, $summary, $section)]);
            }

            $sections[] = [
                'kind' => 'section',
                'context' => $post['title'],
                'id' => $id,
                'order' => $index + 1,
                'url' => $url,
                'title' => $title,
                'summary' => $summary,
                'concepts' => [],
                'search_terms' => [],
                'words' => $this->body_words($group['passages']),
                'passages' => array_merge($passages, $this->body_passages($group['passages'])),
            ];
        }
        return $sections;
    }

    /**
     * The text embedded for a node's metadata: everything its author wrote
     * about what the node is, in one passage.
     */
    private function node_head(array $node): string {
        return $node['title'] . '. ' . $node['summary'] . ' '
            . implode(', ', $node['concepts']) . '. '
            . implode(', ', $node['search_terms']) . '.';
    }

    private function body_passages(array $body): array {
        $passages = [];
        foreach ($body as $passage) {
            $passages[] = $this->embedded($passage['text'], $this->snippet($passage['text']), $passage['section']);
        }
        return $this->passage_list($passages);
    }

    /** Drops the passages that got no embedding. */
    private function passage_list(array $passages): array {
        return array_values(array_filter($passages, fn ($passage) => $passage !== null));
    }

    private function embedded(string $text, string $snippet, ?array $section): ?array {
        $vector = $this->model->encode($text);
        if ($vector === null) {
            return null;
        }

        $rounded = [];
        foreach ($vector as $value) {
            $rounded[] = round($value, self::VECTOR_DECIMALS);
        }

        $passage = ['snippet' => $snippet, 'vector' => $rounded];
        if ($section !== null) {
            $passage['anchor'] = $section['id'];
            $passage['heading'] = $section['heading'];
        }
        return $passage;
    }

    private function body_words(array $body): array {
        return self::lexical_words(implode(' ', array_column($body, 'text')));
    }

    /**
     * What a page and the index say about the model: its name, the URLs of
     * its two files (versioned by the vocabulary's hash and the weights'
     * size, so a new model is a new URL), how many bytes a reader will
     * download, and the embedding's dimension.
     */
    public static function model_info(string $dir, string $name): array {
        $weights = filesize($dir . '/embeddings.q8.bin');
        $vocab = file_get_contents($dir . '/vocab.txt');
        $version = substr(md5($vocab . ':' . $weights), 0, 16);

        // Every row is a 4-byte scale plus one byte per dimension.
        $rows = count(explode("\n", rtrim($vocab, "\n")));
        $dim = intdiv($weights - $rows * 4, $rows);

        return [
            'name' => $name,
            'weights_url' => '/search/model.bin?v=' . $version,
            'vocab_url' => '/search/vocab.txt?v=' . $version,
            'bytes' => $weights + strlen($vocab),
            'dim' => $dim,
        ];
    }

    /**
     * Distinct lowercase runs of letters and digits, sorted. Must split
     * exactly like `words` in retrieval.mjs.
     */
    public static function lexical_words(string $text): array {
        $words = preg_split('/[^\p{L}\p{N}]+/u', mb_strtolower($text), -1, PREG_SPLIT_NO_EMPTY);
        $words = array_values(array_unique($words));
        sort($words, SORT_STRING);
        return $words;
    }

    /**
     * Cuts on a word boundary near SNIPPET_CHARS, never mid-character.
     */
    public function snippet(string $text): string {
        if (mb_strlen($text) <= self::SNIPPET_CHARS) {
            return $text;
        }
        $cut = mb_substr($text, 0, self::SNIPPET_CHARS);
        $space = mb_strrpos($cut, ' ');
        if ($space !== false) {
            $cut = mb_substr($cut, 0, $space);
        }
        return rtrim($cut, ',;:.') . '…';
    }

}
