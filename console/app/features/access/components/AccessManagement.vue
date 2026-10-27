<script setup lang="ts">
import { useAccess } from '../composables/useAccess'
import type { Account } from '../types/access'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { errorMessage } from '~/lib/api/client'
import { formatDate } from '~/lib/format'

const access = useAccess()
const workspace = useWorkspace()
const error = ref('')
const notice = ref('')
const busy = ref(false)
const accountOpen = ref(false)
const editing = ref<Account | null>(null)
const account = reactive({ username: '', password: '', role: 'viewer', disabled: false })
const key = reactive({ name: '', app_id: '', permission: 'deployments:write', expires_in_days: 30 })
const revealed = ref('')
const revealOpen = ref(false)
const currentPassword = ref('')
const password = ref('')
const queryError = computed(
  () => access.accounts.error.value || access.keys.error.value || access.audit.error.value,
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
      access.accounts.asyncStatus.value === 'loading' ||
      access.keys.asyncStatus.value === 'loading' ||
      access.audit.asyncStatus.value === 'loading'
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
      <Card>
        <CardHeader>
          <CardTitle>Accounts</CardTitle>
          <CardDescription>
            Admins manage access. Publishers manage app releases and deployments. Viewers can
            inspect the workspace.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <Button @click="openAccount(null)">New account</Button>
          <div class="overflow-x-auto">
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
      <Card>
        <CardHeader>
          <CardTitle>API keys</CardTitle>
          <CardDescription>
            Create an app-scoped key for CI. The full key is displayed once. Resetting or disabling
            its creator revokes their keys.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-5">
          <form
            class="grid gap-4 sm:grid-cols-2"
            @submit.prevent="createKey"
          >
            <label class="space-y-2 text-sm">
              Key name
              <Input
                v-model="key.name"
                required
                maxlength="128"
                placeholder="GitHub Actions"
              />
            </label>
            <label class="space-y-2 text-sm">
              Application
              <select
                v-model="key.app_id"
                aria-label="Application"
                required
                class="block w-full rounded-md border bg-background p-2"
              >
                <option
                  disabled
                  value=""
                >
                  Select an application
                </option>
                <option
                  v-for="app in workspace.applications.value"
                  :key="app.id"
                  :value="app.id"
                >
                  {{ app.display_name }}
                </option>
              </select>
            </label>
            <label class="space-y-2 text-sm">
              Permission
              <select
                v-model="key.permission"
                aria-label="Permission"
                class="block w-full rounded-md border bg-background p-2"
              >
                <option value="deployments:write">Report deployments</option>
                <option value="publish">Publish app releases and patches</option>
              </select>
            </label>
            <label class="space-y-2 text-sm">
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
          <div class="overflow-x-auto">
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
                  v-for="item in access.keys.data.value?.keys"
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
      <Card>
        <CardHeader>
          <CardTitle>Audit history</CardTitle>
          <CardDescription>
            Successful management changes with the authenticated account or API key responsible.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>When</TableHead>
                  <TableHead>Who</TableHead>
                  <TableHead>Action</TableHead>
                  <TableHead>Resource</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-for="event in access.audit.data.value?.events"
                  :key="event.id"
                >
                  <TableCell>{{ formatDate(event.occurred_at) }}</TableCell>
                  <TableCell>
                    {{ event.actor_name }}
                    <span class="block text-xs text-muted-foreground">{{ event.actor_kind }}</span>
                  </TableCell>
                  <TableCell>{{ event.action }}</TableCell>
                  <TableCell class="max-w-xs break-all text-xs">{{ event.resource }}</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
          <div class="flex gap-2">
            <Button
              v-if="access.before.value"
              variant="outline"
              @click="access.before.value = null"
            >
              Latest
            </Button>
            <Button
              v-if="access.audit.data.value?.events.length === 100"
              variant="outline"
              @click="access.before.value = access.audit.data.value.events.at(-1)?.id ?? null"
            >
              Older events
            </Button>
          </div>
        </CardContent>
      </Card>
    </template>
    <Card>
      <CardHeader>
        <CardTitle>Change password</CardTitle>
        <CardDescription>
          Changing your password signs out your sessions and revokes your API keys.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          class="grid gap-4 sm:grid-cols-2"
          @submit.prevent="changePassword"
        >
          <label class="space-y-2 text-sm">
            Current password
            <Input
              v-model="currentPassword"
              type="password"
              autocomplete="current-password"
              required
              maxlength="1024"
            />
          </label>
          <label class="space-y-2 text-sm">
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
            <select
              v-model="account.role"
              aria-label="Role"
              class="w-full rounded-md border bg-background p-2"
            >
              <option value="viewer">Viewer</option>
              <option value="publisher">Publisher</option>
              <option value="admin">Admin</option>
            </select>
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
