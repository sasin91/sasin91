//! The static search index behind the site's search: the AI article's
//! knowledge nodes and every blog post, each with its metadata, the words of
//! its body for keyword matching, and one embedding per passage. Built once
//! per build and written as `public/search/index.<hash>.json`.
//!
//! The browser downloads it only when a reader opens the search palette or
//! uses /blog/ai/ask/, and ranks documents with `js/retrieval.mjs`. The index does
//! not carry full text, which is already on the pages: just a short snippet
//! per passage, so a result can show which part of a document matched.

use anyhow::Result;
use serde::Serialize;

use crate::content::Post;
use crate::djot;
use crate::embed::StaticModel;
use crate::knowledge::{Article, Node};

/// Passage snippets are cut to about this many characters.
const SNIPPET_CHARS: usize = 160;

/// Four decimals is plenty for cosine similarity between unit vectors (the
/// error is far below the differences that decide a ranking) and keeps the
/// JSON roughly half the size of full f32 precision.
const VECTOR_DECIMALS: f64 = 1e4;

#[derive(Serialize)]
pub struct ModelInfo {
    pub name: String,
    /// Root-relative URLs of the hashed model files.
    pub weights_url: String,
    pub vocab_url: String,
    /// Bytes the reader downloads, so the page can say so before it does.
    pub bytes: usize,
    pub dim: usize,
}

#[derive(Serialize)]
struct Index<'a> {
    version: u32,
    model: &'a ModelInfo,
    /// Every searchable document. Named for the knowledge nodes that came
    /// first; posts are documents of `kind` "post".
    nodes: Vec<Document>,
}

#[derive(Serialize)]
struct Document {
    /// "ai" for a knowledge node, "post" for a blog post.
    kind: &'static str,
    /// What the result belongs to, shown under its title: the article's
    /// title for a node, "Writing" for a post.
    context: String,
    id: String,
    /// Position for breaking ties: article order, then posts as listed.
    order: usize,
    url: String,
    title: String,
    summary: String,
    concepts: Vec<String>,
    search_terms: Vec<String>,
    /// Distinct lowercase words of the body, for keyword matching.
    words: Vec<String>,
    passages: Vec<Passage>,
}

#[derive(Serialize)]
struct Passage {
    snippet: String,
    vector: Vec<f64>,
    /// The section of the page the passage is in, so a result can link
    /// there. Absent for passages above any heading, and for nodes, whose
    /// url already points at the node itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading: Option<String>,
}

/// The text embedded for a node's metadata: everything an author wrote
/// about what the node is, in one passage. Its snippet is the summary.
fn head_passage(node: &Node) -> String {
    format!(
        "{}. {} {}. {}.",
        node.title,
        node.summary,
        node.concepts.join(", "),
        node.search_terms.join(", ")
    )
}

/// Lowercase runs of letters and digits. Must split exactly like `words` in
/// js/retrieval.mjs (`[p{Alphabetic}p{N}]+`), which is what Rust's
/// `char::is_alphanumeric` means.
pub fn lexical_words(text: &str) -> Vec<String> {
    let mut words: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();
    words.sort();
    words.dedup();
    words
}

/// Cuts on a word boundary near `SNIPPET_CHARS`, never mid-character.
pub fn snippet(text: &str) -> String {
    if text.chars().count() <= SNIPPET_CHARS {
        return text.to_string();
    }
    let cut: String = text.chars().take(SNIPPET_CHARS).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{}…", cut.trim_end_matches([',', ';', ':', '.']))
}

/// Embeds a passage. `None` when none of its words are in the model's
/// vocabulary (a line of emoji, say): such a passage could never be found
/// by meaning, so it is left out rather than given a meaningless vector.
fn embedded(
    model: &StaticModel,
    text: &str,
    snippet: String,
    section: Option<&djot::Section>,
) -> Option<Passage> {
    let vector = model.encode(text)?;
    Some(Passage {
        snippet,
        vector: vector
            .into_iter()
            .map(|v| (f64::from(v) * VECTOR_DECIMALS).round() / VECTOR_DECIMALS)
            .collect(),
        anchor: section.map(|s| s.id.clone()),
        heading: section.map(|s| s.heading.clone()),
    })
}

fn body_passages(model: &StaticModel, body: &[djot::Passage]) -> Vec<Passage> {
    body.iter()
        .filter_map(|p| embedded(model, &p.text, snippet(&p.text), p.section.as_ref()))
        .collect()
}

fn body_words(body: &[djot::Passage]) -> Vec<String> {
    let text: Vec<&str> = body.iter().map(|p| p.text.as_str()).collect();
    lexical_words(&text.join(" "))
}

