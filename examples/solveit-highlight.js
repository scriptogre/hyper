(() => {
  const language = 'hyper';
  if (!monaco.languages.getLanguages().some(lang => lang.id === language)) {
    monaco.languages.register({id: language});
  }
  window.hyperHighlight?.dispose();
  window.hyperHighlight = monaco.languages.setMonarchTokensProvider(language, {
    keywords: ['for', 'in', 'if', 'else', 'elif', 'end', 'from', 'import', 'as',
      'def', 'return', 'yield', 'await', 'async', 'True', 'False', 'None', 'and', 'or', 'not'],
    tokenizer: {
      root: [
        [/^%%hyper.*$/, 'metatag'],
        [/^\s*---\s*$/, 'delimiter'],
        [/<\/?[\w-]+/, {token: 'tag', next: '@tag'}],
        [/\{/, {token: 'delimiter.bracket', next: '@expression'}],
        {include: '@python'},
      ],
      tag: [
        [/\s+/, 'white'],
        [/\/?\>/, {token: 'tag', switchTo: '@text'}],
        [/\{/, {token: 'delimiter.bracket', next: '@expression'}],
        [/"[^"\n]*"|'[^'\n]*'/, 'attribute.value'],
        [/[\w:-]+/, 'attribute.name'],
        [/=/, 'delimiter'],
      ],
      text: [
        [/^/, {token: '', next: '@popall'}],
        [/<\/?[\w-]+/, {token: 'tag', next: '@tag'}],
        [/\{/, {token: 'delimiter.bracket', next: '@expression'}],
        [/[^<{]+/, ''],
        [/[<{]/, ''],
      ],
      expression: [
        [/\{/, {token: 'delimiter.bracket', next: '@push'}],
        [/\}/, {token: 'delimiter.bracket', next: '@pop'}],
        {include: '@python'},
      ],
      python: [
        [/#.*$/, 'comment'],
        [/"[^"\n]*"|'[^'\n]*'/, 'string'],
        [/\b\d+(\.\d+)?\b/, 'number'],
        [/[a-zA-Z_]\w*/, {cases: {'@keywords': 'keyword', '@default': 'identifier'}}],
        [/[()[\]]/, '@brackets'],
        [/[=:,.+*/-]/, 'delimiter'],
      ],
    },
  });
  // SolveIT caches magic-to-language mappings when its editor first opens.
  if (typeof _langMap !== 'undefined') _langMap = undefined;
  // SolveIT's Shiki theme has no color rules for dynamically registered tokens.
  document.getElementById('hyper-token-colors')?.remove();
  const style = document.createElement('style');
  style.id = 'hyper-token-colors';
  style.textContent = `
    .hyper-tag {color:#116329!important}
    .hyper-keyword {color:#cf222e!important}
    .hyper-string {color:#0a3069!important}
    .hyper-attribute {color:#0550ae!important}
    .hyper-comment {color:#6e7781!important}
    .dark .hyper-tag {color:#7ee787!important}
    .dark .hyper-keyword {color:#ff7b72!important}
    .dark .hyper-string {color:#a5d6ff!important}
    .dark .hyper-attribute {color:#79c0ff!important}
    .dark .hyper-comment {color:#8b949e!important}
  `;
  document.head.append(style);
  window.hyperModelHooks?.forEach(hook => hook.dispose());
  const hooks = window.hyperModelHooks = [];
  function watch(model) {
    let decorations = [];
    function color() {
      if (model.isDisposed()) return;
      const source = model.getValue();
      if (!source.startsWith('%%hyper')) {
        decorations = model.deltaDecorations(decorations, []);
        return;
      }
      monaco.editor.setModelLanguage(model, language);
      const lines = source.split('\n');
      const spans = monaco.editor.tokenize(source, language).flatMap((tokens, row) =>
        tokens.flatMap((token, column) => {
          const type = token.type.split('.')[0];
          const kind = type === 'attribute' ? 'attribute' : type;
          if (!['tag', 'keyword', 'string', 'attribute', 'comment'].includes(kind)) return [];
          return [{range: new monaco.Range(row + 1, token.offset + 1,
            row + 1, (tokens[column + 1]?.offset ?? lines[row].length) + 1),
          options: {inlineClassName: `hyper-${kind}`}}];
        }));
      decorations = model.deltaDecorations(decorations, spans);
    }
    hooks.push(model.onDidChangeContent(color));
    hooks.push({dispose() {
      if (!model.isDisposed()) model.deltaDecorations(decorations, []);
    }});
    color();
  }
  monaco.editor.getModels().forEach(watch);
  hooks.push(monaco.editor.onDidCreateModel(watch));
})();
