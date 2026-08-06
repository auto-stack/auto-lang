// Plan 012 Batch A (gap 19): a facade composable whose object has its own
// `remove` method. The compiler must NOT rewrite `.recentFiles.remove(i)`
// to `.splice` — the facade's own method wins (R010 Info note).
export interface RecentFilesFacade {
  files: string[]
  remove(index: number): void
}

export function useRecentFiles(): RecentFilesFacade {
  const files: string[] = ['a.md', 'b.md', 'c.md']
  return {
    files,
    remove(index: number): void {
      files.splice(index, 1)
    },
  }
}
