//! Knowledge nodes: small units of explanation, each authored once as a Djot
//! file under `content/ai/nodes/`, and the article manifest that arranges
//! them (`content/ai/article.toml`).
//!
//! A node is not a page and does not know where it is shown. The article
//! decides the reading order by listing node ids in its parts; the timeline
//! and the Ask index are generated from the same nodes; learning paths are
//! further ordered lists of the same ids. So a node can appear in any number
//! of views without its content being copied.
//!
//! Everything that can be checked is checked here, at build time, and a
//! problem stops the build with every error listed at once: a typo in a
//! `related` id should fail loudly, not quietly drop a link.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Range;
use std::path::Path;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::content::{split_toml_frontmatter, toml_date};
use crate::djot;

/// Ids other markup on the article page already uses. A node with one of
/// these ids would make its anchor ambiguous. Parts are `part-1`, `part-2`,
/// ... so that prefix is reserved too.
const RESERVED_IDS: [&str; 3] = ["content", "theme-toggle", "lightbox"];
const PART_PREFIX: &str = "part-";

/// The timeline shows the summary as a single line under the title; past
/// this it stops being a one-glance summary and starts being a paragraph.
pub const MAX_SUMMARY_CHARS: usize = 200;

/// A node's frontmatter, exactly as authored. Unknown fields are an error,
/// so a misspelt `prerequisite = [...]` cannot silently mean "none".
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeFront {
    /// Stable identifier, also the URL anchor (`/ai/#kv-cache`) and the
    /// file name (`kv-cache.dj`). Changing it breaks inbound links.
    pub id: String,
    pub title: String,
    /// One or two sentences. Shown in the timeline and in Ask results.
    pub summary: String,
    /// The ideas this node explains, in the words a reader would use. They
    /// are matched as exact phrases by Ask's lexical ranking.
    pub concepts: Vec<String>,
    /// Nodes a reader should understand first. Must come earlier in the
    /// article.
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Nodes worth reading alongside, in either direction.
    #[serde(default)]
    pub related: Vec<String>,
    /// Extra phrasings, acronyms and questions for retrieval only.
    #[serde(default)]
    pub search_terms: Vec<String>,
    /// For future lessons: what a reader should be able to do afterwards.
    #[serde(default)]
    pub learning_objectives: Vec<String>,
    /// Wrong beliefs this node corrects, written as the belief itself. Shown
    /// under the node, and usable later as quiz distractors.
    #[serde(default)]
    pub misconceptions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NodeLink {
    pub id: String,
    pub title: String,
}

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub concepts: Vec<String>,
    pub prerequisites: Vec<String>,
    pub related: Vec<String>,
    pub search_terms: Vec<String>,
    #[expect(
        dead_code,
        reason = "authored and parsed now; read by the future lessons view"
    )]
    pub learning_objectives: Vec<String>,
    pub misconceptions: Vec<String>,
    /// Djot source of the body, kept for building the search index.
    pub source: String,
    /// Rendered HTML of the body.
    pub body: String,
    /// 1-based position in the article. Derived from the manifest, never
    /// authored, so reordering the article is moving an id in one list.
    pub order: usize,
    /// `prerequisites`, resolved to titles for display.
    pub builds_on: Vec<NodeLink>,
    /// `related`, resolved to titles for display.
    pub see_also: Vec<NodeLink>,
}

