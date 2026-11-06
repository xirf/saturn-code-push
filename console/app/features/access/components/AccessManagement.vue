<script setup lang="ts">
import { useAccess } from '../composables/useAccess'
import type { Account } from '../types/access'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { errorMessage } from '~/lib/api/client'
import { formatDate } from '~/lib/format'
import { ScrollArea } from '~/components/ui/scroll-area'

const props = defineProps<{ section: 'accounts' | 'keys' | 'password' }>()
const access = useAccess(props.section)
const workspace = useWorkspace()
const error = ref('')
const notice = ref('')
const busy = ref(false)
const accountOpen = ref(false)
const editing = ref<Account | null>(null)
const account = reactive({ username: '', password: '', role: 'viewer', disabled: false })
const key = reactive({ name: '', app_id: '', permission: 'deployments:write', expires_in_days: 30 })
// Inside /apps/:appId the keys section is scoped to that application.
const scoped = computed(() => Boolean(workspace.appId.value))
watch(
  () => workspace.appId.value,
  (id) => {
    if (id) key.app_id = id
  },
  { immediate: true },
)
const visibleKeys = computed(() =>
  (access.keys.data.value?.keys ?? []).filter(
    (item) => !scoped.value || item.app_id === workspace.appId.value,
  ),
)
const revealed = ref('')
const revealOpen = ref(false)
const keyOpen = ref(false)
const currentPassword = ref('')
const password = ref('')
const queryError = computed(() =>
  props.section === 'accounts'
    ? access.accounts.error.value
    : props.section === 'keys'
      ? access.keys.error.value
      : null,
)
watch(revealOpen, (open) => {
  if (!open) revealed.value = ''
})
onBeforeUnmount(() => {
  revealed.value = ''
  account.password = ''
  password.value = ''
  currentPassword.value = ''
})
function openAccount(value: Account | null) {
  editing.value = value
  Object.assign(account, {
    username: value?.username ?? '',
    password: '',
    role: value?.role ?? 'viewer',
    disabled: value?.disabled ?? false,
  })
  accountOpen.value = true
}
async function perform(operation: () => Promise<unknown>, message: string) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  notice.value = ''
  try {
    await operation()
    notice.value = message
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}
async function saveAccount() {
  await perform(async () => {
    try {
      if (editing.value)
        await access.updateAccount(editing.value.id, {
          role: account.role,
          disabled: account.disabled,
          ...(account.password ? { password: account.password } : {}),
        })
      else
        await access.createAccount({
          username: account.username,
          password: account.password,
          role: account.role,
        })
      accountOpen.value = false
    } finally {
      account.password = ''
    }
  }, 'Account saved. Updated accounts must sign in again.')
}
async function createKey() {
  await perform(async () => {
    const result = await access.createKey({ ...key })
    revealed.value = result.token
    keyOpen.value = false
    revealOpen.value = true
    key.name = ''
  }, 'API key created.')
}
async function changePassword() {
  await perform(async () => {
    try {
      await access.changePassword({
        current_password: currentPassword.value,
        password: password.value,
      })
    } finally {
      password.value = ''
      currentPassword.value = ''
    }
  }, '')
}
</script>

