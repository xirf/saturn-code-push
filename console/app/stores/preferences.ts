import { defineStore } from 'pinia'
import { ref } from 'vue'

export const usePreferencesStore = defineStore('preferences', () => {
  const selectedAppId = ref('')
  const mobileNavigationOpen = ref(false)
  const periodDays = ref(7)

  function reset() {
    selectedAppId.value = ''
    mobileNavigationOpen.value = false
    periodDays.value = 7
  }

  return { selectedAppId, mobileNavigationOpen, periodDays, reset }
})
