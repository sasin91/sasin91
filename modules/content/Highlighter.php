<?php
/**
 * Fenced code to highlighted HTML, through scrivo/highlight.php.
 *
 * Every block comes out in the same wrapper, <pre class="hl-code"><code>,
 * whatever its language, so site.css frames them all alike and a page can
 * tell it needs syntax.css by looking for that wrapper. A language the
 * highlighter does not know (caddy, text, none at all) is escaped as plain
 * text inside the same wrapper.
 */
class Highlighter {

    private \Highlight\Highlighter $highlighter;

    public function __construct() {
        $this->highlighter = new \Highlight\Highlighter();
    }

    public function to_html(string $code, string $language): string {
        $body = $this->highlighted($code, $language);
        return '<pre class="hl-code"><code>' . $body . '</code></pre>';
    }

    private function highlighted(string $code, string $language): string {
        if ($language === '') {
            return htmlspecialchars($code, ENT_QUOTES);
        }

        try {
            return $this->highlighter->highlight($language, $code)->value;
        } catch (DomainException) {
            return htmlspecialchars($code, ENT_QUOTES);
        }
    }

}
