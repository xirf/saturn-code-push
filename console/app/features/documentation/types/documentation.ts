export interface DocumentationHeading {
  id: string
  text: string
  level: number
}

export interface DocumentationPage {
  slug: string
  title: string
  description: string
  section: string
  source: string
  html: string
  headings: DocumentationHeading[]
  searchText: string
}
