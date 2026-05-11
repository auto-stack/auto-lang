import { StreamLanguage } from '@codemirror/language'
import type { StringStream } from '@codemirror/language'

// Keywords from the Auto lexer token.rs
const keywords = new Set([
  // Declarations
  'fn', 'let', 'mut', 'const', 'var', 'type', 'union', 'enum', 'tag',
  'alias', 'spec', 'ext', 'static', 'shared', 'impl', 'node',
  // Control flow
  'if', 'else', 'for', 'break', 'continue', 'loop', 'is', 'in',
  'on', 'as', 'to', 'return', 'next',
  // Ownership/parameter modes
  'view', 'move', 'copy', 'take', 'hold',
  // Literals
  'true', 'false', 'nil', 'null',
  // Option/Result
  'None', 'Some', 'Ok', 'Err',
  // Concurrency
  'task', 'spawn', 'await', 'reply', 'go',
  // Modules
  'use', 'pac', 'super', 'dep', 'has',
  // Boolean logic (legacy)
  'and', 'or',
  // UI/Routing
  'routes', 'outlet', 'link', 'route', 'nav', 'grid',
])

// Primitive types and type-related keywords
const types = new Set([
  'int', 'uint', 'byte', 'i8', 'i16', 'i64', 'u8', 'u16', 'u64',
  'usize', 'float', 'double', 'bool', 'char', 'void', 'str',
  'String', 'cstr', 'Handle', 'linear',
  'List', 'Map', 'Set', 'Option', 'Result',
  'Link',
])

function isDigit(ch: string): boolean {
  return ch >= '0' && ch <= '9'
}

function isHexDigit(ch: string): boolean {
  return isDigit(ch) || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F')
}

function isIdentifierStart(ch: string): boolean {
  return /[\p{L}_]/u.test(ch)
}

function isIdentifierPart(ch: string): boolean {
  return /[\p{L}\p{N}_-]/u.test(ch)
}

interface AutoState {
  inString: boolean
  stringType: string
  inComment: boolean
  inFString: boolean
  inChar: boolean
  inRawString: boolean
  inMultilineString: boolean
}

export const autoLanguage = StreamLanguage.define<AutoState>({
  name: 'auto',
  startState(): AutoState {
    return {
      inString: false,
      stringType: '',
      inComment: false,
      inFString: false,
      inChar: false,
      inRawString: false,
      inMultilineString: false,
    }
  },

  token(stream: StringStream, state: AutoState): string | null {
    // Handle multi-line comment /* */
    if (state.inComment) {
      if (stream.match('*/')) {
        state.inComment = false
        return 'comment'
      }
      stream.next()
      return 'comment'
    }

    // Handle multi-line string """ """
    if (state.inMultilineString) {
      if (stream.match('"""')) {
        state.inMultilineString = false
        return 'string'
      }
      stream.next()
      return 'string'
    }

    // Skip whitespace
    if (stream.eatSpace()) return null

    const ch = stream.peek()
    if (!ch) return null

    // Comments: //, ///, /*
    if (ch === '/' && stream.match('//')) {
      stream.skipToEnd()
      return 'comment'
    }
    if (ch === '/' && stream.match('/*')) {
      state.inComment = true
      return 'comment'
    }

    // Multi-line string """
    if (ch === '"' && stream.match('"""')) {
      state.inMultilineString = true
      return 'string'
    }

    // Character literals 'a' or '\n'
    if (ch === "'") {
      stream.next() // eat '
      if (stream.match('\\')) {
        stream.next() // eat escape char
      } else {
        stream.next() // eat char
      }
      if (stream.peek() === "'") {
        stream.next()
      }
      return 'string'
    }

    // C string c"..."
    if (ch === 'c' && stream.match('c"')) {
      while (!stream.eol() && stream.peek() !== '"') {
        if (stream.peek() === '\\') stream.next()
        stream.next()
      }
      if (stream.peek() === '"') stream.next()
      return 'string'
    }

    // F-string f"..." or f`...`
    if (ch === 'f' && (stream.match('f"') || stream.match('f`'))) {
      const delim = stream.string[stream.pos - 1]
      while (!stream.eol() && stream.peek() !== delim) {
        if (stream.peek() === '\\') {
          stream.next()
          stream.next()
        } else if (stream.peek() === '$' && stream.string[stream.pos + 1] === '{') {
          return 'string'
        } else {
          stream.next()
        }
      }
      if (stream.peek() === delim) stream.next()
      return 'string'
    }

    // Raw string / f-string with backtick `...`
    if (ch === '`') {
      stream.next()
      while (!stream.eol() && stream.peek() !== '`') {
        if (stream.peek() === '$' && stream.string[stream.pos + 1] === '{') {
          return 'string'
        }
        stream.next()
      }
      if (stream.peek() === '`') stream.next()
      return 'string'
    }

    // Regular double-quoted string "..."
    if (ch === '"') {
      stream.next()
      while (!stream.eol() && stream.peek() !== '"') {
        if (stream.peek() === '\\') {
          stream.next()
          stream.next()
        } else {
          stream.next()
        }
      }
      if (stream.peek() === '"') stream.next()
      return 'string'
    }

    // Numbers
    if (isDigit(ch) || (ch === '.' && isDigit(stream.string[stream.pos + 1] || ''))) {
      return tokenNumber(stream)
    }

    // Hex 0x... or binary 0b...
    if (ch === '0' && (stream.string[stream.pos + 1] === 'x' || stream.string[stream.pos + 1] === 'b')) {
      return tokenNumber(stream)
    }

    // Identifiers and keywords
    if (isIdentifierStart(ch)) {
      return tokenIdentifier(stream)
    }

    // Hash directives #if, #for, #is, #[, #{
    if (ch === '#') {
      stream.next()
      if (stream.match('if') || stream.match('for') || stream.match('is')) {
        return 'keyword'
      }
      if (stream.match('[')) {
        return 'meta'
      }
      if (stream.match('{')) {
        return 'macroName'
      }
      // Annotation like @primary
      return 'operator'
    }

    // At-annotations @...
    if (ch === '@') {
      stream.next()
      if (isIdentifierStart(stream.peek() || '')) {
        stream.eatWhile(isIdentifierPart)
      }
      return 'attributeName'
    }

    // Operators
    if (stream.match('==') || stream.match('!=') || stream.match('<=') || stream.match('>=') ||
        stream.match('->') || stream.match('=>') || stream.match('..=') || stream.match('??') ||
        stream.match('?.') || stream.match('.?') || stream.match('&&') || stream.match('||') ||
        stream.match('+=') || stream.match('-=') || stream.match('*=') || stream.match('/=') || stream.match('%=')) {
      return 'operator'
    }

    if (ch === '.' && stream.match('..')) {
      return 'operator'
    }

    if (ch === '.' && /[a-zA-Z]/.test(stream.string[stream.pos + 1] || '')) {
      stream.next() // eat .
      stream.eatWhile(/[a-zA-Z]/)
      return 'propertyName'
    }

    // Single-char operators and punctuation
    if ('+-*/%=<>!&|~:;,.[](){}'.indexOf(ch) >= 0) {
      stream.next()
      return 'operator'
    }

    // Unknown character
    stream.next()
    return null
  },

  languageData: {
    commentTokens: { line: '//', block: { open: '/*', close: '*/' } },
  },
})

