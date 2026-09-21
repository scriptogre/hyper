export function notebookGrammar(grammar) {
  return {
    ...grammar,
    name: 'hyper',
    patterns: [
      {match: '^%%hyper\\b.*$', name: 'keyword.control.hyper'},
      ...grammar.patterns,
    ],
  };
}

class HyperState {
  constructor(stack = null) { this.stack = stack; }
  clone() { return this; }
  equals(other) {
    return other instanceof HyperState &&
      (this.stack === other.stack || Boolean(this.stack?.equals(other.stack)));
  }
}

export function tokensProvider(highlighter) {
  const grammar = highlighter.getLanguage('hyper');

  return {
    getInitialState: () => new HyperState(),
    tokenize(line, state) {
      const result = grammar.tokenizeLine(line, state.stack);
      return {
        endState: new HyperState(result.ruleStack),
        tokens: result.tokens.map(token => ({
          startIndex: token.startIndex,
          scopes: token.scopes.at(-1),
        })),
      };
    },
  };
}

export function highlightHistory(root, highlighter) {
  const document = root.ownerDocument;
  const rendered = new WeakMap();
  const observer = new document.defaultView.MutationObserver(refresh);

  function refresh() {
    observer.disconnect();

    try {
      for (const code of root.querySelectorAll('.card-in pre code.language-hyper')) {
        const source = code.textContent;
        if (rendered.get(code) === source) continue;

        const template = document.createElement('template');
        template.innerHTML = highlighter.codeToHtml(source, {
          lang: 'hyper',
          themes: {light: 'github-light', dark: 'github-dark'},
          defaultColor: false,
        });
        code.replaceChildren(...template.content.querySelector('code').childNodes);
        code.classList.add('hyper-history');
        rendered.set(code, source);
      }
    } finally {
      observer.observe(root, {childList: true, characterData: true, subtree: true});
    }
  }

  refresh();
  return observer;
}

export async function installHyperHighlighting(grammar) {
  const {createHighlighter} = await import('https://esm.sh/shiki@3.22.0');
  const highlighter = await createHighlighter({
    langs: ['python', notebookGrammar(grammar)],
    themes: await Promise.all(['/vendor/gh-light-1.json', '/vendor/gh-dark-1.json'].map(async url => {
      const response = await fetch(url);
      if (!response.ok) throw new Error(`Cannot load SolveIT theme: ${url}`);
      return response.json();
    })),
  });

  window.hyperHighlight?.dispose();
  window.hyperModelHooks?.forEach(hook => hook.dispose());
  document.getElementById('hyper-token-colors')?.remove();

  if (!monaco.languages.getLanguages().some(language => language.id === 'hyper')) {
    monaco.languages.register({id: 'hyper'});
  }
  const provider = monaco.languages.setTokensProvider('hyper', tokensProvider(highlighter));
  const style = document.createElement('style');
  style.textContent = `
    code.hyper-history span {color:var(--shiki-light);font-style:var(--shiki-light-font-style);font-weight:var(--shiki-light-font-weight)}
    .dark code.hyper-history span {color:var(--shiki-dark);font-style:var(--shiki-dark-font-style);font-weight:var(--shiki-dark-font-weight)}
  `;
  document.head.append(style);
  const history = highlightHistory(document.getElementById('dialog-container'), highlighter);

  window.hyperHighlight = {
    dispose() {
      history.disconnect();
      style.remove();
      provider.dispose();
      highlighter.dispose();
    },
  };

  // SolveIT caches magic-to-language mappings when its editor first opens.
  if (typeof _langMap !== 'undefined') _langMap = undefined;
  for (const model of monaco.editor.getModels()) {
    if (model.getValue().startsWith('%%hyper')) monaco.editor.setModelLanguage(model, 'hyper');
  }
}
