<?php
require_once __DIR__ . '/Front_matter.php';
require_once __DIR__ . '/Prose_view.php';

/**
 * The long-form article at /blog/ai/, assembled from the knowledge nodes in
 * modules/knowledge/views/nodes (one view each) and the reading order in
 * modules/knowledge/article.toml.
 *
 * Nodes do not carry their own position: the article's order is the order
 * their ids appear in the manifest's parts. See docs/knowledge-nodes.md.
 */
class Knowledge_article {

    /** Ids the page already uses for other markup. */
    const RESERVED_IDS = ['content', 'theme-toggle', 'lightbox'];
    const PART_PREFIX = 'part-';
    const MAX_SUMMARY_CHARS = 200;

    const NODE_FIELDS = ['id', 'title', 'summary', 'concepts', 'prerequisites', 'related',
        'search_terms', 'learning_objectives', 'misconceptions'];

    private string $dir;
    public function __construct(string $dir) {
        $this->dir = rtrim($dir, '/');
    }

    /**
     * The article with every node's body rendered, in reading order.
     *
     * Throws with every problem at once when the content is inconsistent: a
     * part naming a missing node, a prerequisite that comes later, a link to
     * an anchor that does not exist, and so on.
     */
    public function load(bool $render_bodies = true): array {
        $manifest_file = $this->dir . '/article.toml';
        $manifest = Front_matter::decode(file_get_contents($manifest_file), $manifest_file);

        $nodes = [];
        foreach ($this->node_files() as $file) {
            $nodes[] = $this->load_node($file, $render_bodies);
        }

        return $this->assemble($manifest, $nodes);
    }

    /** @return string[] */
    private function node_files(): array {
        $files = glob($this->dir . '/views/nodes/*.php');
        sort($files);
        return $files;
    }

    private function load_node(string $file, bool $render_body): array {
        [$front] = Front_matter::split(file_get_contents($file), $file);

        $unknown = array_diff(array_keys($front), self::NODE_FIELDS);
        if ($unknown !== []) {
            throw new RuntimeException($file . ': unknown field(s) ' . implode(', ', $unknown));
        }
        foreach (['id', 'title', 'summary', 'concepts'] as $required) {
            if (!isset($front[$required])) {
                throw new RuntimeException($file . ': frontmatter has no ' . $required);
            }
        }

        $stem = pathinfo($file, PATHINFO_FILENAME);
        if ($stem !== $front['id']) {
            throw new RuntimeException($file . ': id "' . $front['id'] . '" must match the file name, so a node is found where its id says');
        }

        $html = '';
        if ($render_body) {
            $html = Prose_view::html($file);
        }

        return [
            'id' => $front['id'],
            'title' => $front['title'],
            'summary' => $front['summary'],
            'concepts' => $front['concepts'],
            'prerequisites' => $front['prerequisites'] ?? [],
            'related' => $front['related'] ?? [],
            'search_terms' => $front['search_terms'] ?? [],
            'learning_objectives' => $front['learning_objectives'] ?? [],
            'misconceptions' => $front['misconceptions'] ?? [],
            'file' => $file,
            'body' => $html,
            'order' => 0,
            'builds_on' => [],
            'see_also' => [],
        ];
    }

    private function is_valid_id(string $id): bool {
        return preg_match('/^[a-z0-9]+(-[a-z0-9]+)*$/', $id) === 1;
    }

    private function assemble(array $manifest, array $nodes): array {
        $errors = [];

        $by_id = [];
        foreach ($nodes as $node) {
            $errors = array_merge($errors, $this->node_errors($node));
            $id = $node['id'];
            if (isset($by_id[$id])) {
                $errors[] = "node \"{$id}\": id is used by more than one file";
            }
            $by_id[$id] = $node;
        }

        $order = [];
        $parts = [];
        foreach ($manifest['part'] as $index => $part) {
            if ($part['nodes'] === []) {
                $errors[] = "part \"{$part['title']}\" lists no nodes";
            }
            $part_ids = [];
            foreach ($part['nodes'] as $id) {
                if (!isset($by_id[$id])) {
                    $errors[] = "part \"{$part['title']}\" lists \"{$id}\", which is not a node";
                    continue;
                }
                if (in_array($id, $order, true)) {
                    $errors[] = "node \"{$id}\" is listed in the article more than once";
                    continue;
                }
                $order[] = $id;
                $part_ids[] = $id;
            }
            $parts[] = [
                'id' => self::PART_PREFIX . ($index + 1),
                'title' => $part['title'],
                'node_ids' => $part_ids,
            ];
        }

        $unlisted = array_diff(array_keys($by_id), $order);
        sort($unlisted);
        foreach ($unlisted as $id) {
            $errors[] = "node \"{$id}\" is not listed in any part of article.toml";
        }

        $position = array_flip($order);
        $part_ids = array_column($parts, 'id');
        foreach ($order as $id) {
            $errors = array_merge($errors, $this->link_errors($by_id[$id], $by_id, $position, $part_ids, $manifest['path']));
        }

        $errors = array_merge($errors, $this->learning_path_errors($manifest['learning_path'] ?? [], $by_id));

        if ($errors !== []) {
            throw new RuntimeException("the AI article is invalid:\n  - " . implode("\n  - ", $errors));
        }

        $ordered = [];
        foreach ($order as $index => $id) {
            $node = $by_id[$id];
            $node['order'] = $index + 1;
            $node['builds_on'] = $this->links($node['prerequisites'], $by_id);
            $node['see_also'] = $this->links($node['related'], $by_id);
            $node['has_builds_on'] = $node['builds_on'] !== [];
            $node['has_see_also'] = $node['see_also'] !== [];
            $node['has_links'] = $node['has_builds_on'] || $node['has_see_also'];
            $node['has_misconceptions'] = $node['misconceptions'] !== [];
            $ordered[$id] = $node;
        }

        foreach ($parts as $index => $part) {
            $part_nodes = [];
            foreach ($part['node_ids'] as $id) {
                $part_nodes[] = $ordered[$id];
            }
            $parts[$index]['nodes'] = $part_nodes;
        }

        $published = DateTimeImmutable::createFromFormat('!Y-m-d', $manifest['published']);
        $updated = DateTimeImmutable::createFromFormat('!Y-m-d', $manifest['updated']);

        return [
            'path' => $manifest['path'],
            'url' => '/' . $manifest['path'] . '/',
            'title' => $manifest['title'],
            'description' => $manifest['description'],
            'published' => $manifest['published'],
            'published_iso' => $manifest['published'],
            'published_long' => $published->format('F j, Y'),
            'updated_iso' => $manifest['updated'],
            'updated_long' => $updated->format('F j, Y'),
            'was_updated' => $manifest['updated'] !== $manifest['published'],
            'parts' => $parts,
            'nodes' => array_values($ordered),
        ];
    }