/// Serialises the index as JSON: every node of `article`, in reading order,
/// then every post, in listing order.
pub fn build(
    article: &Article,
    posts: &[Post],
    model: &StaticModel,
    info: &ModelInfo,
) -> Result<String> {
    let mut documents = Vec::new();

    for node in &article.nodes {
        let body = djot::passages(&node.source);
        let mut passages: Vec<Passage> =
            embedded(model, &head_passage(node), node.summary.clone(), None)
                .into_iter()
                .collect();
        passages.extend(body_passages(model, &body));

        documents.push(Document {
            kind: "ai",
            context: article.title.clone(),
            id: node.id.clone(),
            order: node.order,
            url: format!("{}#{}", article.url(), node.id),
            title: node.title.clone(),
            summary: node.summary.clone(),
            concepts: node.concepts.clone(),
            search_terms: node.search_terms.clone(),
            words: body_words(&body),
            passages,
        });
    }

    for (i, post) in posts.iter().enumerate() {
        let body = djot::passages(&post.source);
        let head = format!("{}. {}", post.title, post.description);
        let mut passages: Vec<Passage> = embedded(model, &head, post.description.clone(), None)
            .into_iter()
            .collect();
        passages.extend(body_passages(model, &body));

        documents.push(Document {
            kind: "post",
            context: "Writing".to_string(),
            id: post.path.clone(),
            order: article.nodes.len() + i + 1,
            url: post.url(),
            title: post.title.clone(),
            summary: post.description.clone(),
            concepts: Vec::new(),
            search_terms: Vec::new(),
            words: body_words(&body),
            passages,
        });
    }

    Ok(serde_json::to_string(&Index {
        version: 2,
        model: info,
        nodes: documents,
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{content, knowledge};
    use std::path::Path;

    #[test]
    fn splits_words_like_the_browser_does() {
        assert_eq!(
            lexical_words("Top-p, KV-cache and the KV cache; naïve 128k"),
            ["128k", "and", "cache", "kv", "naïve", "p", "the", "top"]
        );
    }

    #[test]
    fn snippets_are_cut_on_a_word_boundary() {
        let long = "word ".repeat(60);
        let s = snippet(long.trim());
        assert!(s.ends_with("word…"), "{s}");
        assert!(s.chars().count() <= SNIPPET_CHARS + 1);
        assert_eq!(snippet("short"), "short");
    }

    fn built_index() -> (serde_json::Value, Article, Vec<Post>, usize) {
        let model = StaticModel::load(Path::new("models/potion-base-4M")).unwrap();
        let article = knowledge::load(Path::new("content/ai"), |b| Ok(b.to_string())).unwrap();
        let posts = content::load_posts(Path::new("content/blog"), |b| Ok(b.to_string())).unwrap();
        let info = ModelInfo {
            name: "potion-base-4M".into(),
            weights_url: "/search/model.x.bin".into(),
            vocab_url: "/search/vocab.x.txt".into(),
            bytes: 1,
            dim: model.dim(),
        };
        let json = serde_json::from_str(&build(&article, &posts, &model, &info).unwrap()).unwrap();
        (json, article, posts, model.dim())
    }

    #[test]
    fn indexes_every_node_and_post_with_unit_vectors_of_the_model_dimension() {
        let (json, article, posts, dim) = built_index();
        let docs = json["nodes"].as_array().unwrap();
        assert_eq!(docs.len(), article.nodes.len() + posts.len());

        for (i, doc) in docs.iter().enumerate() {
            let id = doc["id"].as_str().unwrap();
            assert_eq!(
                doc["order"],
                i + 1,
                "{id}: orders are unique and sequential"
            );
            match doc["kind"].as_str().unwrap() {
                "ai" => assert_eq!(doc["url"], format!("/blog/ai/#{id}")),
                "post" => assert_eq!(doc["url"], format!("/{id}/")),
                other => panic!("unknown kind {other}"),
            }
            let passages = doc["passages"].as_array().unwrap();
            assert!(
                passages.len() >= 2,
                "{id}: metadata plus at least one body passage"
            );
            for p in passages {
                let v: Vec<f64> = serde_json::from_value(p["vector"].clone()).unwrap();
                assert_eq!(v.len(), dim);
                let length = v.iter().map(|x| x * x).sum::<f64>().sqrt();
                assert!((length - 1.0).abs() < 1e-2, "{id}: |v| = {length}");
            }
            assert!(!doc["words"].as_array().unwrap().is_empty());
        }
        assert_eq!(json["model"]["dim"], dim);
    }

    /// Post passages under a heading carry its anchor, so the palette can
    /// link to the section instead of the top of a long post.
    #[test]
    fn post_passages_carry_their_section_anchor() {
        let (json, ..) = built_index();
        let anchored = json["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["kind"] == "post")
            .flat_map(|d| d["passages"].as_array().unwrap().iter())
            .filter(|p| p["anchor"].is_string() && p["heading"].is_string())
            .count();
        assert!(
            anchored > 10,
            "only {anchored} post passages have a section anchor"
        );
    }
}