impl Node {
    pub fn from_front(front: NodeFront, source: String, body: String) -> Self {
        Self {
            id: front.id,
            title: front.title,
            summary: front.summary,
            concepts: front.concepts,
            prerequisites: front.prerequisites,
            related: front.related,
            search_terms: front.search_terms,
            learning_objectives: front.learning_objectives,
            misconceptions: front.misconceptions,
            source,
            body,
            order: 0,
            builds_on: Vec::new(),
            see_also: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// URL path of the article, without slashes: `ai` serves `/ai/`.
    pub path: String,
    pub title: String,
    pub description: String,
    #[serde(deserialize_with = "toml_date")]
    pub published: NaiveDate,
    #[serde(deserialize_with = "toml_date")]
    pub updated: NaiveDate,
    #[serde(rename = "part")]
    pub parts: Vec<PartFront>,
    #[serde(rename = "learning_path", default)]
    pub learning_paths: Vec<LearningPath>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartFront {
    pub title: String,
    pub nodes: Vec<String>,
}

/// A named route through a subset of the nodes, for future guided lessons.
/// Validated now so that paths written ahead of the feature cannot rot.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearningPath {
    pub id: String,
    #[expect(dead_code, reason = "validated now; shown by the future lessons view")]
    pub title: String,
    pub nodes: Vec<String>,
}

#[derive(Debug)]
pub struct Part {
    /// `part-1`, `part-2`, ...: the anchor of the part's heading.
    pub id: String,
    pub title: String,
    /// The part's nodes, as a range into `Article::nodes`.
    pub nodes: Range<usize>,
}

#[derive(Debug)]
pub struct Article {
    pub path: String,
    pub title: String,
    pub description: String,
    pub published: NaiveDate,
    pub updated: NaiveDate,
    pub parts: Vec<Part>,
    /// Every node, in reading order.
    pub nodes: Vec<Node>,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "validated now; the extension point for guided lessons"
        )
    )]
    pub learning_paths: Vec<LearningPath>,
}

impl Article {
    pub fn url(&self) -> String {
        format!("/{}/", self.path)
    }

    pub fn nodes_in(&self, part: &Part) -> &[Node] {
        &self.nodes[part.nodes.clone()]
    }

    pub fn published_iso(&self) -> String {
        self.published.format("%Y-%m-%d").to_string()
    }

    pub fn updated_iso(&self) -> String {
        self.updated.format("%Y-%m-%d").to_string()
    }

    /// "October 2, 2026", matching `Post::date_long`.
    pub fn published_long(&self) -> String {
        self.published
            .format("%B %e, %Y")
            .to_string()
            .replace("  ", " ")
    }

    pub fn updated_long(&self) -> String {
        self.updated
            .format("%B %e, %Y")
            .to_string()
            .replace("  ", " ")
    }

    /// Whether the article has been revised since it was first published.
    pub fn was_updated(&self) -> bool {
        self.updated != self.published
    }

    /// Whether any node renders a highlighted code block (see
    /// `Post::has_syntax`).
    pub fn has_syntax(&self) -> bool {
        self.nodes
            .iter()
            .any(|n| n.body.contains("<pre class=\"hl-code\">"))
    }

    /// Whether any node renders an inlined diagram (see `Post::has_diagrams`).
    pub fn has_diagrams(&self) -> bool {
        self.nodes
            .iter()
            .any(|n| n.body.contains("<figure class=\"diagram\""))
    }
}

