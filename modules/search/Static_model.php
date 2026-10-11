<?php
/**
 * The static embedding model search runs on (models/potion-base-4M): a
 * WordPiece vocabulary and one 8-bit row per token. A text's embedding is
 * the mean of its known tokens' rows, normalised to unit length.
 *
 * This is the same tokenizer and arithmetic as StaticModel in
 * modules/search/js/retrieval.mjs, which embeds the reader's question in the
 * browser. The two must agree, or a question would be compared against
 * passages embedded some other way; models/potion-base-4M/wordpiece-fixture.json
 * is checked by both test suites for that reason.
 */
class Static_model {

    const UNK = '[UNK]';
    const MAX_WORD_CHARS = 100;

    /** @var array<string, int> token => id */
    private array $vocab = [];
    /** @var float[] one scale per row */
    private array $scales;
    /** The i8 rows, left packed: unpacked they would take tens of megabytes. */
    private string $weights;
    private int $dim;

    public function __construct(string $vocab_text, string $bytes) {
        $tokens = explode("\n", rtrim($vocab_text, "\n"));
        $rows = count($tokens);
        $weight_bytes = strlen($bytes) - $rows * 4;
        if ($weight_bytes <= 0 || $weight_bytes % $rows !== 0) {
            throw new RuntimeException('embeddings of ' . strlen($bytes) . " bytes do not fit {$rows} tokens");
        }

        $this->dim = intdiv($weight_bytes, $rows);
        $this->vocab = array_flip($tokens);
        $this->scales = array_values(unpack('g' . $rows, $bytes));
        $this->weights = substr($bytes, $rows * 4);
    }

    public static function from_dir(string $dir): self {
        $dir = rtrim($dir, '/');
        return new self(file_get_contents($dir . '/vocab.txt'), file_get_contents($dir . '/embeddings.q8.bin'));
    }

    public function dim(): int {
        return $this->dim;
    }

    /**
     * The unit-length embedding of $text, or null when none of its tokens is
     * in the vocabulary (a line of emoji, say): such a text could never be
     * found by meaning, so it gets no vector rather than a meaningless one.
     *
     * @return float[]|null
     */
    public function encode(string $text): ?array {
        $sum = array_fill(0, $this->dim, 0.0);
        $known = 0;

        foreach ($this->tokenize($text) as $token) {
            if ($token === self::UNK) {
                continue;
            }
            $id = $this->vocab[$token];
            $scale = $this->scales[$id];
            $row = unpack('c' . $this->dim, $this->weights, $id * $this->dim);
            $i = 0;
            foreach ($row as $weight) {
                $sum[$i] += $weight * $scale;
                $i++;
            }
            $known++;
        }

        if ($known === 0) {
            return null;
        }

        $length = 0.0;
        foreach ($sum as $value) {
            $length += $value * $value;
        }
        $length = sqrt($length);

        $vector = [];
        foreach ($sum as $value) {
            $vector[] = $value / $length;
        }
        return $vector;
    }

    /** @return string[] */
    public function tokenize(string $text): array {
        $tokens = [];
        foreach ($this->pre_tokenize($this->normalize($text)) as $word) {
            foreach ($this->word_piece($word) as $piece) {
                $tokens[] = $piece;
            }
        }
        return $tokens;
    }

    /**
     * Clean, split CJK, strip accents, lowercase: the BERT normaliser.
     */
    public function normalize(string $text): string {
        $cleaned = '';
        foreach (mb_str_split($text) as $char) {
            $code = mb_ord($char);
            if ($code === 0 || $code === 0xFFFD || $this->is_control($char)) {
                continue;
            }
            if (preg_match('/^\s$/u', $char) === 1) {
                $cleaned .= ' ';
            } elseif ($this->is_cjk($code)) {
                $cleaned .= ' ' . $char . ' ';
            } else {
                $cleaned .= $char;
            }
        }

        $decomposed = Normalizer::normalize($cleaned, Normalizer::FORM_D);
        $unaccented = preg_replace('/\p{Mn}/u', '', $decomposed);
        return mb_strtolower($unaccented);
    }

    /**
     * Normalised text split on whitespace, every punctuation mark its own word.
     *
     * @return string[]
     */
    public function pre_tokenize(string $normalized): array {
        $words = [];
        foreach (preg_split('/\s/u', $normalized) as $chunk) {
            $word = '';
            foreach (mb_str_split($chunk) as $char) {
                if ($this->is_punctuation($char)) {
                    if ($word !== '') {
                        $words[] = $word;
                    }
                    $words[] = $char;
                    $word = '';
                    continue;
                }
                $word .= $char;
            }
            if ($word !== '') {
                $words[] = $word;
            }
        }
        return $words;
    }

    /**
     * Greedy longest-match-first WordPiece. A word the vocabulary cannot
     * cover completely becomes a single [UNK].
     *
     * @return string[]
     */
    private function word_piece(string $word): array {
        $chars = mb_str_split($word);
        $length = count($chars);
        if ($length > self::MAX_WORD_CHARS) {
            return [self::UNK];
        }

        $pieces = [];
        $start = 0;
        while ($start < $length) {
            $end = $length;
            $piece = null;
            while ($start < $end) {
                $body = implode('', array_slice($chars, $start, $end - $start));
                $candidate = $start > 0 ? '##' . $body : $body;
                if (isset($this->vocab[$candidate])) {
                    $piece = $candidate;
                    break;
                }
                $end--;
            }
            if ($piece === null) {
                return [self::UNK];
            }
            $pieces[] = $piece;
            $start = $end;
        }
        return $pieces;
    }

    private function is_control(string $char): bool {
        if ($char === "\t" || $char === "\n" || $char === "\r") {
            return false;
        }
        return preg_match('/^\p{C}$/u', $char) === 1;
    }

    /** CJK ideographs are split into one token each, as BERT does. */
    private function is_cjk(int $code): bool {
        return ($code >= 0x4E00 && $code <= 0x9FFF)
            || ($code >= 0x3400 && $code <= 0x4DBF)
            || ($code >= 0x20000 && $code <= 0x2A6DF)
            || ($code >= 0x2A700 && $code <= 0x2B73F)
            || ($code >= 0x2B740 && $code <= 0x2B81F)
            || ($code >= 0x2B820 && $code <= 0x2CEAF)
            || ($code >= 0xF900 && $code <= 0xFAFF)
            || ($code >= 0x2F800 && $code <= 0x2FA1F);
    }

    /**
     * Unicode punctuation, and the ASCII symbols ($ + < = > ^ ` | ~) BERT
     * treats as punctuation although Unicode does not.
     */
    private function is_punctuation(string $char): bool {
        $code = mb_ord($char);
        $ascii_symbol = ($code >= 33 && $code <= 47)
            || ($code >= 58 && $code <= 64)
            || ($code >= 91 && $code <= 96)
            || ($code >= 123 && $code <= 126);
        return $ascii_symbol || preg_match('/^\p{P}$/u', $char) === 1;
    }

}
