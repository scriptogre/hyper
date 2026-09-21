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

export async function installHyperHighlighting(grammar) {
  const {createHighlighter} = await import('https://esm.sh/shiki@3.22.0');
  const highlighter = await createHighlighter({
    langs: ['python', notebookGrammar(grammar)],
    themes: [],
  });

  window.hyperHighlight?.dispose();
  window.hyperModelHooks?.forEach(hook => hook.dispose());
  document.getElementById('hyper-token-colors')?.remove();

  if (!monaco.languages.getLanguages().some(language => language.id === 'hyper')) {
    monaco.languages.register({id: 'hyper'});
  }
  const provider = monaco.languages.setTokensProvider('hyper', tokensProvider(highlighter));
  window.hyperHighlight = {
    dispose() { provider.dispose(); highlighter.dispose(); },
  };

  // SolveIT caches magic-to-language mappings when its editor first opens.
  if (typeof _langMap !== 'undefined') _langMap = undefined;
  for (const model of monaco.editor.getModels()) {
    if (model.getValue().startsWith('%%hyper')) monaco.editor.setModelLanguage(model, 'hyper');
  }
}
