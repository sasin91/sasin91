<?php
/**
 * A heading and everything under it, as one <section> carrying the heading's
 * id, so it can be linked to (#1-Create-the-server) and styled as a unit.
 */
class Section {

    /** Ids handed out in the page being rendered; see Section::reset_ids(). */
    private static array $used_ids = [];

    /** Called before rendering a page, so ids repeat only within one page. */
    public static function reset_ids(): void {
        self::$used_ids = [];
    }

    /**
     * The id the published pages always had: the title's letters, digits,
     * hyphens and underscores, each run of whitespace a hyphen, everything
     * else dropped. A repeat gets -1, -2 after it.
     */
    public static function id(string $title): string {
        $kept = preg_replace('/[^\p{L}\p{N}_\-\s]/u', '', $title);
        $id = implode('-', preg_split('/\s+/u', trim($kept), -1, PREG_SPLIT_NO_EMPTY));

        $unique = $id;
        $number = 1;
        while (isset(self::$used_ids[$unique])) {
            $unique = $id . '-' . $number;
            $number++;
        }
        self::$used_ids[$unique] = true;
        return $unique;
    }

    /** The id is taken when the section opens, so ids follow the page's order. */
    public function opened(array $props): array {
        $props['id'] = self::id(html_entity_decode(strip_tags($props['title'])));
        return $props;
    }

    public function data(array $props, string $slot): array {
        $level = $props['level'] ?? 2;
        return [
            'id' => $props['id'] ?? self::id(html_entity_decode(strip_tags($props['title']))),
            'title' => $props['title'],
            'tag' => 'h' . $level,
            'slot' => $slot,
        ];
    }

}
