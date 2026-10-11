<?php
require_once __DIR__ . '/Front_matter.php';

/**
 * content/cv.toml: the person behind the site. The /cv page, the /about page,
 * the home page header and the CV PDF all read from this one file, so they
 * cannot disagree.
 */
class Cv_data {

    public static function load(string $file): array {
        $cv = Front_matter::decode(file_get_contents($file), $file);

        $cv['education_note'] = $cv['education_note'] ?? null;
        $cv['site']['stack_line'] = implode(' · ', $cv['site']['stack']);
        $cv['contact']['phone_href'] = str_replace(' ', '', $cv['contact']['phone']);

        foreach ($cv['roles'] as $index => $role) {
            $where = 'role ' . $role['title'] . ' at ' . $role['company'];
            $role['start_label'] = self::month($role['start'], $where . ': start date');
            $role['end'] = $role['end'] ?? null;
            $role['end_label'] = self::optional_month($role['end'], $where . ': end date');
            $role['end_text'] = $role['end_label'] ?? 'present';
            $role['achievements'] = $role['achievements'] ?? [];
            $role['achievements_html'] = array_map([self::class, 'achievement_html'], $role['achievements']);
            $role['note'] = $role['note'] ?? null;
            $cv['roles'][$index] = $role;
        }

        foreach ($cv['education'] as $index => $education) {
            $where = 'education ' . $education['title'] . ' at ' . $education['school'];
            $education['start_label'] = self::month($education['start'], $where . ': start date');
            $education['end'] = $education['end'] ?? null;
            $education['end_label'] = self::optional_month($education['end'], $where . ': end date');
            $education['end_text'] = $education['end_label'] ?? 'present';
            $education['note'] = $education['note'] ?? null;
            $cv['education'][$index] = $education;
        }

        return $cv;
    }

    /**
     * "2024-09" as "September 2024". Anything else is an error in the file.
     */
    private static function month(string $value, string $where): string {
        $date = DateTimeImmutable::createFromFormat('!Y-m', $value);
        $errors = DateTimeImmutable::getLastErrors();
        $invalid = $date === false || ($errors !== false && $errors['warning_count'] + $errors['error_count'] > 0);
        if ($invalid) {
            throw new RuntimeException("content/cv.toml has a bad date: {$where}: invalid date \"{$value}\" (want YYYY-MM)");
        }
        return $date->format('F Y');
    }

    private static function optional_month(?string $value, string $where): ?string {
        if ($value === null) {
            return null;
        }
        return self::month($value, $where);
    }

    /**
     * An achievement, escaped, with the one flourish the page has: the 6 and
     * 7 of "version 6 to 7" get spans the stylesheet animates.
     */
    private static function achievement_html(string $achievement): string {
        $escaped = htmlspecialchars($achievement, ENT_QUOTES);
        return str_replace(
            'version 6 to 7',
            'version <span class="cv-jump cv-jump-six">6</span> to <span class="cv-jump cv-jump-seven">7</span>',
            $escaped
        );
    }

}
