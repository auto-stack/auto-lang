import { createApp } from 'vue'
import App from './App.blocks.vue'
import './assets/index.css'
import 'prismjs/themes/prism-tomorrow.css'
import Prism from 'prismjs'

// Define custom 'auto' language for Prism
Prism.languages.auto = {
  'comment': /\/\/.*|\/\*[\s\S]*?\*\//,
  'string': {
    pattern: /f?"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'/,
    greedy: true
  },
  'keyword': /\b(?:widget|view|model|msg|fn|let|mut|const|if|else|for|in|return|use|type|spec|import|export|struct|enum|interface|extends|implements|new|true|false|null)\b/,
  'function': /\b[a-z_][a-z0-9_]*(?=\s*\()/i,
  'number': /\b\d+\.?\d*\b/,
  'operator': /[+\-*/%=<>!&|^~?:]+/,
  'punctuation': /[{}[\]();,.]/,
  'property': /\.[a-z_][a-z0-9_]*/i,
  'element': /\b(?:col|row|button|text|input|card|link|div|span|p|h1|h2|h3|h4|h5|h6|ul|ol|li|table|thead|tbody|tr|td|th|form|label|checkbox|switch|select|option|dialog|modal|toast|dropdown|menu|tab|tabs|accordion|badge|avatar|progress|slider|scroll|codeblock|pre|code|img|video|audio|canvas|svg|path|rect|circle|ellipse|line|polyline|polygon|header|footer|nav|main|aside|section|article|header|footer|sidebar|outlet|slot)\b/,
  'attr': /\([^)]*\)/,
};

import router from './router/blocks'

const app = createApp(App)
app.use(router)
app.mount('#app')
