import {expect, test, beforeAll, afterAll} from 'bun:test';
import {createHighlighter} from 'shiki';
import grammar from './Syntaxes/hyper.tmLanguage.json';
import {notebookGrammar, tokensProvider} from '../../examples/solveit-highlight.js';

let highlighter;

beforeAll(async () => {
  highlighter = await createHighlighter({langs: ['python', notebookGrammar(grammar)], themes: []});
});

afterAll(() => highlighter.dispose());

function scopesAt(source, text, language = 'hyper') {
  const offset = source.indexOf(text);
  const {tokens} = highlighter.getLanguage(language).tokenizeLine(source);
  return tokens.find(token => token.startIndex <= offset && token.endIndex > offset).scopes;
}

test('prop annotations use the Python grammar, including generic types', () => {
  const source = 'names: list[str]';
  for (const text of ['names', 'list', 'str']) {
    expect(scopesAt(source, text).slice(1)).toEqual(scopesAt(source, text, 'python').slice(1));
  }
  expect(scopesAt(source, 'str').length).toBeGreaterThan(1);
});

test('the props separator is highlighted', () => {
  expect(scopesAt('---', '---')).toContain('keyword.control.hyper');
});

test('HTML tags and text stay distinct', () => {
  expect(scopesAt('<button>for sale</button>', 'button')).toContain('entity.name.tag.html');
  expect(scopesAt('<button>for sale</button>', 'for')).toEqual(['source.hyper']);
});

test('component tags keep their shared grammar scopes', () => {
  expect(scopesAt('<{Button} />', 'Button')).toContain('support.class.component.html');
});

test('notebook support only prepends the magic header rule', () => {
  expect(notebookGrammar(grammar).patterns.slice(1)).toEqual(grammar.patterns);
  expect(scopesAt('%%hyper Button', '%%hyper')).toContain('keyword.control.hyper');
});

test('Monaco tokenization preserves multiline grammar state', () => {
  const provider = tokensProvider(highlighter);
  const first = provider.tokenize('<!-- comment', provider.getInitialState());
  const second = provider.tokenize('continued --> <button>', first.endState);

  expect(second.tokens[0].scopes).toBe('comment.block.html');
  expect(second.tokens.some(token => token.scopes === 'entity.name.tag.html')).toBe(true);
  expect(first.endState.equals(first.endState.clone())).toBe(true);
  expect(first.endState.equals(second.endState)).toBe(false);
});
