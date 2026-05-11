import type { LanguageFn } from 'highlight.js';

const abt: LanguageFn = () => {
  const MNEMONICS = [
    'nop', 'pop', 'pop.n', 'dup', 'swap', 'drop', 'reserve',
    'const.i32', 'const.u8', 'const.0', 'const.1', 'const.f32',
    'const.f64', 'const.i64', 'const.u64', 'load.str',
    'set.field', 'set.elem', 'get.elem', 'get.field',
    'create.obj', 'create.arr', 'arr.len', 'mod.f', 'mod.d',
    'slice', 'create.tuple', 'get.tuple.field', 'promote.f64', 'ret.d',
    'create.range', 'create.range.eq', 'build.fstr', 'null.coalesce',
    'error.propagate', 'create.node', 'create.some', 'create.none',
    'create.ok', 'create.err', 'is.some', 'is.ok', 'unwrap.some',
    'unwrap.ok', 'unwrap.err', 'cast.i32', 'cast.u32', 'cast.i64',
    'cast.u64', 'cast.f64', 'cast.ptr', 'to.str', 'to.i32', 'to.f64',
    'f64.to.str', 'i64.to.str', 'u64.to.str', 'bool.to.str',
    'f64.to.i32', 'str.to.i64', 'f32.to.str', 'f32.to.i32',
    'to_str', 'is.nil', 'str.cat',
    'load.local', 'store.local', 'load.loc.0', 'load.loc.1', 'load.loc.2',
    'store.loc.0', 'store.loc.1',
    'add', 'sub', 'mul', 'div', 'mod', 'neg',
    'add.f', 'sub.f', 'mul.f', 'div.f', 'neg.f',
    'add.d', 'sub.d', 'mul.d', 'div.d', 'neg.d',
    'mod.u64', 'i32.to.f32', 'i64.to.f64', 'u64.to.f64',
    'add.u64', 'sub.u64', 'mul.u64', 'div.u64',
    'and', 'or', 'xor', 'not', 'shl', 'shr',
    'eq', 'ne', 'lt', 'gt', 'le', 'ge',
    'eq.d', 'ne.d', 'lt.d', 'gt.d', 'le.d', 'ge.d',
    'jmp', 'jmp.z', 'jmp.nz', 'jmp.l',
    'call', 'ret', 'call.nat', 'call.spec',
    'spawn', 'task.id', 'yield', 'sleep', 'join',
    'chan.new', 'send', 'recv', 'try.recv', 'spawn.go',
    'task.loop', 'handle.msg', 'reply',
    'closure', 'capture.var', 'load.captured', 'store.captured', 'call.closure',
    'create.list.int', 'create.list.str', 'create.list.bool',
    'list.push.int', 'list.pop.int', 'list.get.int', 'list.set.int',
    'create.list.int.inline', 'create.list.str.inline', 'create.list.bool.inline',
    'new.instance', 'construct.instance', 'get.generic.field', 'set.generic.field',
    'load.ref', 'store.ref', 'load.mut.ref', 'store.mut.ref',
    'fn.prolog', 'is.variant',
    'create.future', 'await.future', 'poll.future',
    '.line', 'print', 'halt',
  ];

  return {
    name: 'ABT',
    aliases: ['abt'],
    case_insensitive: false,
    contains: [
      // Comments: ; to end of line
      {
        className: 'comment',
        begin: /;|#/,
        end: /$/,
      },

      // Section directives
      {
        className: 'section',
        begin: /^\s*\.(strings|exports|code|object_keys|object_types|line)\b/,
        relevance: 10,
      },

      // Labels
      {
        className: 'label',
        begin: /^\s*[A-Za-z_][A-Za-z0-9_]*:/,
        relevance: 5,
      },

      // Label references: @name
      {
        className: 'link',
        begin: /@[A-Za-z_][A-Za-z0-9_]*/,
        relevance: 3,
      },

      // Parameter references: arg0, arg1, ...
      {
        className: 'variable',
        begin: /\barg\d+\b/,
        relevance: 2,
      },

      // Mnemonics
      {
        className: 'keyword',
        begin: new RegExp(`\\b(${MNEMONICS.join('|')})\\b`),
        relevance: 1,
      },

      // String pool / field indices
      {
        className: 'string',
        begin: /\bstr\[/,
        end: /\]/,
        relevance: 1,
      },
      {
        className: 'string',
        begin: /\bfield\[/,
        end: /\]/,
        relevance: 1,
      },

      // Native indices: nat#N
      {
        className: 'number',
        begin: /\bnat#\d+\b/,
        relevance: 1,
      },

      // Hex numbers
      {
        className: 'number',
        begin: /\b0x[0-9a-fA-F]+\b/,
        relevance: 0,
      },

      // Decimal / float numbers
      {
        className: 'number',
        begin: /\b-?\d+\.?\d*\b/,
        relevance: 0,
      },

      // Quoted strings
      {
        className: 'string',
        begin: /"/,
        end: /"/,
        contains: [
          {
            className: 'subst',
            begin: /\\./,
          },
        ],
      },
    ],
  };
};

export default abt;
