// Plan 405 vue-ref mock data — seeds the prototype with RealWorld-shaped
// records (flattened: no {article:{...}} wrapper, matching the planned auto
// backend). These mirror the 023 toy's existing titles so the port is faithful.

export type User = { id: number; email: string; username: string; bio: string; image: string; token: string }
export type Article = { slug: string; title: string; description: string; body: string; tagList: string; author: string; favoritesCount: number; createdAt: string }
export type Comment = { id: number; body: string; author: string; createdAt: string }

export const seedUsers: User[] = [
  { id: 1, email: 'sarah@vercel.com', username: 'Sarah Chen', bio: 'Senior Frontend Engineer at Vercel. Writing about React, TypeScript, and the future of web development.', image: 'https://i.pravatar.cc/100?u=sarah', token: 'mock-token-sarah' },
  { id: 2, email: 'marcus@example.com', username: 'Marcus Johnson', bio: 'API craftsman. tRPC, GraphQL, type safety enthusiast.', image: 'https://i.pravatar.cc/100?u=marcus', token: 'mock-token-marcus' },
]

export const seedArticles: Article[] = [
  {
    slug: 'understanding-react-server-components',
    title: 'Understanding React Server Components',
    description: 'Server Components represent a paradigm shift in how we think about rendering. They allow you to render components on the server, reducing the JavaScript sent to the client.',
    body: 'Server Components represent a paradigm shift in how we think about rendering. They allow you to render components on the server, reducing the JavaScript sent to the client. This enables faster initial loads, direct access to backend resources, and a simpler mental model for data-heavy components.\n\nIn this article we walk through how RSC differs from SSR, when to reach for client components, and the new patterns the server-first model unlocks.',
    tagList: 'React,JavaScript,Performance',
    author: 'Sarah Chen',
    favoritesCount: 42,
    createdAt: '2026-03-15',
  },
  {
    slug: 'building-type-safe-apis-with-trpc',
    title: 'Building Type-Safe APIs with tRPC',
    description: 'tRPC eliminates the need for REST endpoints and client-side fetching. Your TypeScript types flow seamlessly from server to client.',
    body: 'tRPC eliminates the need for REST endpoints and client-side fetching. Your TypeScript types flow seamlessly from server to client. With end-to-end type safety, refactors become fearless and the contract between your layers is enforced by the compiler, not by hand-written schemas.\n\nWe build a small CRUD service and show how the client infers every procedure signature.',
    tagList: 'TypeScript,tRPC,API',
    author: 'Marcus Johnson',
    favoritesCount: 18,
    createdAt: '2026-03-12',
  },
  {
    slug: 'the-state-of-css-in-2026',
    title: 'The State of CSS in 2026',
    description: 'Container queries, cascade layers, and native nesting have changed everything. Here is what you need to know.',
    body: 'Container queries, cascade layers, and native nesting have changed everything. The CSS working group shipped long-requested features that used to require preprocessors or build steps.\n\nThis tour covers :has(), container queries, cascade layers, and native nesting — with practical migration notes.',
    tagList: 'CSS,Web,Design',
    author: 'Emily Park',
    favoritesCount: 27,
    createdAt: '2026-03-10',
  },
]

export const seedComments: Record<string, Comment[]> = {
  'understanding-react-server-components': [
    { id: 1, body: 'Great breakdown of RSC vs SSR — the mental model finally clicked for me.', author: 'Marcus Johnson', createdAt: '2026-03-16' },
    { id: 2, body: 'Would love a follow-up on streaming + suspense boundaries.', author: 'Emily Park', createdAt: '2026-03-16' },
  ],
  'building-type-safe-apis-with-trpc': [
    { id: 3, body: 'The end-to-end type safety story is what won my team over.', author: 'Sarah Chen', createdAt: '2026-03-13' },
  ],
  'the-state-of-css-in-2026': [] as Comment[],
}

export const seedTags: string[] = ['React', 'TypeScript', 'tRPC', 'CSS', 'JavaScript', 'API', 'Web', 'Design', 'Performance']
