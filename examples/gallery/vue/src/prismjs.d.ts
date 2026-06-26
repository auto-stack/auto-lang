declare module 'prismjs' {
  const Prism: {
    highlightAll: () => void;
    highlightElement: (element: Element) => void;
    languages: Record<string, any>;
  };
  export default Prism;
}
