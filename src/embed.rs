//! A static embedding model, run at build time to embed the AI article's
//! passages for the Ask view.
//!
//! The model is Model2Vec's potion-base (see models/README.md): a table with
//! one vector per WordPiece token. A text's embedding is the average of its
//! tokens' rows, normalised to unit length. There is no network to run, which
//! is the whole reason it was chosen: the browser does the identical lookup
//! for the reader's question in `js/retrieval.mjs`, with no ML runtime.
//!
//! The tokenizer here and the one in `js/retrieval.mjs` must agree exactly,
//! or a question and a passage would be embedded from different tokens. Both
//! test suites check themselves against the same reference fixture, generated
//! with Hugging Face's own tokenizer: models/potion-base-4M/wordpiece-fixture.json.

use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::{Result, ensure};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

const UNK: &str = "[UNK]";
const MAX_WORD_CHARS: usize = 100;

static CONTROL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{C}$").unwrap());
static MARK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{Mn}$").unwrap());
static PUNCTUATION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{P}$").unwrap());

fn matches(re: &Regex, c: char) -> bool {
    re.is_match(c.encode_utf8(&mut [0; 4]))
}

fn is_control(c: char) -> bool {
    !matches!(c, '\t' | '\n' | '\r') && matches(&CONTROL, c)
}

/// CJK ideographs, which BERT splits into one token each.
fn is_cjk(c: char) -> bool {
    matches!(u32::from(c),
        0x4E00..=0x9FFF
        | 0x3400..=0x4DBF
        | 0x20000..=0x2A6DF
        | 0x2A700..=0x2B73F
        | 0x2B740..=0x2B81F
        | 0x2B820..=0x2CEAF
        | 0xF900..=0xFAFF
        | 0x2F800..=0x2FA1F)
}

/// ASCII symbols such as `$ + < = > ^ | ~` are not Unicode punctuation, but
/// BERT treats them as punctuation anyway.
fn is_punctuation(c: char) -> bool {
    c.is_ascii_punctuation() || matches(&PUNCTUATION, c)
}

/// Clean, split CJK, strip accents, lowercase: the BERT normaliser.
pub fn normalize(text: &str) -> String {
    let mut cleaned = String::with_capacity(text.len());
    for c in text.chars() {
        if c == '\0' || c == '\u{fffd}' || is_control(c) {
            continue;
        }
        if c.is_whitespace() {
            cleaned.push(' ');
        } else if is_cjk(c) {
            cleaned.push(' ');
            cleaned.push(c);
            cleaned.push(' ');
        } else {
            cleaned.push(c);
        }
    }
    cleaned
        .nfd()
        .filter(|&c| !matches(&MARK, c))
        .collect::<String>()
        .to_lowercase()
}

/// Split normalised text on whitespace, isolating every punctuation mark.
pub fn pre_tokenize(normalized: &str) -> Vec<String> {
    let mut words = Vec::new();
    for chunk in normalized.split(char::is_whitespace) {
        let mut word = String::new();
        for c in chunk.chars() {
            if is_punctuation(c) {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
                words.push(c.to_string());
            } else {
                word.push(c);
            }
        }
        if !word.is_empty() {
            words.push(word);
        }
    }
    words
}

pub struct StaticModel {
    vocab: HashMap<String, usize>,
    scales: Vec<f32>,
    weights: Vec<i8>,
    dim: usize,
}

impl StaticModel {
    /// Loads `vocab.txt` and `embeddings.q8.bin` from a model directory.
    /// The build reads the files itself, because it also copies them out
    /// under hashed names; this is for tests.
    #[cfg(test)]
    pub fn load(dir: &std::path::Path) -> Result<Self> {
        use anyhow::Context;
        let vocab = std::fs::read_to_string(dir.join("vocab.txt"))
            .with_context(|| format!("reading {}/vocab.txt", dir.display()))?;
        let bytes = std::fs::read(dir.join("embeddings.q8.bin"))
            .with_context(|| format!("reading {}/embeddings.q8.bin", dir.display()))?;
        Self::from_parts(&vocab, &bytes)
    }

    /// `bytes` is the layout `tools/quantize-model.mjs` writes: one
    /// little-endian f32 scale per token, then `dim` i8 weights per token.
    pub fn from_parts(vocab_text: &str, bytes: &[u8]) -> Result<Self> {
        let tokens: Vec<&str> = vocab_text
            .strip_suffix('\n')
            .unwrap_or(vocab_text)
            .split('\n')
            .collect();
        let rows = tokens.len();
        ensure!(
            bytes.len() > rows * 4 && (bytes.len() - rows * 4).is_multiple_of(rows),
            "embeddings of {} bytes do not fit {rows} tokens",
            bytes.len()
        );
        let dim = (bytes.len() - rows * 4) / rows;

        let scales = bytes[..rows * 4]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&b| f32::from_le_bytes(b))
            .collect();
        let weights = bytes[rows * 4..].iter().map(|&b| b as i8).collect();
        let vocab = tokens
            .iter()
            .enumerate()
            .map(|(id, token)| (token.to_string(), id))
            .collect();

