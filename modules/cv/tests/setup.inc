<?php
define('APPPATH', str_replace('\\', '/', dirname(__DIR__, 3)) . '/');
require_once APPPATH . 'packages/autoload.php';
require_once APPPATH . 'modules/content/Cv_data.php';
require_once __DIR__ . '/../Cv_pdf.php';

$cv = Cv_data::load(APPPATH . 'content/cv.toml');
