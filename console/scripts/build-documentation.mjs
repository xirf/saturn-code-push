import { mkdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { format } from 'prettier'
import { documentationSources } from './documentation/catalog.mjs'
import { renderDocumentation } from './documentation/render.mjs'

const consoleRoot = fileURLToPath(new URL('..', import.meta.url))
const repositoryRoot = process.env.SATURN_DOCS_ROOT ?? path.resolve(consoleRoot, '..')
const routes = new Map(documentationSources.map((page) => [page.source, page.slug]))
const pages = []

for (const page of documentationSources) {
  const contents = await readFile(path.join(repositoryRoot, page.source), 'utf8')
  const markdown =
    page.source === 'LICENSE'
      ? ['# GNU Lesser General Public License v3.0', '', '```text', contents, '```'].join('\n')
      : contents
  const rendered = renderDocumentation(markdown, page.source, routes)
  pages.push({ ...page, ...rendered })
}

for (const page of pages) {
  for (const href of page.links) {
    const match = /^\/docs\/([^#]+)#(.+)$/.exec(href)
    if (!match) continue
    const target = pages.find((candidate) => candidate.slug === match[1])
    if (!target?.headings.some((heading) => heading.id === match[2])) {
      throw new Error(`Documentation heading does not exist: ${page.source} -> ${href}`)
    }
  }
}

const output = path.join(consoleRoot, 'app/generated/documentation.json')
await mkdir(path.dirname(output), { recursive: true })
await writeFile(output, await format(JSON.stringify(pages), { parser: 'json' }))
process.stdout.write(`Generated ${pages.length} documentation pages from repository Markdown.\n`)
