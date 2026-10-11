<?php
/**
 * Components for views, in the spirit of Vue's: a name, props, and a default
 * slot, each defined in one place and used from any .phtml view.
 *
 * A component is a folder under modules/components/:
 *
 *   callout/
 *     Callout.php      optional: turns props (and the slot) into view data
 *     callout.phtml    the template, given that data
 *
 * Without a class the props go to the template as they are. With one, its
 * data() method is the only place a component may compute anything, so the
 * template stays markup and a breakpoint in data() shows every input.
 *
 * Views use the three functions in components_helper.php:
 *
 *   <?= component('math', ['tex' => 'e = w(1 + r/30)']) ?>
 *
 *   <?php open_component('callout', ['kind' => 'warning']) ?>
 *     <p>Anything here is the slot.</p>
 *   <?php close_component() ?>
 *
 * This module answers no URL: it has no Components.php controller, so
 * /components/... is a 404, and it needs no framework to run, so a
 * command-line script can render a post too.
 */
class Component {

    /** Components opened and not yet closed, innermost last. */
    private static array $open = [];

    /**
     * Renders component $name with $props and $slot (HTML), and returns it.
     */
    public static function render(string $name, array $props = [], string $slot = ''): string {
        $dir = self::dir($name);
        $data = $props;

        $component = self::instance($name);
        if ($component !== null) {
            $data = $component->data($props, $slot);
        }
        $data['slot'] = $data['slot'] ?? $slot;

        return self::template($dir . '/' . $name . '.phtml', $data);
    }

    /**
     * Starts capturing a slot: everything the view outputs until close() is
     * the component's content.
     */
    public static function open(string $name, array $props = []): void {
        $component = self::instance($name);
        // A component that must act where it starts, not where it ends (a
        // section takes its id before the callouts inside it take theirs),
        // does so in opened().
        if ($component !== null && method_exists($component, 'opened')) {
            $props = $component->opened($props);
        }
        self::$open[] = ['name' => $name, 'props' => $props];
        ob_start();
    }

    public static function close(): string {
        if (self::$open === []) {
            throw new LogicException('close_component() without an open_component()');
        }
        $slot = ob_get_clean();
        $component = array_pop(self::$open);
        return self::render($component['name'], $component['props'], $slot);
    }

    /**
     * Renders any .phtml view with $data as its variables, the way the
     * components' templates and the posts written with them are rendered.
     */
    public static function template(string $file, array $data = []): string {
        extract($data);
        ob_start();
        require $file;
        return ob_get_clean();
    }

    /** The component's class, or null when it is a template alone. */
    private static function instance(string $name): ?object {
        $class_file = self::dir($name) . '/' . ucfirst($name) . '.php';
        if (!is_file($class_file)) {
            return null;
        }
        require_once $class_file;
        $class = ucfirst($name);
        return new $class();
    }

    private static function dir(string $name): string {
        if (preg_match('/^[a-z][a-z0-9_]*$/', $name) !== 1) {
            throw new InvalidArgumentException("\"{$name}\" is not a component name");
        }
        $dir = __DIR__ . '/' . $name;
        if (!is_dir($dir)) {
            throw new InvalidArgumentException("there is no component \"{$name}\" (modules/components/{$name}/)");
        }
        return $dir;
    }

}
