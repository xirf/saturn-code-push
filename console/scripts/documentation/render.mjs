import path from 'node:path'
import MarkdownIt from 'markdown-it'
import GithubSlugger from 'github-slugger'

export function renderDocumentation(markdown, source, routes) {
  const parser = new MarkdownIt({ html: false, linkify: false, typographer: false })
  const slugger = new GithubSlugger()
  const headings = []
  const links = []
  const tokens = parser.parse(markdown, {})

  for (const [index, token] of tokens.entries()) {
    if (token.type === 'heading_open') {
      const text = tokens[index + 1]?.content ?? ''
      const id = slugger.slug(text)
      token.attrSet('id', id)
      headings.push({ id, text, level: Number(token.tag.slice(1)) })
    }
    for (const child of token.children ?? []) {
      if (child.type !== 'link_open') continue
      const href = child.attrGet('href')
      if (!href) continue
      const resolved = resolveDocumentationLink(href, source, routes)
      child.attrSet('href', resolved)
      links.push(resolved)
      if (/^https?:\/\//.test(resolved)) child.attrSet('rel', 'noreferrer')
    }
  }

  const fence = parser.renderer.rules.fence
  parser.renderer.rules.fence = (items, index, options, environment, renderer) => {
    const code = fence(items, index, options, environment, renderer)
    return `
      <div class="relative my-5 rounded-lg border bg-muted/30">
        <button
          type="button"
          data-copy-code
          aria-label="Copy code"
          class="absolute top-3 right-3 rounded-md border bg-background px-2 py-1 text-xs text-muted-foreground hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          Copy
        </button>
        ${code}
      </div>
    `
  }
  parser.renderer.rules.table_open = () => '<div class="my-6 overflow-x-auto"><table>\n'
  parser.renderer.rules.table_close = () => '</table></div>\n'

  return {
    html: parser.renderer.render(tokens, parser.options, {}),
    headings,
    links,
    searchText: tokens
      .filter((token) => token.type === 'inline')
      .map((token) => token.content)
      .join(' '),
  }
}

export function resolveDocumentationLink(href, source, routes) {
  if (/^(https?:|mailto:|#|\/)/.test(href)) return href
  const [file, fragment] = href.split('#', 2)
  const target = path.posix.normalize(path.posix.join(path.posix.dirname(source), file))
  const slug = routes.get(target)
  if (slug) return `/docs/${slug}${fragment ? `#${fragment}` : ''}`
  if (target.startsWith('console/public/licenses/')) {
    return `/licenses/${path.posix.basename(target)}`
  }
  throw new Error(`Documentation link has no published target: ${source} -> ${href}`)
}
