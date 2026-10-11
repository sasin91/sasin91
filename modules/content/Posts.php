<?php
require_once __DIR__ . '/Front_matter.php';

/**
 * The blog posts: one view each under modules/blog/views/posts, newest first.
 *
 * Loading reads every header but renders no body; a page that shows one post
 * renders that one (Content::post).
 */
class Posts {

    private string $dir;

    public function __construct(string $dir) {
        $this->dir = rtrim($dir, '/');
    }

    /**
     * Every post, newest first, with one exception: a series is kept together
     * and read in part order, and sits in the list at the date of its newest
     * part. Ties outside a series break on path, so the order is stable.
     *
     * @return array<int, array>
     */
    public function all(): array {
        $posts = [];
        foreach ($this->files() as $file) {
            $posts[] = $this->load($file);
        }

        $anchors = $this->series_anchors($posts);
        foreach ($posts as $index => $post) {
            $posts[$index]['listed_date'] = $anchors[$post['series']] ?? $post['date'];
        }

        usort($posts, [$this, 'compare']);
        return $posts;
    }

    private function compare(array $a, array $b): int {
        return [$b['listed_date'], $a['series'] ?? '', $a['part'] ?? 0, $a['path']]
            <=> [$a['listed_date'], $b['series'] ?? '', $b['part'] ?? 0, $b['path']];
    }

    /**
     * The date of each series' newest part.
     */
    private function series_anchors(array $posts): array {
        $anchors = [];
        foreach ($posts as $post) {
            $series = $post['series'];
            if ($series === null) {
                continue;
            }
            $newest = $anchors[$series] ?? '';
            if ($post['date'] > $newest) {
                $anchors[$series] = $post['date'];
            }
        }
        return $anchors;
    }

    /** @return string[] */
    private function files(): array {
        $files = [];
        $walk = new RecursiveIteratorIterator(new RecursiveDirectoryIterator($this->dir, FilesystemIterator::SKIP_DOTS));
        foreach ($walk as $entry) {
            if ($entry->getExtension() === 'php') {
                $files[] = str_replace('\\', '/', $entry->getPathname());
            }
        }
        sort($files);
        return $files;
    }

    private function load(string $file): array {
        [$front] = Front_matter::split(file_get_contents($file), $file);

        foreach (['path', 'title', 'date', 'description'] as $required) {
            if (!isset($front[$required])) {
                throw new RuntimeException($file . ': frontmatter has no ' . $required);
            }
        }

        $date = DateTimeImmutable::createFromFormat('!Y-m-d', $front['date'], new DateTimeZone('UTC'));

        return [
            'file' => $file,
            'path' => $front['path'],
            'url' => '/' . $front['path'] . '/',
            'title' => $front['title'],
            'date' => $front['date'],
            'date_iso' => $front['date'],
            'date_long' => $date->format('F j, Y'),
            'date_rfc2822' => $date->format('D, j M Y H:i:s O'),
            'description' => $front['description'],
            'hero' => $front['hero'] ?? null,
            'hero_alt' => $front['hero_alt'] ?? null,
            'card' => $front['card'] ?? null,
            'card_alt' => $front['card_alt'] ?? null,
            'series' => $front['series'] ?? null,
            'part' => $front['part'] ?? null,
            'ask' => $front['ask'] ?? [],
        ];
    }

}
