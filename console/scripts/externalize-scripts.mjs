import { createHash } from 'node:crypto'
import { readdir, readFile, mkdir, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { parse, serialize } from 'parse5'

const publicDirectory = new URL('../.output/public/', import.meta.url)
const scriptDirectory = new URL('_nuxt/', publicDirectory)
await mkdir(scriptDirectory, { recursive: true })

async function* htmlFiles(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) yield* htmlFiles(path)
    else if (entry.name.endsWith('.html')) yield path
  }
}

async function externalize(node) {
  if (node.tagName === 'script') {
    const attributes = Object.fromEntries(node.attrs.map(({ name, value }) => [name, value]))
    if (attributes.type === 'importmap') {
      throw new Error('Disable Nuxt entryImportMap before building for the strict server CSP')
    }
    const executable = ['', 'module', 'text/javascript', 'application/javascript'].includes(
      attributes.type ?? '',
    )
    if (executable && !attributes.src) {
      const source = node.childNodes.map((child) => child.value ?? '').join('')
      if (source.trim()) {
        const hash = createHash('sha256').update(source).digest('hex').slice(0, 24)
        const filename = `bootstrap-${hash}.js`
        await writeFile(new URL(filename, scriptDirectory), source)
        node.attrs.push({ name: 'src', value: `/_nuxt/${filename}` })
        node.childNodes = []
      }
    }
  }
  for (const child of node.childNodes ?? []) await externalize(child)
}

// Nuxt's static bootstrap normally includes executable inline configuration.
// External files preserve execution order while retaining script-src 'self'.
for await (const file of htmlFiles(publicDirectory.pathname)) {
  const document = parse(await readFile(file, 'utf8'))
  await externalize(document)
  await writeFile(file, serialize(document))
}
