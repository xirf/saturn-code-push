<script setup lang="ts">
import { useQuery } from '@pinia/colada'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { request, errorMessage } from '~/lib/api/client'
import { z } from '~/lib/validation'
const schema = z.object({
  backend: z.enum(['local', 's3']),
  s3: z
    .object({ bucket: z.string(), endpoint: z.string(), region: z.string(), prefix: z.string() })
    .nullable(),
})
const scope = useSessionScope()
const identity = useIdentity()
const query = useQuery({
  key: () => ['private', scope.generation, 'storage-settings'],
  enabled: () => identity.session.data.value?.role === 'admin',
  query: async ({ signal }) => schema.parse(await request('/admin/storage', { signal })),
})
</script>
<template>
  <Card v-if="identity.session.data.value?.role === 'admin'">
    <CardHeader>
      <CardTitle>Artifact storage</CardTitle>
      <CardDescription>Automatic folders for apps, releases, and patches.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <p
        v-if="query.error.value"
        role="alert"
        class="text-sm text-destructive"
      >
        {{ errorMessage(query.error.value) }}
      </p>
      <template v-if="query.data.value">
        <p class="text-sm">
          New uploads:
          <strong>
            {{ query.data.value.backend === 's3' ? 'S3 compatible bucket' : 'Local disk' }}
          </strong>
        </p>
        <p
          v-if="query.data.value.s3"
          class="break-all text-sm text-muted-foreground"
        >
          Bucket: {{ query.data.value.s3.bucket }} · Region: {{ query.data.value.s3.region }}
          <br />
          Endpoint: {{ query.data.value.s3.endpoint }}
          <br />
          Prefix: {{ query.data.value.s3.prefix || '(none)' }}
        </p>
        <p class="text-sm text-muted-foreground">
          Choose the storage backend and credentials in your hosting provider's environment
          settings. Existing files keep their recorded location when you switch.
        </p>
        <NuxtLink
          to="/docs/artifact-storage"
          class="text-sm underline"
        >
          Storage setup and folder layout
        </NuxtLink>
      </template>
    </CardContent>
  </Card>
</template>
