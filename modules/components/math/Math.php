<?php
use Latex2MathML\Converter as Latex2MathML;

/**
 * TeX as MathML, which every current browser renders natively, rather than
 * markup a script has to find and typeset later. A formula that does not
 * convert shows as its source, marked, rather than breaking the page.
 */
class Math {

    public function data(array $props, string $slot): array {
        $tex = $props['tex'];
        $display = ($props['display'] ?? false) ? 'block' : 'inline';

        try {
            return ['mathml' => Latex2MathML::convert($tex, $display), 'error' => null, 'tex' => $tex];
        } catch (Throwable $error) {
            return ['mathml' => null, 'error' => $error->getMessage(), 'tex' => $tex];
        }
    }

}
