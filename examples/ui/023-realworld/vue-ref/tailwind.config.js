/** @type {import('tailwindcss').Config} */
// RealWorld classic palette: brand green #5cb85c, link blue #5cb8cc.
// Kept plain (no shadcn tokens) to mirror what auto codegen emits from .at.
export default {
  content: ['./index.html', './src/**/*.{vue,ts}'],
  theme: { extend: {
    colors: { brand: '#5cb85c', 'brand-dark': '#449d44', link: '#5cb85c' },
  } },
  plugins: [],
}
