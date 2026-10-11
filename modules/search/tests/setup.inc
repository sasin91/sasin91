<?php
define('APPPATH', str_replace('\\', '/', dirname(__DIR__, 3)) . '/');
require_once APPPATH . 'packages/autoload.php';
require_once APPPATH . 'modules/content/Markdown_html.php';
require_once __DIR__ . '/../Search_index.php';

$model = Static_model::from_dir(APPPATH . 'models/potion-base-4M');
