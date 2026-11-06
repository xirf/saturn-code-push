import { useSessionQuery } from '~/features/identity/queries/session'
import { useCatalogQuery } from '~/features/workspace/queries/catalog'

// Before the console became application-scoped, these pages lived at the top level.
// Existing bookmarks and shared links are forwarded to their application-scoped equivalents.
const sections: Record<string, string> = {
  '/releases': 'releases',
  '/patches': 'patches',
  '/analytics': 'analytics',
  '/settings/management/api-keys': 'settings/api-keys',
  '/settings/integrations/github': 'settings/github',
}

export default defineNuxtRouteMiddleware(async (to) => {
  const path = to.path.replace(/\/+$/, '') || '/'
  if (path === '/applications') return navigateTo('/', { replace: true })
  const release = /^\/releases\/(\d+)$/.exec(path)
  const section = sections[path]
  if (!release && !section) return

  const session = useSessionQuery()
  await session.refresh()
  if (!session.data.value) return navigateTo('/login')
  const catalog = useCatalogQuery()
  try {
    await catalog.refresh()
  } catch {
    return navigateTo('/', { replace: true })
  }
  const data = catalog.data.value
  if (release) {
    const match = data?.releases.find((item) => String(item.id) === release[1])
    return navigateTo(
      match ? `/apps/${encodeURIComponent(match.app_id)}/releases/${match.id}` : '/',
      { replace: true },
    )
  }
  const app = data?.apps[0]
  return navigateTo(app ? `/apps/${encodeURIComponent(app.id)}/${section}` : '/', {
    replace: true,
  })
})
