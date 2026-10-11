  <h1>Ask this article</h1>
  <p class="lede">
    Ask a question in your own words, and get back the sections of
    <a href="<?= out($read_url) ?>"><?= out($title) ?></a> that answer it best.
    This is retrieval, not a chatbot: it finds sections, it does not write
    answers.
  </p>

  <nav class="view-nav" aria-label="Views of this article">
    <a href="<?= out($read_url) ?>">Read</a>
    <a href="<?= out($timeline_url) ?>">Timeline</a>
    <a href="<?= out($ask_url) ?>" aria-current="page">Ask</a>
  </nav>

  <?php /* The controls start disabled and the script enables them. Without
     JavaScript a submit would put the question in a URL and send it to the
     server, which is exactly what this page promises not to do. */ ?>
  <form class="ask" role="search" data-ask data-index="<?= out($index_url) ?>" data-scope="<?= out($scope) ?>" data-model-bytes="<?= $model['bytes'] ?>">
    <label for="ask-query">Your question</label>
    <div class="ask-row">
      <input id="ask-query" name="q" type="search" autocomplete="off" spellcheck="false"
             placeholder="<?= out($placeholder) ?>" disabled>
      <button type="submit" disabled>Ask</button>
    </div>
    <label class="ask-option">
      <input id="ask-semantic" type="checkbox" checked disabled>
      Rank by meaning as well as by matching words
    </label>
    <p id="ask-status" class="ask-status" role="status" aria-live="polite"></p>
    <noscript>
      <p class="ask-status">
        Ask runs entirely in your browser, so it needs JavaScript.
        Without it, the
        <a href="<?= out($timeline_url) ?>">timeline</a> is the quickest way to
        find a section.
      </p>
    </noscript>
  </form>

  <div class="ask-examples" hidden>
    <p>Or try:</p>
    <ul>
<?php foreach ($examples as $example) { ?>
      <li><button type="button"><?= out($example) ?></button></li>
<?php } ?>
    </ul>
  </div>

  <section class="ask-results" aria-labelledby="ask-results-title" hidden>
    <h2 id="ask-results-title">Relevant sections</h2>
    <ol id="ask-results"></ol>
  </section>

  <section class="ask-how" aria-labelledby="ask-how-title">
    <h2 id="ask-how-title">How this works</h2>
    <p>
      This page is a small retrieval system of the kind described in
      <a href="<?= out($ai_url) ?>#rag">Retrieval-augmented generation</a>. Every
      step happens in your browser; your question is never sent anywhere.
    </p>
    <ol class="pipeline">
      <li>
        <strong>Tokenize.</strong> Your question is split into the WordPiece
        tokens of the model's vocabulary
        (<a href="<?= out($ai_url) ?>#tokenization">tokenization</a>).
        <output id="ask-step-tokens" class="pipeline-output"></output>
      </li>
      <li>
        <strong>Embed.</strong> Each token selects one row of a static embedding
        table, and the rows are averaged into a single vector of
        <?= $model['dim'] ?> numbers
        (<a href="<?= out($ai_url) ?>#embeddings">embeddings</a>).
        <output id="ask-step-vector" class="pipeline-output"></output>
      </li>
      <li>
        <strong>Compare.</strong> That vector is compared with vectors computed
        for every passage of the article ahead of time, by cosine
        similarity.
        <output id="ask-step-compare" class="pipeline-output"></output>
      </li>
      <li>
        <strong>Match words.</strong> Exact words and phrases from your question
        are matched against each section's title, concepts and text, so a
        precise term like <q>KV cache</q> is never outranked by something only
        vaguely similar.
        <output id="ask-step-words" class="pipeline-output"></output>
      </li>
      <li>
        <strong>Rank.</strong> The two signals are combined, and the best
        sections are listed above, each linking to its place in the article.
      </li>
    </ol>
    <p class="ask-credit">
      Model: <?= out($model['name']) ?> by Minish Lab (MIT licence), a static embedding
      table of <?= $model['dim'] ?> dimensions, quantised to 8 bits. It is served
      from this site and downloaded once, when you first use the form.
    </p>
  </section>

  <script type="module">
<?= $script ?>
  </script>
