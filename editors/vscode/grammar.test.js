import {afterAll, beforeAll, expect, test} from 'bun:test';
import {createHighlighter} from 'shiki';
import grammar from './Syntaxes/hyper.tmLanguage.json';

let highlighter;
beforeAll(async () => {
  highlighter = await createHighlighter({langs: ['python', {...grammar, name: 'hyper'}], themes: []});
});
afterAll(() => highlighter.dispose());

function scopesAt(source, text) {
  const offset = source.indexOf(text);
  const {tokens} = highlighter.getLanguage('hyper').tokenizeLine(source);
  return tokens.find(token => token.startIndex <= offset && offset < token.endIndex).scopes;
}

test('defined components have a keyword and a function name', () => {
  const source = 'component Button(*, label: str = "Save"):';
  expect(scopesAt(source, 'component')).toContain('storage.type.function.hyper');
  expect(scopesAt(source, 'Button')).toContain('entity.name.function.python');
  expect(scopesAt(source, 'str')).toContain('support.type.python');
});

test('multiline signatures return to HTML after the colon', () => {
  const language = highlighter.getLanguage('hyper');
  let state;
  for (const line of ['component Button(', '    *, label: str,', '):']) {
    state = language.tokenizeLine(line, state).ruleStack;
  }
  const {tokens} = language.tokenizeLine('    <button>{label}</button>', state);
  expect(tokens.some(token => token.scopes.includes('entity.name.tag.html'))).toBe(true);
});

test('props and loop expressions use Python highlighting', () => {
  expect(scopesAt('names: list[str]', 'str')).toContain('support.type.python');
  expect(scopesAt('for name in names:', 'in')).toContain('keyword.operator.logical.python');
  expect(scopesAt('---', '---')).toContain('keyword.control.hyper');
});

test('HTML text does not become a Python keyword', () => {
  expect(scopesAt('<p>Hyper in SolveIT</p>', 'in')).toEqual(['source.hyper']);
});
