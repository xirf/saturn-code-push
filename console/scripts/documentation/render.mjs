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
    const language = items[index].info.trim().split(/\s+/, 1)[0] || 'Code'
    return `
      <div data-code-block class="my-5 overflow-hidden rounded-lg border bg-muted/30">
        <div class="flex min-h-11 items-center justify-between gap-3 border-b bg-muted/50 px-3">
          <span class="min-w-0 truncate font-mono text-xs text-muted-foreground">${escapeHtml(language)}</span>
        <button
          type="button"
          data-copy-code
          aria-label="Copy code"
          class="shrink-0 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          Copy
        </button>
        </div>
        <div class="overflow-auto p-4 font-mono text-xs leading-relaxed [&_pre]:m-0 [&_pre]:overflow-visible [&_pre]:bg-transparent [&_pre]:p-0 [&_code]:font-mono [&_code]:text-xs">
        ${code}
        </div>
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

function escapeHtml(value) {
  return value.replace(
    /[&<>"']/g,
    (character) =>
      ({
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        '"': '&quot;',
        "'": '&#39;',
      })[character],
  )
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
