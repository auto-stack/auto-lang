// PLAN-825 T-06 V8/Wasm raw benchmark. All samples are emitted; the
// parent-side PowerShell analyzer computes statistics and the gate verdict.
import fs from 'node:fs';
import crypto from 'node:crypto';

const WARMUPS = 10;
const SAMPLES = 30;
const args = process.argv.slice(2);
const opt = (name, fallback = '') => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : fallback;
};
const engine = opt('--engine', 'v8-js-ignition');
const out = opt('--out', '-');
const fp = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const writer = out === '-' ? process.stdout : fs.createWriteStream(out);
const emit = row => writer.write(JSON.stringify({
  engine,
  node: process.versions.node,
  v8: process.versions.v8,
  v8_flags: process.execArgv,
  ...row,
}) + '\n');

function u32(n) {
  const out = [];
  do {
    let b = n & 0x7f;
    n >>>= 7;
    if (n) b |= 0x80;
    out.push(b);
  } while (n);
  return out;
}
const str = s => [...u32(s.length), ...Buffer.from(s)];
const section = (id, body) => [id, ...u32(body.length), ...body];
const vector = items => [...u32(items.length), ...items.flat()];
const funcType = (params, results) => [0x60, ...vector(params), ...vector(results)];
const I32 = 0x7f;
const CONST = n => [0x41, ...u32(n)];
const GET = n => [0x20, ...u32(n)];
const SET = n => [0x21, ...u32(n)];
const CALL = n => [0x10, ...u32(n)];
const ADD = [0x6a], SUB = [0x6b], EQZ = [0x45], GT_S = [0x4a];
const END = [0x0b], RETURN = [0x0f], BR = n => [0x0c, ...u32(n)];
const BR_IF = n => [0x0d, ...u32(n)];

function moduleFor(name) {
  let types = [funcType([], [I32])];
  let funcs = [0];
  let bodies = [];
  if (name.startsWith('short-b')) {
    const count = Number(name.slice(7));
    const body = [0];
    for (let i = 0; i < count; i++) body.push(0x02, 0x40, ...BR(0), 0x0b);
    body.push(...CONST(42), ...END);
    bodies.push(body);
  } else if (name === 'long1024') {
    const body = [1, 1, I32, ...CONST(0), ...SET(0)];
    for (let i = 0; i < 1024; i++) body.push(...GET(0), ...CONST(1), ...ADD, ...SET(0));
    body.push(...GET(0), ...END);
    bodies.push(body);
  } else if (name === 'arithmetic') {
    const body = [
      1, 2, I32,
      ...CONST(1), ...SET(0),
      ...CONST(0), ...SET(1),
      0x02, 0x40,
      0x03, 0x40,
      ...GET(0), ...CONST(1000), ...GT_S, ...BR_IF(1),
      ...GET(1), ...GET(0), ...ADD, ...SET(1),
      ...GET(0), ...CONST(1), ...ADD, ...SET(0),
      ...BR(0),
      0x0b,
      0x0b,
      ...GET(1), ...END,
    ];
    bodies.push(body);
  } else if (name === 'recursive32') {
    types = [funcType([], [I32]), funcType([I32], [I32])];
    funcs = [0, 1];
    bodies.push([0, ...CONST(32), ...CALL(1), ...END]);
    bodies.push([
      0, ...GET(0), ...EQZ,
      0x04, 0x7f,
      ...CONST(0),
      0x05,
      ...GET(0), ...GET(0), ...CONST(1), ...SUB, ...CALL(1), ...ADD,
      0x0b,
      ...END,
    ]);
  } else {
    throw new Error(`unknown workload ${name}`);
  }
  const bytes = [
    0, 97, 115, 109, 1, 0, 0, 0,
    ...section(1, vector(types)),
    ...section(3, vector(funcs)),
    ...section(7, [1, ...str('main'), 0, 0]),
    ...section(10, vector(bodies.map(body => [...u32(body.length), ...body]))),
  ];
  return Buffer.from(bytes);
}

function jsFor(name) {
  if (name.startsWith('short-b')) {
    const count = Number(name.slice(7));
    let body = '';
    for (let i = 0; i < count; i++) body += `b${i}: { break b${i}; }\n`;
    return `return function main(){ ${body} return 42; }`;
  }
  if (name === 'long1024') {
    let body = 'let x=0;';
    for (let i = 0; i < 1024; i++) body += 'x=x+1;';
    return `return function main(){ ${body} return x; }`;
  }
  if (name === 'arithmetic') {
    return 'return function main(){ let s=0; for(let i=1;i<=1000;i++){s+=i;} return s; }';
  }
  if (name === 'recursive32') {
    return 'return function main(){ function sum(n){ return n<=1 ? n : n+sum(n-1); } return sum(32); }';
  }
  throw new Error(`unknown workload ${name}`);
}

const workloads = [
  ['arithmetic', 500500],
  ['short-b1', 42], ['short-b8', 42], ['short-b64', 42], ['short-b256', 42], ['short-b1024', 42],
  ['long1024', 1024],
  ['recursive32', 528],
];

for (const [name, expected] of workloads) {
  const input = engine === 'v8-js-ignition' ? Buffer.from(jsFor(name)) : moduleFor(name);
  const inputFp = fp(input);
  const startedLoad = performance.now();
  let run;
  if (engine === 'v8-js-ignition') {
    run = new Function(input.toString())();
  } else {
    const module = new WebAssembly.Module(input);
    const instance = new WebAssembly.Instance(module);
    run = instance.exports.main;
  }
  emit({ workload: name, phase: 'load', sample: 0, ms: performance.now() - startedLoad,
    expected, value: expected, input_sha256: inputFp });
  const invoke = (phase, sample) => {
    const started = performance.now();
    const value = run();
    const ms = performance.now() - started;
    if (value !== expected) throw new Error(`${engine}/${name}: ${value} != ${expected}`);
    emit({ workload: name, phase, sample, ms, expected, value, input_sha256: inputFp });
  };
  invoke('first', 0);
  for (let i = 1; i <= WARMUPS; i++) invoke('warmup', i);
  for (let i = 1; i <= SAMPLES; i++) invoke('hot', i);
  emit({ workload: name, phase: 'process-peak', sample: 0, ms: 0,
    expected, value: expected, input_sha256: inputFp,
    private_bytes_peak: process.resourceUsage().maxRSS * 1024 });
}
await new Promise(resolve => writer.end ? writer.end(resolve) : resolve());
