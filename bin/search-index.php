<?php
/**
 * Prints the search index, built from content/ exactly as /search/index.json
 * serves it, for the JavaScript tests to ask real questions of:
 *
 *   php bin/search-index.php > build/search-index.json
 *   node --test modules/search/js/
 */
define('APPPATH', str_replace('\\', '/', dirname(__DIR__)) . '/');

require APPPATH . 'packages/autoload.php';
require APPPATH . 'modules/content/Markdown_html.php';
require APPPATH . 'modules/content/Posts.php';
require APPPATH . 'modules/content/Knowledge_article.php';
require APPPATH . 'modules/search/Search_index.php';

$posts = (new Posts(APPPATH . 'content/blog'))->all();
$article = (new Knowledge_article(APPPATH . 'content/ai', new Markdown_html()))->load(false);
$model_dir = APPPATH . 'models/potion-base-4M';

$index = new Search_index(Static_model::from_dir($model_dir), new Markdown_html());
echo $index->build($article, $posts, Search_index::model_info($model_dir, 'potion-base-4M'));