        Ok(Self {
            vocab,
            scales,
            weights,
            dim,
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Greedy longest-match-first WordPiece over one pre-tokenized word. A
    /// word the vocabulary cannot fully cover becomes a single `[UNK]`.
    fn word_piece(&self, word: &str) -> Vec<String> {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() > MAX_WORD_CHARS {
            return vec![UNK.to_string()];
        }

        let mut pieces = Vec::new();
        let mut start = 0;
        while start < chars.len() {
            let mut end = chars.len();
            let mut piece = None;
            while start < end {
                let body: String = chars[start..end].iter().collect();
                let candidate = if start > 0 { format!("##{body}") } else { body };
                if self.vocab.contains_key(&candidate) {
                    piece = Some(candidate);
                    break;
                }
                end -= 1;
            }
            match piece {
                Some(p) => pieces.push(p),
                None => return vec![UNK.to_string()],
            }
            start = end;
        }
        pieces
    }

    /// WordPiece tokens of `text`, including any `[UNK]`.
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        pre_tokenize(&normalize(text))
            .iter()
            .flat_map(|word| self.word_piece(word))
            .collect()
    }

    /// Mean of the known tokens' rows, normalised to unit length. Unknown
    /// tokens are dropped, as Model2Vec does. `None` when no token is known:
    /// a zero vector would be similar to nothing, and is better reported than
    /// written into the index.
    pub fn encode(&self, text: &str) -> Option<Vec<f32>> {
        let mut sum = vec![0f32; self.dim];
        let mut known = 0;
        for token in self.tokenize(text) {
            if token == UNK {
                continue;
            }
            let id = self.vocab[&token];
            let scale = self.scales[id];
            let row = &self.weights[id * self.dim..(id + 1) * self.dim];
            for (s, &w) in sum.iter_mut().zip(row) {
                *s += f32::from(w) * scale;
            }
            known += 1;
        }
        if known == 0 {
            return None;
        }
        let length = sum.iter().map(|v| v * v).sum::<f32>().sqrt();
        Some(sum.into_iter().map(|v| v / length).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const MODEL: &str = "models/potion-base-4M";

    fn model() -> StaticModel {
        StaticModel::load(Path::new(MODEL)).expect("the committed model must load")
    }

    #[derive(serde::Deserialize)]
    struct Case {
        text: String,
        tokens: Vec<String>,
    }

    /// The fixture was produced by Hugging Face's reference tokenizer from the
    /// model's own tokenizer.json. js/retrieval.test.mjs checks the browser
    /// implementation against the same file, so passing both is what keeps
    /// the build-time and in-browser embeddings in one vector space.
    #[test]
    fn tokenizes_exactly_like_the_reference_tokenizer() {
        let model = model();
        let cases: Vec<Case> = serde_json::from_str(
            &std::fs::read_to_string(format!("{MODEL}/wordpiece-fixture.json")).unwrap(),
        )
        .unwrap();
        assert!(cases.len() >= 10, "fixture looks truncated");
        for case in cases {
            assert_eq!(
                model.tokenize(&case.text),
                case.tokens,
                "for {:?}",
                case.text
            );
        }
    }

    #[test]
    fn encodes_to_a_unit_vector_of_the_model_dimension() {
        let model = model();
        let v = model.encode("What does a KV cache store?").unwrap();
        assert_eq!(v.len(), model.dim());
        let length: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((length - 1.0).abs() < 1e-4, "length {length}");
    }

    #[test]
    fn text_with_no_known_tokens_has_no_embedding() {
        assert!(model().encode("🚀 🚀").is_none());
        assert!(model().encode("").is_none());
    }

    #[test]
    fn unknown_tokens_do_not_change_the_embedding() {
        let model = model();
        assert_eq!(
            model.encode("context window").unwrap(),
            model.encode("context 🚀 window").unwrap()
        );
    }

    #[test]
    fn related_text_is_closer_than_unrelated_text() {
        let model = model();
        let cos = |a: &str, b: &str| -> f32 {
            let (a, b) = (model.encode(a).unwrap(), model.encode(b).unwrap());
            a.iter().zip(&b).map(|(x, y)| x * y).sum()
        };
        let near = cos(
            "sampling the next token",
            "choosing a word at random by probability",
        );
        let far = cos(
            "sampling the next token",
            "deploying a static site with rsync",
        );
        assert!(near > far, "near {near} vs far {far}");
    }

    #[test]
    fn rejects_a_weights_file_that_does_not_fit_the_vocabulary() {
        assert!(StaticModel::from_parts("a\nb\nc\n", &[0; 13]).is_err());
        let ok = StaticModel::from_parts("a\nb\n", &[0; 2 * 4 + 2 * 3]).unwrap();
        assert_eq!(ok.dim(), 3);
    }
}
