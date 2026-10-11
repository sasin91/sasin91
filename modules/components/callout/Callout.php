<?php
require_once __DIR__ . '/../section/Section.php';

/**
 * An aside set apart from the text: kind "note" or "warning", with an
 * optional title that is linkable like a section's.
 */
class Callout {

    const KINDS = ['note', 'warning'];

    public function data(array $props, string $slot): array {
        $kind = $props['kind'] ?? 'note';
        if (!in_array($kind, self::KINDS, true)) {
            throw new InvalidArgumentException("a callout is one of " . implode(', ', self::KINDS) . ", not \"{$kind}\"");
        }

        $title = $props['title'] ?? null;
        $id = null;
        if ($title !== null) {
            $id = Section::id(html_entity_decode(strip_tags($title)));
        }

        return ['kind' => $kind, 'title' => $title, 'id' => $id, 'has_title' => $title !== null, 'slot' => $slot];
    }

}
