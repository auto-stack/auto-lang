import { StreamLanguage } from '@codemirror/language';

const keywords = new Set([
  'true', 'false', 'nil', 'null',
  'if', 'else', 'for', 'loop', 'is', 'in',
  'fn', 'type', 'union', 'tag', 'enum', 'grid', 'alias',
  'let', 'var', 'const', 'mut', 'move', 'view', 'take', 'copy', 'hold',
  'spec', 'use', 'pac', 'super', 'as', 'to', 'ext', 'static', 'shared',
  'impl', 'dep', 'has', 'break', 'continue', 'next', 'return',
  'routes', 'outlet', 'link', 'route', 'nav',
  'task', 'spawn', 'await', 'reply', 'go', 'on', 'node',
]);

const typeKeywords = new Set([
  'None', 'Some', 'Ok', 'Err', 'Link',
  'int', 'float', 'str', 'bool', 'void', 'char',
  'List', 'Map', 'Set', 'Option', 'Result',
]);

export const autoLanguage = StreamLanguage.define({
  name: 'auto',

  startState() {
    return { inString: false as false | '"' | 'f"' | "c'" | "'", stringPrefix: '' };
  },

  token(stream: any, state: any): string | null {
    // Comments
    if (stream.sol() && !state.inString) {
      if (stream.match('//')) {
        stream.skipToEnd();
        return 'comment';
      }
      if (stream.match('/*')) {
        let ch;
        while ((ch = stream.next()) != null) {
          if (ch === '*' && stream.next() === '/') {
            stream.eat('/');
            return 'comment';
          }
        }
        return 'comment';
      }
    }

    // Mid-line comment
    if (!state.inString && stream.match('//')) {
      stream.skipToEnd();
      return 'comment';
    }

    // Strings
    if (state.inString) {
      if (stream.peek() === '\\') {
        stream.next();
        stream.next();
        return 'string';
      }
      const quote = state.inString === 'f"' || state.inString === 'c"' ? '"' : state.inString === "'" ? "'" : '"';
      if (stream.peek() === quote) {
        stream.next();
        state.inString = false;
        return 'string';
      }
      stream.next();
      return 'string';
    }

    // F-strings and C-strings
    if (stream.match('f"')) {
      state.inString = 'f"';
      return 'string';
    }
    if (stream.match('c"')) {
      state.inString = 'c"';
      return 'string';
    }

    // Regular strings
    if (stream.peek() === '"') {
      stream.next();
      state.inString = '"';
      return 'string';
    }

    // Char literals
    if (stream.peek() === "'") {
      stream.next();
      if (stream.peek() === '\\') {
        stream.next();
        stream.next();
      } else {
        stream.next();
      }
      if (stream.peek() === "'") stream.next();
      return 'string';
    }

    // Numbers (hex, binary, float, int)
    if (stream.match(/^0x[0-9a-fA-F_]+/)) return 'number';
    if (stream.match(/^0b[01_]+/)) return 'number';
    if (stream.match(/^[0-9][0-9_]*\.[0-9][0-9_]*/)) return 'number';
    if (stream.match(/^[0-9][0-9_]*/)) return 'number';

    // Comptime keywords (#if, #for, #is)
    if (stream.match(/^#(if|for|is)\b/)) return 'keyword';
    // Annotations #[...]
    if (stream.match('#[')) {
      let ch;
      while ((ch = stream.next()) != null) {
        if (ch === ']') break;
      }
      return 'meta';
    }

    // Operators and punctuation
    if (stream.match('=>')) return 'punctuation';
    if (stream.match('->')) return 'punctuation';
    if (stream.match('..=')) return 'operator';
    if (stream.match('..')) return 'operator';
    if (stream.match('==')) return 'operator';
    if (stream.match('!=')) return 'operator';
    if (stream.match('<=')) return 'operator';
    if (stream.match('>=')) return 'operator';
    if (stream.match('??')) return 'operator';
    if (stream.match('?.') ) return 'operator';
    if (stream.match(/^[+\-*\/%=<>!&|^~?:]/)) return 'operator';

    // Punctuation
    if (stream.match(/^[{}()\[\];,.@#]/)) return 'punctuation';

    // Identifiers and keywords
    if (stream.match(/^[A-Za-z_]\w*/)) {
      const word = stream.current();
      if (keywords.has(word)) return 'keyword';
      if (typeKeywords.has(word)) return 'typeName';
      return 'variableName';
    }

    stream.next();
    return null;
  },

  languageData: {
    commentTokens: { line: '//', block: { open: '/*', close: '*/' } },
  },
});
