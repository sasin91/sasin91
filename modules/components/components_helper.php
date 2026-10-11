<?php
require_once __DIR__ . '/Component.php';

/**
 * The view-side names for Component. See Component.php.
 */

/** A component with no content of its own: returns its HTML. */
function component(string $name, array $props = []): string {
    return Component::render($name, $props);
}

/** Starts a component whose content (its slot) follows in the view. */
function open_component(string $name, array $props = []): void {
    Component::open($name, $props);
}

/** Ends the innermost open component and outputs it. */
function close_component(): void {
    echo Component::close();
}

/** Shorthand for the math component, since it sits inside sentences. */
function math(string $tex, bool $display = false): string {
    return Component::render('math', ['tex' => $tex, 'display' => $display]);
}
