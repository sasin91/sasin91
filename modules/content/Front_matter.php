<?php
use Devium\Toml\Toml;
use Devium\Toml\TomlLocalDate;

/**
 * The +++ TOML +++ header that starts every post and knowledge node.
 */
class Front_matter {

    /**
     * Splits a file into its decoded header and its Markdown body.
     *
     * @return array{0: array, 1: string}
     */
    public static function split(string $source, string $file): array {
        if (!str_starts_with($source, '+++')) {
            throw new RuntimeException($file . ': file must start with a +++ frontmatter block');
        }

        $rest = substr($source, 3);
        $end = strpos($rest, '+++');
        if ($end === false) {
            throw new RuntimeException($file . ': frontmatter block is never closed');
        }

        $header = self::decode(substr($rest, 0, $end), $file);
        $body = substr($rest, $end + 3);
        return [$header, $body];
    }

    /**
     * TOML to an array, with every TOML date turned into a Y-m-d string.
     */
    public static function decode(string $toml, string $file): array {
        try {
            $data = Toml::decode($toml, true);
        } catch (Throwable $error) {
            throw new RuntimeException($file . ': invalid TOML: ' . $error->getMessage());
        }
        return self::dates_to_strings($data);
    }

    private static function dates_to_strings(array $data): array {
        foreach ($data as $key => $value) {
            if ($value instanceof TomlLocalDate) {
                $data[$key] = sprintf('%04d-%02d-%02d', $value->year, $value->month, $value->day);
                continue;
            }
            if (is_array($value)) {
                $data[$key] = self::dates_to_strings($value);
            }
        }
        return $data;
    }

}
