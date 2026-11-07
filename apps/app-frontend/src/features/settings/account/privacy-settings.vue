<script setup lang="ts">
import {
	defineMessages,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { onMounted, ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'

const { formatMessage } = useVIntl()
const settings = ref<any>(null)

onMounted(async () => {
	const current = await get()
	current.telemetry = false
	current.personalized_ads = false
	settings.value = current
	await set(current)
})

const messages = defineMessages({
	discordRichPresenceTitle: {
		id: 'app.settings.privacy.discord-rich-presence.title',
		defaultMessage: 'Discord Rich Presence',
	},
	discordRichPresenceDescription: {
		id: 'app.settings.privacy.discord-rich-presence.description',
		defaultMessage:
			'Show Craftrinth as your current activity on Discord. This does not affect Rich Presence added to instances by mods. Requires an app restart.',
	},
})

watch(
	() => settings.value?.discord_rpc,
	async (newVal, oldVal) => {
		if (newVal !== undefined && oldVal !== undefined && settings.value) {
			await set(settings.value)
		}
	},
)
</script>

<template>
	<div v-if="settings" class="flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.discordRichPresenceTitle) }}
			</h2>
			<p class="m-0 mt-1">
				{{ formatMessage(messages.discordRichPresenceDescription) }}
			</p>
		</div>
		<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
	</div>
</template>
