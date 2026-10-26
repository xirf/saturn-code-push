import { useSessionQuery } from '~/features/identity/queries/session'

export default defineNuxtRouteMiddleware(async () => {
  const session = useSessionQuery()
  await session.refresh()
  if (!session.data.value) return navigateTo('/login')
})
