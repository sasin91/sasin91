<?php
require_once APPPATH . 'modules/content/Highlighter.php';

/**
 * A block of code, highlighted, with a file name bar when it has a title.
 * The slot is the code itself, written as it is: the component escapes it.
 */
class Code {

    public function data(array $props, string $slot): array {
        // The slot starts on the line after open_component() and ends on the
        // line before close_component(); those two line breaks are not code.
        $code = preg_replace('/^\r?\n/', '', $slot);
        $code = rtrim($code) . "\n";

        $title = $props['title'] ?? null;
        return [
            'highlighted' => (new Highlighter())->to_html($code, $props['language'] ?? ''),
            'title' => $title,
            'has_title' => $title !== null,
        ];
    }

}
