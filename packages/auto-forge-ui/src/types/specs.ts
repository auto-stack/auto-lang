export type SectionType =
  | 'goals'
  | 'architecture'
  | 'designs'
  | 'plans'
  | 'tests'
  | 'reviews'
  | 'reports'
  | 'apis'
  | 'requirements'
  | 'analysis'
  | 'todos'

export type Status =
  | 'empty'
  | 'proposed'
  | 'draft'
  | 'under_review'
  | 'approved'
  | 'in_progress'
  | 'in_implementation'
  | 'implemented'
  | 'verified'
  | 'done'
  | 'archived'
  | 'rejected'
  | 'backlog'
  | 'ready'
  | 'in_review'
  | 'blocked'
  | 'superseded'
  | 'outdated'
  | 'stable'
  | 'deprecated'
  | 'published'
  | 'analysed'
  | 'obsolete'
  | 'drift'

export interface SpecItem {
  id: string
  title: string
  content: string
  status: Status
  depends_on?: string[]
  related?: string[]
  priority?: string
  assignee?: string
  test_file?: string
  created_at: number
  modified_at: number
  completed_at?: number
}

export interface SpecsSection {
  id: string
  section_type: SectionType
  title: string
  items: SpecItem[]
  status: Status
  content: string
  depends_on?: string[]
  last_modified: number
  last_verified?: number
}

export interface SpecsDocument {
  project: string
  version: number
  sections: SpecsSection[]
}
