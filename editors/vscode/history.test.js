import {afterAll, beforeAll, expect, test} from 'bun:test';
import {Window} from 'happy-dom';
import {createHighlighter} from 'shiki';
import grammar from './Syntaxes/hyper.tmLanguage.json';
import {highlightHistory, notebookGrammar} from '../../examples/solveit-highlight.js';

let highlighter;

beforeAll(async () => {
  highlighter = await createHighlighter({
    langs: ['python', notebookGrammar(grammar)],
    themes: ['github-light', 'github-dark'],
  });
});

afterAll(() => highlighter.dispose());

test('history uses the shared grammar and preserves source text', () => {
  const {document} = new Window();
  document.body.innerHTML = '<div class="card-in"><pre><code class="language-hyper"></code></pre></div>';
  const code = document.querySelector('code');
  const source = '%%hyper Greeting\nfor name in names:\n    <p>{name}</p>\nend\n';
  code.textContent = source;

  const observer = highlightHistory(document.body, highlighter);

  expect(code.textContent).toBe(source);
  expect(code.querySelector('p')).toBeNull();
  const keyword = [...code.querySelectorAll('span')].find(span => span.textContent === 'in');
  expect(keyword.getAttribute('style')).toContain('--shiki-light:');
  expect(keyword.getAttribute('style')).toContain('--shiki-dark:');
  observer.disconnect();
});

test('history handles new and edited Hyper cells without touching Python cells', async () => {
  const window = new Window();
  const {document} = window;
  document.body.innerHTML = '<div class="card-in"><pre><code class="language-python">x = 1</code></pre></div>';
  const python = document.querySelector('code');
  const observer = highlightHistory(document.body, highlighter);

  const card = document.createElement('div');
  card.className = 'card-in';
  card.innerHTML = '<pre><code class="language-hyper">%%hyper Button\n&lt;button&gt;Save&lt;/button&gt;</code></pre>';
  document.body.append(card);
  await window.happyDOM.waitUntilComplete();

  const code = card.querySelector('code');
  expect(code.classList.contains('hyper-history')).toBe(true);
  code.textContent = '%%hyper Button\n<button>Updated</button>';
  await window.happyDOM.waitUntilComplete();

  expect(code.querySelectorAll('span').length).toBeGreaterThan(0);
  expect(code.textContent).toBe('%%hyper Button\n<button>Updated</button>');
  expect(python.innerHTML).toBe('x = 1');

  observer.disconnect();
  code.textContent = 'Stopped';
  await window.happyDOM.waitUntilComplete();
  expect(code.innerHTML).toBe('Stopped');
});
