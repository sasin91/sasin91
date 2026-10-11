--TEST--
the content in the repository loads: every post, the CV and the AI article with all its checks
--FILE--
<?php
require __DIR__ . '/setup.inc';
$posts = (new Posts(APPPATH . 'content/blog'))->all();
foreach ($posts as $post) {
    $markdown->render($post['source']);
}
$cv = Cv_data::load(APPPATH . 'content/cv.toml');
$article = (new Knowledge_article(APPPATH . 'content/ai', $markdown))->load();
var_dump(count($posts) > 0, $cv['site']['name'] !== '', count($article['nodes']) > 0);
var_dump($article['nodes'][0]['order'], $article['parts'][0]['id']);
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
int(1)
string(6) "part-1"