<template>
  <div
    data-testid="access-management"
    class="space-y-6"
    :aria-busy="
      access.accounts.asyncStatus.value === 'loading' || access.keys.asyncStatus.value === 'loading'
    "
  >
    <p
      v-if="error || queryError"
      role="alert"
      class="text-sm text-destructive"
    >
      {{ error || errorMessage(queryError) }}
    </p>
    <p
      v-if="notice"
      role="status"
      class="text-sm"
    >
      {{ notice }}
    </p>
    <template v-if="access.admin.value">
      <Card
        v-if="section === 'accounts'"
        id="accounts"
        class="scroll-mt-6"
      >
        <CardHeader>
          <CardTitle>Accounts</CardTitle>
          <CardDescription>
            Admins manage access. Publishers manage app releases and deployments. Viewers can
            inspect the workspace.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <Button @click="openAccount(null)">New account</Button>
          <div>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Username</TableHead>
                  <TableHead>Role</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-for="item in access.accounts.data.value?.accounts"
                  :key="item.id"
                >
                  <TableCell>{{ item.username }}</TableCell>
                  <TableCell>{{ item.role }}</TableCell>
                  <TableCell>{{ item.disabled ? 'Disabled' : 'Active' }}</TableCell>
                  <TableCell>
                    <Button
                      variant="outline"
                      size="sm"
                      :disabled="busy"
                      :aria-label="`Edit account ${item.username}`"
                      @click="openAccount(item)"
                    >
                      Edit
                    </Button>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
      <Card
        v-if="section === 'keys'"
        id="api-keys"
        class="scroll-mt-6"
      >
        <CardHeader class="flex flex-wrap items-start justify-between gap-4">
          <div class="space-y-1.5">
            <CardTitle>{{ scoped ? 'Keys for this application' : 'API keys' }}</CardTitle>
            <CardDescription>
              Create an app-scoped key for CI. The full key is displayed once. Resetting or
              disabling its creator revokes their keys.
            </CardDescription>
          </div>
          <Button @click="keyOpen = true">New API key</Button>
        </CardHeader>
        <CardContent class="space-y-5">
          <Dialog v-model:open="keyOpen">
            <DialogContent class="max-h-[90svh] overflow-hidden">
              <ScrollArea class="max-h-[80svh] w-full pr-3">
                <DialogHeader>
                  <DialogTitle>New API key</DialogTitle>
                  <DialogDescription>
                    {{
                      scoped
                        ? `Choose the access this key needs for ${workspace.application.value?.display_name}.`
                        : 'Choose an application and the access this key needs.'
                    }}
                  </DialogDescription>
                </DialogHeader>
                <form
                  class="space-y-4"
                  @submit.prevent="createKey"
                >
                  <label class="block space-y-2 text-sm">
                    Key name
                    <Input
                      v-model="key.name"
                      required
                      maxlength="128"
                      placeholder="GitHub Actions"
                    />
                  </label>
                  <label
                    v-if="!scoped"
                    class="block space-y-2 text-sm"
                  >
                    Application
                    <Select
                      :model-value="key.app_id"
                      @update:model-value="(value) => (key.app_id = String(value ?? ''))"
                    >
                      <SelectTrigger
                        aria-label="Application"
                        class="w-full"
                      >
                        <SelectValue placeholder="Select an application" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem
                          v-for="app in workspace.applications.value"
                          :key="app.id"
                          :value="app.id"
                        >
                          {{ app.display_name }}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </label>
                  <label class="block space-y-2 text-sm">
                    Permission
                    <Select
                      :model-value="key.permission"
                      @update:model-value="(value) => (key.permission = String(value ?? ''))"
                    >
                      <SelectTrigger
                        aria-label="Permission"
                        class="w-full"
                      >
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="deployments:write">Report deployments</SelectItem>
                        <SelectItem value="publish">Publish app releases and patches</SelectItem>
                      </SelectContent>
                    </Select>
                  </label>
                  <label class="block space-y-2 text-sm">
                    Expires in days
                    <Input
                      v-model.number="key.expires_in_days"
                      type="number"
                      min="1"
                      max="365"
                      required
                    />
                  </label>
                  <Button
                    type="submit"
                    :disabled="busy || !key.app_id"
                  >
                    Create API key
                  </Button>
                </form>
                <p
                  v-if="error"
                  role="alert"
                  class="text-sm text-destructive"
                >
                  {{ error }}
                </p>
              </ScrollArea>
            </DialogContent>
          </Dialog>
          <p
            v-if="!visibleKeys.length && !access.keys.asyncStatus.value.startsWith('loading')"
            class="rounded-lg border border-dashed px-4 py-8 text-center text-sm text-muted-foreground"
          >
            No API keys yet. Create a Publish key to let your CLI or CI release and patch this
            application.
          </p>
          <div v-else>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Application / permission</TableHead>
                  <TableHead>Creator</TableHead>
                  <TableHead>Expires / last used</TableHead>
                  <TableHead>Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-for="item in visibleKeys"
                  :key="item.id"
                >
                  <TableCell>
                    {{ item.name }}
                    <code class="block text-xs text-muted-foreground">{{ item.prefix }}…</code>
                  </TableCell>
                  <TableCell>
                    {{
                      workspace.applications.value.find((app) => app.id === item.app_id)
                        ?.display_name ?? item.app_id
                    }}
                    <span class="block text-xs">{{ item.permission }}</span>
                  </TableCell>
                  <TableCell>{{ item.creator }}</TableCell>
                  <TableCell>
                    {{ formatDate(new Date(item.expires_at * 1000).toISOString()) }}
                    <span class="block text-xs">
                      Last used: {{ item.last_used_at ? formatDate(item.last_used_at) : 'Never' }}
                    </span>
                  </TableCell>
                  <TableCell>
                    <span v-if="item.revoked_at">Revoked</span>
                    <span v-else-if="item.expires_at * 1000 <= Date.now()">Expired</span>
                    <Button
                      v-else
                      variant="outline"
                      size="sm"
                      :disabled="busy"
                      :aria-label="`Revoke key ${item.name}`"
                      @click="perform(() => access.revokeKey(item.id), 'API key revoked.')"
                    >
                      Revoke
                    </Button>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
    </template>
    <Card
      v-if="section === 'password'"
      id="password"
      class="scroll-mt-6"
    >
      <CardHeader>
        <CardTitle>Change password</CardTitle>
        <CardDescription>
          Changing your password signs out your sessions and revokes your API keys.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          class="max-w-5xl space-y-4"
          @submit.prevent="changePassword"
        >
          <label class="block space-y-2 text-sm">
            Current password
            <Input
              v-model="currentPassword"
              type="password"
              autocomplete="current-password"
              required
              maxlength="1024"
            />
          </label>
          <label class="block space-y-2 text-sm">
            New password
            <Input
              v-model="password"
              type="password"
              autocomplete="new-password"
              required
              minlength="12"
              maxlength="1024"
            />
          </label>
          <Button
            type="submit"
            :disabled="busy"
            class="h-auto min-h-9 max-w-full whitespace-normal py-2"
          >
            Change password and sign out
          </Button>
        </form>
      </CardContent>
    </Card>
    <Dialog
      :open="accountOpen"
      @update:open="
        (value) => {
          if (!busy) {
            accountOpen = value
            account.password = ''
          }
        }
      "
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ editing ? `Edit ${editing.username}` : 'New account' }}</DialogTitle>
          <DialogDescription>
            Keep at least one active administrator. Account changes invalidate sessions and revoke
            their API keys.
          </DialogDescription>
        </DialogHeader>
        <form
          class="space-y-4"
          @submit.prevent="saveAccount"
        >
          <label
            v-if="!editing"
            class="block space-y-2 text-sm"
          >
            Username
            <Input
              v-model="account.username"
              autocomplete="off"
              required
              maxlength="128"
            />
          </label>
          <label class="block space-y-2 text-sm">
            {{ editing ? 'Reset password (optional)' : 'Password' }}
            <Input
              v-model="account.password"
              type="password"
              autocomplete="new-password"
              :required="!editing"
              minlength="12"
              maxlength="1024"
            />
          </label>
          <label class="block space-y-2 text-sm">
            Role
            <Select
              :model-value="account.role"
              @update:model-value="(value) => (account.role = String(value ?? 'viewer'))"
            >
              <SelectTrigger
                aria-label="Role"
                class="w-full"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="viewer">Viewer</SelectItem>
                <SelectItem value="publisher">Publisher</SelectItem>
                <SelectItem value="admin">Admin</SelectItem>
              </SelectContent>
            </Select>
          </label>
          <label
            v-if="editing"
            class="flex items-center gap-2 text-sm"
          >
            <input
              v-model="account.disabled"
              type="checkbox"
            />
            Disable account
          </label>
          <p
            v-if="error"
            role="alert"
            class="text-sm text-destructive"
          >
            {{ error }}
          </p>
          <DialogFooter>
            <Button
              type="submit"
              :disabled="busy"
            >
              {{ busy ? 'Saving…' : 'Save account' }}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
    <Dialog v-model:open="revealOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Save your API key</DialogTitle>
          <DialogDescription>
            Copy this key into your CI secret store. It cannot be displayed again.
          </DialogDescription>
        </DialogHeader>
        <textarea
          :value="revealed"
          readonly
          aria-label="New API key"
          class="min-h-24 w-full rounded-md border bg-background p-3 font-mono text-xs"
        />
        <DialogFooter>
          <Button @click="revealOpen = false">I have saved the key</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