    private function node_errors(array $node): array {
        $errors = [];
        $id = $node['id'];
        if (!$this->is_valid_id($id)) {
            $errors[] = "node \"{$id}\": ids are lowercase letters, digits and single hyphens";
        }
        if (in_array($id, self::RESERVED_IDS, true) || str_starts_with($id, self::PART_PREFIX)) {
            $errors[] = "node \"{$id}\": that id is reserved for other markup on the page";
        }
        if (trim($node['title']) === '') {
            $errors[] = "node \"{$id}\": title is empty";
        }
        if (trim($node['summary']) === '') {
            $errors[] = "node \"{$id}\": summary is empty";
        }
        $summary_chars = mb_strlen($node['summary']);
        if ($summary_chars > self::MAX_SUMMARY_CHARS) {
            $errors[] = "node \"{$id}\": summary is {$summary_chars} characters; keep it under "
                . self::MAX_SUMMARY_CHARS . ' so it reads as one line of the timeline';
        }
        if ($node['concepts'] === []) {
            $errors[] = "node \"{$id}\": list at least one concept";
        }
        if (preg_match('/<h[1-6][\s>]/', $node['body']) === 1) {
            $errors[] = "node \"{$id}\": the body contains a heading; the node title is the heading, so split the node instead";
        }
        return $errors;
    }

    private function link_errors(array $node, array $by_id, array $position, array $part_ids, string $path): array {
        $errors = [];
        $id = $node['id'];

        foreach (['prerequisites', 'related'] as $field) {
            $listed = [];
            foreach ($node[$field] as $target) {
                if ($target === $id) {
                    $errors[] = "node \"{$id}\": {$field} lists the node itself";
                } elseif (!isset($by_id[$target])) {
                    $errors[] = "node \"{$id}\": {$field} lists \"{$target}\", which is not a node";
                } elseif (isset($listed[$target])) {
                    $errors[] = "node \"{$id}\": {$field} lists \"{$target}\" twice";
                } elseif ($field === 'prerequisites' && isset($position[$target], $position[$id]) && $position[$target] > $position[$id]) {
                    $errors[] = "node \"{$id}\": prerequisite \"{$target}\" comes later in the article; move one of them, or make it `related` instead";
                }
                $listed[$target] = true;
            }
        }

        foreach ($node['related'] as $target) {
            if (in_array($target, $node['prerequisites'], true)) {
                $errors[] = "node \"{$id}\": \"{$target}\" is both a prerequisite and related; keep it in one list";
            }
        }

        foreach ($this->page_anchors($node['body'], $path) as $anchor) {
            if (!isset($by_id[$anchor]) && !in_array($anchor, $part_ids, true)) {
                $errors[] = "node \"{$id}\": links to #{$anchor}, which is not a node or part";
            }
        }

        return $errors;
    }

    private function learning_path_errors(array $paths, array $by_id): array {
        $errors = [];
        $seen = [];
        foreach ($paths as $path) {
            $id = $path['id'];
            if (!$this->is_valid_id($id)) {
                $errors[] = "learning path \"{$id}\": ids are lowercase letters, digits and hyphens";
            }
            if (isset($seen[$id])) {
                $errors[] = "learning path \"{$id}\" is defined more than once";
            }
            $seen[$id] = true;
            if ($path['nodes'] === []) {
                $errors[] = "learning path \"{$id}\" lists no nodes";
            }
            $listed = [];
            foreach ($path['nodes'] as $node_id) {
                if (!isset($by_id[$node_id])) {
                    $errors[] = "learning path \"{$id}\" lists \"{$node_id}\", which is not a node";
                } elseif (isset($listed[$node_id])) {
                    $errors[] = "learning path \"{$id}\" lists \"{$node_id}\" twice";
                }
                $listed[$node_id] = true;
            }
        }
        return $errors;
    }

    /**
     * Each link with the separator that follows it in a sentence: a comma
     * after every one but the last.
     */
    private function links(array $ids, array $by_id): array {
        $links = [];
        $last = count($ids) - 1;
        foreach ($ids as $index => $id) {
            $links[] = [
                'id' => $id,
                'title' => $by_id[$id]['title'],
                'separator' => $index < $last ? ', ' : '',
            ];
        }
        return $links;
    }

    /**
     * Same-page anchors a rendered body links to: href="#x" and
     * href="/blog/ai/#x".
     */
    private function page_anchors(string $html, string $path): array {
        preg_match_all('~href="(?:/' . preg_quote($path, '~') . '/)?#([^"]*)"~', $html, $matches);
        return $matches[1];
    }

}
