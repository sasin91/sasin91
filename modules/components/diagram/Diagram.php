<?php
/**
 * A local SVG, inlined so its colours follow the page's theme toggle, with
 * its description as the figure's accessible name.
 */
class Diagram {

    public function data(array $props, string $slot): array {
        $file = APPPATH . 'public/' . ltrim($props['src'], '/');
        if (!is_file($file)) {
            throw new RuntimeException('missing SVG referenced by post: ' . $props['src']);
        }
        return ['alt' => $props['alt'], 'svg' => file_get_contents($file)];
    }

}
