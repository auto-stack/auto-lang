export type SectionType =
  | 'goals'
  | 'architecture'
  | 'designs'
  | 'plans'
  | 'reviews'
  | 'reports'
  | 'apis'
  | 'requirements'
  | 'analysis'
  | 'todos'

export type StatusBadge =
  | 'draft'
  | 'approved'
  | 'in_progress'
  | 'verified'
  | 'archived'
  | 'drift'

export interface LedgerSection {
  id: string
  type: SectionType
  title: string
  status: StatusBadge
  content: string
  depends_on?: string[]
  last_modified: number
  last_verified?: number
}

export interface LedgerDocument {
  project: string
  sections: LedgerSection[]
}