function tokenNumber(stream: StringStream): string {
  const start = stream.pos
  const ch = stream.peek()

  // Hex
  if (ch === '0' && (stream.string[start + 1] === 'x' || stream.string[start + 1] === 'X')) {
    stream.next()
    stream.next()
    stream.eatWhile(isHexDigit)
    stream.eatWhile(/[uUiIfFdD]/)
    return 'number'
  }

  // Binary
  if (ch === '0' && (stream.string[start + 1] === 'b' || stream.string[start + 1] === 'B')) {
    stream.next()
    stream.next()
    stream.eatWhile(/[01]/)
    return 'number'
  }

  // Integer or float
  stream.eatWhile(isDigit)
  stream.eatWhile(/[_]/)
  stream.eatWhile(isDigit)

  if (stream.peek() === '.' && isDigit(stream.string[stream.pos + 1] || '')) {
    stream.next()
    stream.eatWhile(isDigit)
    stream.eatWhile(/[_]/)
    stream.eatWhile(isDigit)
  }

  if (stream.peek() === 'e' || stream.peek() === 'E') {
    stream.next()
    if (stream.peek() === '-' || stream.peek() === '+') stream.next()
    stream.eatWhile(isDigit)
  }

  // Suffixes: u, u8, i8, i16, i64, u16, u64, f, d, usize
  if (stream.match('usize')) return 'number'
  if (stream.match('i64')) return 'number'
  if (stream.match('i16')) return 'number'
  if (stream.match('i8')) return 'number'
  if (stream.match('u64')) return 'number'
  if (stream.match('u16')) return 'number'
  if (stream.match('u8')) return 'number'
  if (stream.match('u')) return 'number'
  if (stream.match('f')) return 'number'
  if (stream.match('d')) return 'number'

  return 'number'
}

function tokenIdentifier(stream: StringStream): string {
  stream.eatWhile(isIdentifierPart)
  const word = stream.current()

  if (keywords.has(word)) return 'keyword'
  if (types.has(word)) return 'typeName'

  // Check if next non-space char is ( → function call
  const rest = stream.string.slice(stream.pos)
  const nextChar = rest.trimStart()[0]
  if (nextChar === '(') return 'function'

  return 'variableName'
}
