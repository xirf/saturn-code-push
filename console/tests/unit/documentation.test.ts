import { describe, expect, it } from 'vitest'
import { renderDocumentation } from '../../scripts/documentation/render.mjs'

const routes = new Map([
  ['docs/start.md', 'start'],
  ['docs/assets.md', 'asset-updates'],
])

describe('build-time documentation rendering', () => {
  it('escapes source HTML and rejects executable links', () => {
    const page = renderDocumentation(
      '# Safety\n\n<script>window.compromised = true</script>\n\n[unsafe](javascript:alert(1))\n\n![image](javascript:alert(1))',
      'docs/start.md',
      routes,
    )
    expect(page.html).not.toContain('<script>')
    expect(page.html).not.toMatch(/(?:href|src)="javascript:/)
    expect(page.html).toContain('&lt;script&gt;')
  })

  it('preserves code as text and resolves guides without publishing filesystem paths', () => {
    const page = renderDocumentation(
      '# Start\n\n## Same heading\n\n## Same heading\n\n[Assets](assets.md#publish)\n\n```html\n<img src=x onerror=alert(1)>\n```',
      'docs/start.md',
      routes,
    )
    expect(page.links).toEqual(['/docs/asset-updates#publish'])
    expect(page.headings.map((heading: { id: string }) => heading.id)).toEqual([
      'start',
      'same-heading',
      'same-heading-1',
    ])
    expect(page.html).toContain('data-copy-code')
    expect(page.html).toContain('&lt;img src=x onerror=alert(1)&gt;')
    expect(page.html).not.toContain('<img src=x')
    expect(() => renderDocumentation('[Private](../../.env)', 'docs/start.md', routes)).toThrow(
      'no published target',
    )
  })
})
