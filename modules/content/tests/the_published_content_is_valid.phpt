--TEST--
the content in the repository loads: every post renders, the CV reads, and the AI article passes all its checks
--FILE--
<?php
require __DIR__ . '/setup.inc';
$posts = (new Posts(APPPATH . 'modules/blog/views/posts'))->all();
foreach ($posts as $post) {
    Prose_view::html($post['file']);
}
$cv = Cv_data::load(APPPATH . 'modules/cv/cv.toml');
$article = (new Knowledge_article(APPPATH . 'modules/knowledge'))->load();
var_dump(count($posts) > 0, $cv['site']['name'] !== '', count($article['nodes']) > 0);
var_dump($article['nodes'][0]['order'], $article['parts'][0]['id']);
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
int(1)
string(6) "part-1"
