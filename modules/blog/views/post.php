  <article class="prose">
<?php if ($post['hero_html'] !== null) { ?>
    <?= $post['hero_html'] ?>
<?php } ?>
<?php if ($post['hero_is_img']) { ?>
    <img src="<?= out($post['hero']) ?>" alt="<?= out($post['alt_text']) ?>" />
<?php } ?>

    <h1><?= out($post['title']) ?></h1>
    <div class="meta">
      <time datetime="<?= $post['date_iso'] ?>"><?= $post['date_long'] ?></time>
    </div>

    <?php /* Ask, right under the title and date: a question answered from
       this post's own sections. The controls start disabled; /search/ask.js
       (imported by the loader at the end of the page, only on pages with this
       form) enables them, and fetches the index and model only once the
       reader focuses the field. */ ?>
    <form class="ask ask-inline" role="search" data-post-ask data-index="<?= out($index_url) ?>"
          data-scope="post:<?= out($post['path']) ?>" data-model-bytes="<?= $model_bytes ?>">
      <label for="ask-query" class="visually-hidden">Ask this article</label>
      <div class="ask-row">
        <input id="ask-query" name="q" type="search" autocomplete="off" spellcheck="false"
               placeholder="Ask this article a question" disabled>
        <button type="submit" disabled>Ask</button>
      </div>
      <p id="ask-status" class="ask-status" role="status" aria-live="polite"></p>
<?php if ($has_examples) { ?>
      <div class="ask-examples" hidden>
        <p>Try:</p>
        <ul>
<?php foreach ($post['ask'] as $example) { ?>
          <li><button type="button"><?= out($example) ?></button></li>
<?php } ?>
        </ul>
      </div>
<?php } ?>
      <section class="ask-results" aria-labelledby="ask-results-title" hidden>
        <h2 id="ask-results-title" class="visually-hidden">Relevant sections</h2>
        <ol id="ask-results"></ol>
        <p class="ask-credit">
          Retrieval over this article, run on your device: it finds sections, it
          does not write answers. <a href="/blog/ai/ask/#ask-how-title">How it works</a>.
        </p>
      </section>
    </form>

    <?= $post['body'] ?>
  </article>