/// Loads `dir/article.toml` and every `dir/nodes/*.dj`, renders each body
/// with `render`, and validates the whole set.
pub fn load(dir: &Path, render: impl Fn(&str) -> Result<String>) -> Result<Article> {
    let manifest_path = dir.join("article.toml");
    let manifest: Manifest = toml::from_str(
        &fs::read_to_string(&manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parsing {}", manifest_path.display()))?;

    let mut files: Vec<_> = WalkDir::new(dir.join("nodes"))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("walking {}/nodes", dir.display()))?
        .into_iter()
        .filter(|e| e.path().extension().is_some_and(|x| x == "dj"))
        .map(|e| e.into_path())
        .collect();
    files.sort();

    let mut nodes = Vec::new();
    for file in files {
        let source =
            fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
        let (front, body): (NodeFront, &str) = split_toml_frontmatter(&source)
            .with_context(|| format!("parsing {}", file.display()))?;

        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if stem != front.id {
            bail!(
                "{}: id {:?} must match the file name, so a node is found where its id says",
                file.display(),
                front.id
            );
        }

        let html = render(body).with_context(|| format!("rendering {}", file.display()))?;
        nodes.push(Node::from_front(front, body.to_string(), html));
    }

    assemble(manifest, nodes)
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// Orders the nodes as the manifest says, resolves their links, and checks
/// every rule in this module. Returns all problems at once.
pub fn assemble(manifest: Manifest, nodes: Vec<Node>) -> Result<Article> {
    let mut errors: Vec<String> = Vec::new();

    // ---- the nodes themselves ----
    let mut by_id: HashMap<String, Node> = HashMap::new();
    for node in nodes {
        let id = node.id.clone();
        if !is_valid_id(&id) {
            errors.push(format!(
                "node {id:?}: ids are lowercase letters, digits and single hyphens"
            ));
        }
        if RESERVED_IDS.contains(&id.as_str()) || id.starts_with(PART_PREFIX) {
            errors.push(format!(
                "node {id:?}: that id is reserved for other markup on the page"
            ));
        }
        if node.title.trim().is_empty() {
            errors.push(format!("node {id:?}: title is empty"));
        }
        if node.summary.trim().is_empty() {
            errors.push(format!("node {id:?}: summary is empty"));
        }
        let summary_chars = node.summary.chars().count();
        if summary_chars > MAX_SUMMARY_CHARS {
            errors.push(format!(
                "node {id:?}: summary is {summary_chars} characters; keep it under {MAX_SUMMARY_CHARS} \
                 so it reads as one line of the timeline"
            ));
        }
        if node.concepts.is_empty() {
            errors.push(format!("node {id:?}: list at least one concept"));
        }
        if djot::has_heading(&node.source) {
            errors.push(format!(
                "node {id:?}: the body contains a heading; the node title is the heading, \
                 so split the node instead"
            ));
        }
        if let Some(previous) = by_id.insert(id.clone(), node) {
            errors.push(format!(
                "node {:?}: id is used by more than one file",
                previous.id
            ));
        }
    }

    // ---- the article order ----
    let mut order: Vec<String> = Vec::new();
    let mut parts: Vec<Part> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (index, part) in manifest.parts.iter().enumerate() {
        if part.nodes.is_empty() {
            errors.push(format!("part {:?} lists no nodes", part.title));
        }
        let start = order.len();
        for id in &part.nodes {
            if !by_id.contains_key(id) {
                errors.push(format!(
                    "part {:?} lists {id:?}, which is not a node",
                    part.title
                ));
            } else if !seen.insert(id.clone()) {
                errors.push(format!(
                    "node {id:?} is listed in the article more than once"
                ));
            } else {
                order.push(id.clone());
            }
        }
        parts.push(Part {
            id: format!("{PART_PREFIX}{}", index + 1),
            title: part.title.clone(),
            nodes: start..order.len(),
        });
    }
    let mut unlisted: Vec<&String> = by_id.keys().filter(|id| !seen.contains(*id)).collect();
    unlisted.sort();
    for id in unlisted {
        errors.push(format!(
            "node {id:?} is not listed in any part of article.toml"
        ));
    }

    let position: HashMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    // ---- links between nodes ----
    for id in &order {
        let node = &by_id[id];
        for (field, ids) in [
            ("prerequisites", &node.prerequisites),
            ("related", &node.related),
        ] {
            let mut listed = HashSet::new();
            for target in ids {
                if target == id {
                    errors.push(format!("node {id:?}: {field} lists the node itself"));
                } else if !by_id.contains_key(target) {
                    errors.push(format!(
                        "node {id:?}: {field} lists {target:?}, which is not a node"
                    ));
                } else if !listed.insert(target) {
                    errors.push(format!("node {id:?}: {field} lists {target:?} twice"));
                } else if field == "prerequisites"
                    && matches!(
                        (position.get(target.as_str()), position.get(id.as_str())),
                        (Some(t), Some(n)) if t > n
                    )
                {
                    errors.push(format!(
                        "node {id:?}: prerequisite {target:?} comes later in the article; \
                         move one of them, or make it `related` instead"
                    ));
                }
            }
        }
        for target in &node.related {
            if node.prerequisites.contains(target) {
                errors.push(format!(
                    "node {id:?}: {target:?} is both a prerequisite and related; keep it in one list"
                ));
            }
        }

        // In-body links to an anchor on the article page must land somewhere.
        for anchor in page_anchors(&node.body, &manifest.path) {
            let exists = by_id.contains_key(anchor) || parts.iter().any(|p| p.id == anchor);
            if !exists {
                errors.push(format!(
                    "node {id:?}: links to #{anchor}, which is not a node or part"
                ));
            }
        }
    }

    // ---- learning paths ----
    let mut path_ids = HashSet::new();
    for path in &manifest.learning_paths {
        if !is_valid_id(&path.id) {
            errors.push(format!(
                "learning path {:?}: ids are lowercase letters, digits and hyphens",
                path.id
            ));
        }
        if !path_ids.insert(&path.id) {
            errors.push(format!(
                "learning path {:?} is defined more than once",
                path.id
            ));
        }
        if path.nodes.is_empty() {
            errors.push(format!("learning path {:?} lists no nodes", path.id));
        }
        let mut listed = HashSet::new();
        for id in &path.nodes {
            if !by_id.contains_key(id) {
                errors.push(format!(
                    "learning path {:?} lists {id:?}, which is not a node",
                    path.id
                ));
            } else if !listed.insert(id) {
                errors.push(format!("learning path {:?} lists {id:?} twice", path.id));
            }
        }
    }

    if !errors.is_empty() {
        bail!("content/ai is invalid:\n  - {}", errors.join("\n  - "));
    }

    // ---- everything checks out: build the article ----
    let link = |id: &String| NodeLink {
        id: id.clone(),
        title: by_id[id].title.clone(),
    };
    let resolved: Vec<(Vec<NodeLink>, Vec<NodeLink>)> = order
        .iter()
        .map(|id| {
            let node = &by_id[id];
            (
                node.prerequisites.iter().map(link).collect(),
                node.related.iter().map(link).collect(),
            )
        })
        .collect();

    let nodes = order
        .iter()
        .zip(resolved)
        .enumerate()
        .map(|(i, (id, (builds_on, see_also)))| {
            let mut node = by_id.remove(id).expect("validated above");
            node.order = i + 1;
            node.builds_on = builds_on;
            node.see_also = see_also;
            node
        })
        .collect();

    Ok(Article {
        path: manifest.path,
        title: manifest.title,
        description: manifest.description,
        published: manifest.published,
        updated: manifest.updated,
        parts,
        nodes,
        learning_paths: manifest.learning_paths,
    })
}

/// Anchors that `html` links to on the article page itself: `href="#x"` and
/// `href="/<path>/#x"`.
fn page_anchors<'a>(html: &'a str, path: &str) -> Vec<&'a str> {
    let absolute = format!("href=\"/{path}/#");
    let mut anchors = Vec::new();
    for prefix in ["href=\"#", absolute.as_str()] {
        let mut rest = html;
        while let Some(at) = rest.find(prefix) {
            rest = &rest[at + prefix.len()..];
            let end = rest.find('"').unwrap_or(rest.len());
            anchors.push(&rest[..end]);
        }
    }
    anchors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str) -> Node {
        let front = NodeFront {
            id: id.into(),
            title: format!("Title of {id}"),
            summary: "A summary.".into(),
            concepts: vec![id.replace('-', " ")],
            prerequisites: vec![],
            related: vec![],
            search_terms: vec![],
            learning_objectives: vec![],
            misconceptions: vec![],
        };
        Node::from_front(front, "Body.\n".into(), "<p>Body.</p>".into())
    }

    fn manifest(parts: &[&[&str]]) -> Manifest {
        Manifest {
            path: "ai".into(),
            title: "t".into(),
            description: "d".into(),
            published: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            parts: parts
                .iter()
                .enumerate()
                .map(|(i, ids)| PartFront {
                    title: format!("Part {i}"),
                    nodes: ids.iter().map(|s| s.to_string()).collect(),
                })
                .collect(),
            learning_paths: vec![],
        }
    }

    fn error_of(result: Result<Article>) -> String {
        format!(
            "{:#}",
            result.expect_err("expected the article to be rejected")
        )
    }

    #[test]
    fn orders_nodes_by_the_manifest_not_by_file() {
        let article = assemble(
            manifest(&[&["b", "a"], &["c"]]),
            vec![node("a"), node("b"), node("c")],
        )
        .unwrap();
        let ids: Vec<_> = article
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n.order))
            .collect();
        assert_eq!(ids, [("b", 1), ("a", 2), ("c", 3)]);
        assert_eq!(article.parts[0].id, "part-1");
        let second: Vec<_> = article
            .nodes_in(&article.parts[1])
            .iter()
            .map(|n| &n.id)
            .collect();
        assert_eq!(second, ["c"]);
    }

    #[test]
    fn rejects_duplicate_ids() {
        let err = error_of(assemble(manifest(&[&["a"]]), vec![node("a"), node("a")]));
        assert!(err.contains("more than one file"), "{err}");
    }

    #[test]
    fn rejects_a_broken_related_or_prerequisite_id() {
        let mut a = node("a");
        a.related = vec!["nope".into()];
        let mut b = node("b");
        b.prerequisites = vec!["missing".into()];
        let err = error_of(assemble(manifest(&[&["a", "b"]]), vec![a, b]));
        assert!(err.contains("related lists \"nope\""), "{err}");
        assert!(err.contains("prerequisites lists \"missing\""), "{err}");
    }

    #[test]
    fn rejects_a_prerequisite_that_comes_later_in_the_article() {
        let mut a = node("a");
        a.prerequisites = vec!["b".into()];
        let err = error_of(assemble(manifest(&[&["a", "b"]]), vec![a, node("b")]));
        assert!(err.contains("comes later"), "{err}");
    }

    #[test]
    fn accepts_related_links_in_either_direction() {
        let mut a = node("a");
        a.related = vec!["b".into()];
        let mut b = node("b");
        b.prerequisites = vec!["a".into()];
        let article = assemble(manifest(&[&["a", "b"]]), vec![a, b]).unwrap();
        assert_eq!(article.nodes[0].see_also[0].title, "Title of b");
        assert_eq!(article.nodes[1].builds_on[0].id, "a");
    }

    #[test]
    fn rejects_self_links_and_duplicates_within_a_list() {
        let mut a = node("a");
        a.related = vec!["a".into(), "b".into(), "b".into()];
        let err = error_of(assemble(manifest(&[&["a", "b"]]), vec![a, node("b")]));
        assert!(err.contains("the node itself"), "{err}");
        assert!(err.contains("twice"), "{err}");
    }

    #[test]
    fn every_node_must_be_listed_exactly_once() {
        let err = error_of(assemble(
            manifest(&[&["a"], &["a", "ghost"]]),
            vec![node("a"), node("b")],
        ));
        assert!(
            err.contains("\"a\" is listed in the article more than once"),
            "{err}"
        );
        assert!(err.contains("\"ghost\", which is not a node"), "{err}");
        assert!(err.contains("\"b\" is not listed"), "{err}");
    }

    #[test]
    fn rejects_ids_that_are_not_safe_anchors_or_are_reserved() {
        let err = error_of(assemble(
            manifest(&[&["KV Cache", "content", "part-2", "a--b"]]),
            vec![
                node("KV Cache"),
                node("content"),
                node("part-2"),
                node("a--b"),
            ],
        ));
        assert!(err.contains("\"KV Cache\": ids are lowercase"), "{err}");
        assert!(err.contains("\"a--b\": ids are lowercase"), "{err}");
        assert!(err.contains("\"content\": that id is reserved"), "{err}");
        assert!(err.contains("\"part-2\": that id is reserved"), "{err}");
    }

    #[test]
    fn rejects_headings_inside_a_node_body() {
        let mut a = node("a");
        a.source = "## A sub-heading\n\nText.\n".into();
        let err = error_of(assemble(manifest(&[&["a"]]), vec![a]));
        assert!(err.contains("contains a heading"), "{err}");
    }

    #[test]
    fn rejects_an_overlong_summary() {
        let mut a = node("a");
        a.summary = "word ".repeat(60);
        let err = error_of(assemble(manifest(&[&["a"]]), vec![a]));
        assert!(err.contains("summary is 300 characters"), "{err}");
    }

    #[test]
    fn rejects_in_body_links_to_missing_anchors() {
        let mut a = node("a");
        a.body = r##"<p><a href="#b">ok</a> <a href="/ai/#part-1">ok</a> <a href="#gone">x</a> <a href="/ai/#also-gone">y</a> <a href="/blog/#x">elsewhere</a></p>"##.into();
        let err = error_of(assemble(manifest(&[&["a", "b"]]), vec![a, node("b")]));
        assert!(err.contains("#gone"), "{err}");
        assert!(err.contains("#also-gone"), "{err}");
        assert!(!err.contains("#b,"), "{err}");
        assert!(!err.contains("#part-1"), "{err}");
        assert!(!err.contains("#x"), "{err}");
    }

    #[test]
    fn validates_learning_paths() {
        let mut m = manifest(&[&["a", "b"]]);
        m.learning_paths = vec![
            LearningPath {
                id: "intro".into(),
                title: "Intro".into(),
                nodes: vec!["b".into(), "a".into()],
            },
            LearningPath {
                id: "intro".into(),
                title: "Again".into(),
                nodes: vec!["a".into(), "zzz".into(), "a".into()],
            },
        ];
        let err = error_of(assemble(m, vec![node("a"), node("b")]));
        assert!(err.contains("\"intro\" is defined more than once"), "{err}");
        assert!(err.contains("lists \"zzz\", which is not a node"), "{err}");
        assert!(err.contains("lists \"a\" twice"), "{err}");
    }

    #[test]
    fn a_learning_path_may_order_nodes_differently_from_the_article() {
        let mut m = manifest(&[&["a", "b"]]);
        m.learning_paths = vec![LearningPath {
            id: "reverse".into(),
            title: "Reverse".into(),
            nodes: vec!["b".into(), "a".into()],
        }];
        assert!(assemble(m, vec![node("a"), node("b")]).is_ok());
    }

    #[test]
    fn unknown_frontmatter_fields_are_rejected() {
        let source = "+++\nid = \"a\"\ntitle = \"A\"\nsummary = \"s\"\nconcepts = [\"a\"]\nprerequisite = [\"b\"]\n+++\nBody\n";
        let result: Result<(NodeFront, &str)> = split_toml_frontmatter(source);
        let err = format!("{:#}", result.unwrap_err());
        assert!(err.contains("prerequisite"), "{err}");
    }

    /// The real content must load and validate: this is what turns a broken
    /// link in content/ai into a failing `cargo test` as well as a failing
    /// build.
    #[test]
    fn the_real_article_is_valid() {
        let article = load(Path::new("content/ai"), |body| Ok(body.to_string())).unwrap();
        assert!(article.nodes.len() >= 20, "expected the seeded nodes");
        for (i, node) in article.nodes.iter().enumerate() {
            assert_eq!(node.order, i + 1);
        }
        let paths: Vec<_> = article
            .learning_paths
            .iter()
            .map(|p| p.id.as_str())
            .collect();
        assert!(paths.contains(&"llm-generation"), "{paths:?}");
        for id in [
            "tokenization",
            "attention",
            "kv-cache",
            "rag",
            "mcp",
            "training",
        ] {
            assert!(
                article.nodes.iter().any(|n| n.id == id),
                "anchor #{id} must exist"
            );
        }
    }
}
