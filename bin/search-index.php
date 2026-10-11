<?php
/**
 * Prints the search index, built from the posts and nodes exactly as /search/index.json
 * serves it, for the JavaScript tests to ask real questions of:
 *
 *   php bin/search-index.php > build/search-index.json
 *   node --test modules/search/js/
 */
define('APPPATH', str_replace('\\', '/', dirname(__DIR__)) . '/');

require APPPATH . 'packages/autoload.php';
// Posts written with the prose helpers reach their module through Trongate.
define('BASE_URL', 'http://localhost/');
require APPPATH . 'engine/Trongate.php';
require APPPATH . 'engine/Modules.php';
require APPPATH . 'modules/content/Posts.php';
require APPPATH . 'modules/content/Knowledge_article.php';
require APPPATH . 'modules/search/Search_index.php';

$posts = (new Posts(APPPATH . 'modules/blog/views/posts'))->all();
$article = (new Knowledge_article(APPPATH . 'modules/knowledge'))->load(false);
$model_dir = APPPATH . 'models/potion-base-4M';

$index = new Search_index(Static_model::from_dir($model_dir));
echo $index->build($article, $posts, Search_index::model_info($model_dir, 'potion-base-4M'));
