<template>
	<div
		v-if="activeUser"
		class="discord-rpc-widget fixed bottom-3 right-3 z-30 select-none transition-all duration-300 ease-[cubic-bezier(0.16,1,0.3,1)]"
		:class="{
			'is-expanded': isExpanded,
			'opacity-40 hover:opacity-100': !isRpcEnabled,
		}"
		@mouseenter="isExpanded = true"
		@mouseleave="isExpanded = false"
	>
		<div
			class="flex items-center gap-2.5 rounded-none border border-white/10 bg-[#0d0f14]/90 px-2.5 py-1.5 shadow-[0_4px_24px_rgba(0,0,0,0.65)] backdrop-blur-xl transition-all duration-200 hover:border-white/20"
		>
			<div class="relative shrink-0">
				<img
					:src="playerHeadUrl"
					:alt="activeUser.name"
					class="size-7 rounded-none object-contain [image-rendering:pixelated]"
					@error="onHeadError"
				/>
				<div
					class="absolute -bottom-1 -right-1 flex size-3.5 items-center justify-center rounded-none bg-[#111217] p-0.5 shadow-sm"
					:title="isRpcEnabled ? 'Discord RPC Active' : 'Discord RPC Disabled'"
				>
					<DiscordIcon class="size-2.5 shrink-0 text-[#5865F2]" />
				</div>
				<span
					v-if="isRpcEnabled"
					class="absolute -top-0.5 -right-0.5 size-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_#34d399]"
				/>
			</div>

			<div class="flex flex-col min-w-0 pr-1">
				<div class="flex items-center gap-1.5">
					<span class="truncate text-xs font-bold text-contrast">
						{{ activeUser.name }}
					</span>
					<span
						v-if="activeInstanceName"
						class="rounded-none bg-sky-500/20 px-1 py-0.2 text-[9px] font-semibold text-sky-400 uppercase tracking-wider"
					>
						Playing
					</span>
				</div>
				<span class="truncate text-[10px] text-secondary font-medium leading-tight">
					{{ statusText }}
				</span>
			</div>

			<div v-show="isExpanded" class="flex items-center pl-1 border-l border-white/10">
				<button
					type="button"
					class="rounded-none px-1.5 py-0.5 text-[10px] font-semibold transition-colors"
					:class="
						isRpcEnabled
							? 'bg-emerald-500/20 text-emerald-300 hover:bg-emerald-500/30'
							: 'bg-white/5 text-secondary hover:bg-white/10 text-neutral-400'
					"
					@click.stop="toggleDiscordRpc"
				>
					{{ isRpcEnabled ? 'RPC ON' : 'RPC OFF' }}
				</button>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { DiscordIcon } from '@/assets/external'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_default_user, users } from '@/helpers/auth.js'
import { get_all } from '@/helpers/process.js'
import { resolvePlayerHeadUrl } from '@/helpers/rendering/player-head'

const { settings, updateSettings } = useAppSettings()

const isExpanded = ref(false)
const activeUser = ref<{ id: string; name: string } | null>(null)
const activeInstanceName = ref<string | null>(null)
const playerHeadUrl = ref('/steve_head.png')

const isRpcEnabled = computed(() => settings.value?.discord_rpc ?? true)

const statusText = computed(() => {
	if (!isRpcEnabled.value) return 'RPC Inactive'
	if (activeInstanceName.value) return activeInstanceName.value
	return 'Craftrinth • Idling'
})

async function updatePlayerHead() {
	if (!activeUser.value?.name) {
		playerHeadUrl.value = 'https://launcher-files.modrinth.com/assets/steve_head.png'
		return
	}
	try {
		const head = await resolvePlayerHeadUrl(activeUser.value.name, 64)
		playerHeadUrl.value = head
	} catch {
		playerHeadUrl.value = 'https://launcher-files.modrinth.com/assets/steve_head.png'
	}
}

watch(activeUser, () => {
	void updatePlayerHead()
})

function onHeadError() {
	playerHeadUrl.value = '/steve_head.png'
}

async function refreshActiveUser() {
	try {
		const defaultId = await get_default_user()
		const allUsers = (await users()) || []
		if (Array.isArray(allUsers) && allUsers.length > 0) {
			const found = defaultId
				? allUsers.find((u: any) => u.offline_profile?.id === defaultId || u.profile?.id === defaultId)
				: allUsers[0]
			if (found) {
				const name = found.offline_profile?.name || found.profile?.name || 'Player'
				const id = found.offline_profile?.id || found.profile?.id || ''
				if (activeUser.value?.name !== name) {
					activeUser.value = { id, name }
				}
				return
			}
		}
		activeUser.value = null
	} catch {
		activeUser.value = null
	}
}

async function refreshProcesses() {
	try {
		const running = await get_all()
		if (Array.isArray(running) && running.length > 0) {
			activeInstanceName.value = running[0].instance_name || 'Minecraft'
		} else {
			activeInstanceName.value = null
		}
	} catch {
		activeInstanceName.value = null
	}
}

async function toggleDiscordRpc() {
	if (settings.value) {
		await updateSettings({
			discord_rpc: !settings.value.discord_rpc,
		})
	}
}

let pollTimer: ReturnType<typeof setInterval> | undefined

onMounted(async () => {
	await Promise.all([refreshActiveUser(), refreshProcesses()])
	await updatePlayerHead()
	pollTimer = setInterval(() => {
		void refreshActiveUser()
		void refreshProcesses()
	}, 3000)
})

onUnmounted(() => {
	if (pollTimer) clearInterval(pollTimer)
})
</script>

<style scoped>
.discord-rpc-widget {
	contain: layout style;
	transform: translateZ(0);
	will-change: transform, opacity;
}
</style>

