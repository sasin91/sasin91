<?php
require_once __DIR__ . '/Cv_pdf.php';

/**
 * /cv/ and the same CV as a PDF, both from content/cv.toml.
 */
class Cv extends Trongate {

    const PDF_NAME = 'cv_jonas_hansen_software_developer.pdf';

    public function index(): void {
        $cv = $this->content->cv();

        $data = [
            'view_module' => 'cv',
            'view_file' => 'cv',
            'cv' => $cv,
            'pdf_url' => '/' . self::PDF_NAME,
            'nav' => 'cv',
            'meta' => [
                'title' => 'CV — ' . $cv['site']['name'],
                'description' => $cv['site']['name'] . ', ' . $cv['site']['title'] . '. '
                    . $cv['contact']['town'] . ', ' . $cv['contact']['postcode'] . '.',
                'path' => '/cv/',
                'og_type' => 'website',
            ],
        ];
        $this->layout->page($data);
    }

    /**
     * The PDF, laid out on request. It is a few kilobytes of text and takes
     * milliseconds, so there is no file to keep in step with the page.
     */
    public function pdf(): void {
        $pdf = Cv_pdf::render($this->content->cv());
        header('Content-Type: application/pdf');
        header('Content-Disposition: inline; filename="' . self::PDF_NAME . '"');
        header('Content-Length: ' . strlen($pdf));
        echo $pdf;
    }

}
