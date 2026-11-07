<script setup lang="ts">
import {
	ArchiveIcon,
	EyeIcon,
	FolderOpenIcon,
	PlayIcon,
	PlusIcon,
	SettingsIcon,
	SpinnerIcon,
	StopCircleIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	ContextMenu,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import dirtBlockIcon from '@/assets/instance-icons/dirt-block.png'
import fabricIcon from '@/assets/instance-icons/fabric.png'
import forgeIcon from '@/assets/instance-icons/forge.png'
import grassBlockIcon from '@/assets/instance-icons/grass-block.png'
import neoforgeIcon from '@/assets/instance-icons/neoforge.png'
import optifineIcon from '@/assets/instance-icons/optifine.png'
import quiltIcon from '@/assets/instance-icons/quilt.png'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import AccountsCard from '@/features/minecraft-accounts/accounts-card.vue'
import type { MinecraftAccountsActions } from '@/features/minecraft-accounts/context'
import { trackEvent } from '@/helpers/analytics'
import { get_default_user, users } from '@/helpers/auth.js'
import { getInstanceIconUrl, kill, run } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/instance/types'
import type { ModrinthCredentials } from '@/helpers/mr_auth'
import { get_all } from '@/helpers/process'
import { resolvePlayerHeadUrl } from '@/helpers/rendering/player-head'
import { showInstanceInFolder } from '@/helpers/utils'
import { instanceListQueryOptions } from '@/pages/instance/query-options'

const props = defineProps<{
	credentials?: ModrinthCredentials | null | undefined
	hasPlus?: boolean
	showAd?: boolean
	visible: boolean
	requestSignIn?: () => void
}>()

const emit = defineEmits<{
	'create-instance': []
	'open-settings': []
	'manage-profiles': []
	'login-modrinth': []
}>()

const router = useRouter()
const route = useRoute()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const accounts = ref<InstanceType<typeof AccountsCard> | null>(null)
const minecraftAccounts = computed<MinecraftAccountsActions | null>(() => {
	const card = accounts.value
	if (!card) return null
	return {
		loginDisabled: card.loginDisabled,
		refreshValues: card.refreshValues,
		setEquippedSkin: card.setEquippedSkin,
		setLoginDisabled: card.setLoginDisabled,
		login: card.login,
	}
})

defineExpose({ minecraftAccounts })

const activeUserName = ref('Player')
const playerHeadUrl = ref('/steve_head.png')

async function refreshActiveUser() {
	if (accounts.value?.selectedAccount?.profile?.name) {
		activeUserName.value = accounts.value.selectedAccount.profile.name
	} else {
		try {
			const defaultId = await get_default_user()
			const allUsers = (await users()) || []
			if (Array.isArray(allUsers) && allUsers.length > 0) {
				const found = defaultId
					? allUsers.find(
							(u: any) =>
								u.profile?.id === defaultId || u.offline_profile?.id === defaultId,
						)
					: allUsers[0]
				if (found) {
					activeUserName.value =
						found.profile?.name || found.offline_profile?.name || 'Player'
				}
			}
		} catch {}
	}

	if (accounts.value?.avatarUrl) {
		playerHeadUrl.value = accounts.value.avatarUrl
		return
	}

	if (activeUserName.value && activeUserName.value !== 'Player') {
		const head = await resolvePlayerHeadUrl(activeUserName.value, 64)
		if (head) {
			playerHeadUrl.value = head
			return
		}
	}
	playerHeadUrl.value = '/steve_head.png'
}

watch(
	() => [accounts.value?.avatarUrl, accounts.value?.selectedAccount?.profile?.name],
	([newAvatar, newName]) => {
		if (typeof newAvatar === 'string' && newAvatar) {
			playerHeadUrl.value = newAvatar
		}
		if (typeof newName === 'string' && newName) {
			activeUserName.value = newName
		}
	},
	{ immediate: true },
)

function onHeadError() {
	if (accounts.value?.avatarUrl) {
		playerHeadUrl.value = accounts.value.avatarUrl
	} else {
		playerHeadUrl.value = '/steve_head.png'
	}
}

const instancesQuery = useQuery(instanceListQueryOptions())
const allInstances = computed<GameInstance[]>(() => {
	const list = (instancesQuery.data.value ?? []).slice()
	return list.sort((a, b) => {
		const dateACreated = dayjs(a.created)
		const dateAPlayed = a.last_played ? dayjs(a.last_played) : dayjs(0)
		const dateBCreated = dayjs(b.created)
		const dateBPlayed = b.last_played ? dayjs(b.last_played) : dayjs(0)
		const dateA = dateACreated.isAfter(dateAPlayed) ? dateACreated : dateAPlayed
		const dateB = dateBCreated.isAfter(dateBPlayed) ? dateBCreated : dateBPlayed
		if (dateA.isSame(dateB)) {
			return a.name.localeCompare(b.name)
		}
		return dateB.diff(dateA)
	})
})

const selectedInstanceId = ref<string | null>(null)
const selectedInstance = computed<GameInstance | null>(() => {
	if (!allInstances.value.length) return null
	if (selectedInstanceId.value) {
		const found = allInstances.value.find((i) => i.id === selectedInstanceId.value)
		if (found) return found
	}
	return allInstances.value[0]
})

watch(
	() => route.params.id,
	(id) => {
		if (typeof id === 'string' && id) {
			selectedInstanceId.value = id
		}
	},
	{ immediate: true },
)

watch(
	allInstances,
	(instances) => {
		if (instances.length > 0 && !selectedInstanceId.value) {
			selectedInstanceId.value = instances[0].id
		}
	},
	{ immediate: true },
)

function selectInstance(instance: GameInstance) {
	selectedInstanceId.value = instance.id
	if (route.params.id !== instance.id) {
		void router.push(`/instance/${encodeURIComponent(instance.id)}`)
	}
}

function getInstanceIcon(instance: GameInstance): string {
	if (instance.icon_path) {
		const custom = getInstanceIconUrl(instance.icon_path)
		if (custom) return custom
	}
	const name = (instance.name || '').toLowerCase()
	const loader = (instance.loader || '').toLowerCase()
	const version = (instance.game_version || '').toLowerCase()

	if (name.includes('snapshot') || version.includes('snapshot') || version.includes('w')) {
		return dirtBlockIcon
	}
	if (name.includes('optifine') || loader.includes('optifine')) {
		return optifineIcon
	}
	if (loader === 'fabric' || name.includes('fabric')) {
		return fabricIcon
	}
	if (loader === 'forge' || name.includes('forge')) {
		return forgeIcon
	}
	if (loader === 'neoforge' || name.includes('neoforge')) {
		return neoforgeIcon
	}
	if (loader === 'quilt' || name.includes('quilt')) {
		return quiltIcon
	}
	return grassBlockIcon
}

const runningInstances = ref<string[]>([])
async function checkProcesses() {
	const processes = (await get_all().catch(handleError)) ?? []
	runningInstances.value = processes.map((process: any) => process.instance_id)
}

useAppEvent('process', () => {
	void checkProcesses()
	void refreshActiveUser()
})

const isLaunching = ref(false)
const isSelectedRunning = computed(() => {
	return selectedInstance.value ? runningInstances.value.includes(selectedInstance.value.id) : false
})

async function playInstance(instance: GameInstance) {
	if (instance.quarantined || instance.install_stage !== 'installed') return
	await run(instance.id)
		.catch((err) => handleSevereError(err, { instanceId: instance.id }))
		.finally(() => {
			trackEvent('InstanceStart', {
				loader: instance.loader,
				game_version: instance.game_version,
				source: 'AppSidebar',
			})
		})
}

async function stopInstance(instance: GameInstance) {
	await kill(instance.id).catch(handleError)
	trackEvent('InstanceStop', {
		loader: instance.loader,
		game_version: instance.game_version,
		source: 'AppSidebar',
	})
	await checkProcesses()
}

async function handlePlayButtonClick() {
	if (!selectedInstance.value) {
		emit('create-instance')
		return
	}
	if (isSelectedRunning.value) {
		await stopInstance(selectedInstance.value)
		return
	}
	if (
		selectedInstance.value.quarantined ||
		selectedInstance.value.install_stage !== 'installed'
	) {
		return
	}
	isLaunching.value = true
	try {
		await playInstance(selectedInstance.value)
	} finally {
		isLaunching.value = false
		await checkProcesses()
	}
}

const instanceOptions = ref<InstanceType<typeof ContextMenu> | null>(null)
const messages = defineMessages({
	viewInstance: {
		id: 'app.quick-instance-switcher.view-instance',
		defaultMessage: 'View instance',
	},
	instanceActions: {
		id: 'app.quick-instance-switcher.actions.label',
		defaultMessage: 'Instance actions',
	},
	instanceLocked: {
		id: 'app.quick-instance-switcher.instance-locked',
		defaultMessage: 'This instance has been locked',
	},
})

function openContextMenu(event: MouseEvent, instance: GameInstance) {
	const playing = runningInstances.value.includes(instance.id)
	instanceOptions.value?.open(event, [
		playing
			? {
					id: 'stop',
					label: formatMessage(commonMessages.stopButton),
					icon: StopCircleIcon,
					tone: 'red',
					action: () => stopInstance(instance),
				}
			: {
					id: 'play',
					label: formatMessage(commonMessages.playButton),
					icon: PlayIcon,
					tone: 'brand',
					disabled: instance.quarantined || instance.install_stage !== 'installed',
					tooltip: instance.quarantined ? formatMessage(messages.instanceLocked) : undefined,
					action: () => playInstance(instance),
				},
		{ type: 'divider' },
		{
			id: 'open-instance',
			label: formatMessage(messages.viewInstance),
			icon: EyeIcon,
			action: () => router.push(`/instance/${encodeURIComponent(instance.id)}`),
		},
		{
			id: 'open-folder',
			label: formatMessage(commonMessages.openFolderButton),
			icon: FolderOpenIcon,
			action: () => showInstanceInFolder(instance.id),
		},
	])
}

onMounted(() => {
	void checkProcesses()
	void refreshActiveUser()
})
</script>

<template>
	<aside
		class="app-sidebar select-none relative flex h-full w-[260px] shrink-0 flex-col overflow-hidden border-r border-white/10 bg-[#16171b]/95 backdrop-blur-md z-20 text-neutral-200"
	>
		<div style="display: none !important;">
			<Suspense>
				<AccountsCard ref="accounts" />
			</Suspense>
		</div>

		<!-- Top Profile Section -->
		<div
			class="sidebar-profile-card flex items-center gap-3 p-3 border-b border-white/10 hover:bg-white/5 cursor-pointer transition-colors"
			@click="emit('manage-profiles')"
		>
			<div
				class="size-9 rounded bg-neutral-900 overflow-hidden shrink-0 border border-white/10 shadow-sm flex items-center justify-center"
			>
				<img
					:src="playerHeadUrl"
					class="w-full h-full object-contain [image-rendering:pixelated]"
					alt=""
					@error="onHeadError"
				/>
			</div>
			<div class="flex flex-col min-w-0 flex-1 leading-tight">
				<span class="text-[11px] text-neutral-400 font-normal">Logged in as</span>
				<span class="text-sm font-semibold text-white truncate">{{ activeUserName }}</span>
			</div>
		</div>

		<!-- Action Menu: Launcher Settings & Installations Manager -->
		<div class="sidebar-action-menu py-2 px-2 flex flex-col gap-0.5 border-b border-white/10">
			<button
				type="button"
				class="w-full flex items-center gap-2.5 px-3 py-2 rounded text-xs font-medium text-neutral-300 hover:text-white hover:bg-white/5 transition-colors cursor-pointer border-0 bg-transparent text-left"
				@click="emit('open-settings')"
			>
				<SettingsIcon class="size-4 shrink-0 text-neutral-400" />
				<span>Launcher Settings</span>
			</button>

			<div
				class="w-full flex items-center justify-between px-3 py-1.5 rounded text-xs font-medium text-neutral-300 hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
				@click="router.push('/')"
			>
				<div class="flex items-center gap-2.5 min-w-0">
					<ArchiveIcon class="size-4 shrink-0 text-neutral-400" />
					<span class="truncate">Installations Manager</span>
				</div>
				<button
					type="button"
					class="p-1 rounded hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer border-0 bg-transparent"
					title="Create new installation"
					@click.stop="emit('create-instance')"
				>
					<PlusIcon class="size-3.5 shrink-0" />
				</button>
			</div>
		</div>

		<!-- Instance List -->
		<div
			class="sidebar-instances-list flex-1 min-h-0 overflow-y-auto overflow-x-hidden p-2 flex flex-col gap-1"
		>
			<div
				v-for="instance in allInstances"
				:key="instance.id"
				class="sidebar-instance-item group relative flex items-center gap-2.5 px-3 py-2 text-xs transition-colors cursor-pointer rounded select-none"
				:class="[
					selectedInstance?.id === instance.id
						? 'bg-[#2563eb] text-white font-semibold shadow-sm'
						: 'text-neutral-300 hover:text-white hover:bg-white/5 font-medium',
				]"
				@click="selectInstance(instance)"
				@contextmenu.prevent.stop="(e) => openContextMenu(e, instance)"
			>
				<img
					:src="getInstanceIcon(instance)"
					class="size-5 shrink-0 object-contain [image-rendering:pixelated]"
					alt=""
				/>
				<span class="truncate flex-1">{{ instance.name }}</span>
				<span
					v-if="runningInstances.includes(instance.id)"
					class="size-2 rounded-full bg-emerald-400 animate-pulse shrink-0"
					title="Running"
				></span>
				<SpinnerIcon
					v-else-if="instance.install_stage !== 'installed'"
					class="size-3.5 animate-spin text-white/70 shrink-0"
				/>
			</div>

			<div
				v-if="allInstances.length === 0"
				class="p-4 text-center text-xs text-neutral-500 flex flex-col items-center gap-2"
			>
				<span>No installations found</span>
				<button
					type="button"
					class="px-2 py-1 text-xs bg-white/5 hover:bg-white/10 text-neutral-300 rounded border border-white/10 cursor-pointer"
					@click="emit('create-instance')"
				>
					+ Create Installation
				</button>
			</div>
		</div>

		<!-- Pinned Bottom PLAY Button -->
		<div class="sidebar-bottom-action p-3 pt-2 shrink-0 border-t border-white/10 bg-inherit">
			<button
				type="button"
				class="w-full py-2.5 px-3 rounded font-bold transition-all duration-150 flex flex-col items-center justify-center cursor-pointer select-none text-white shadow-md active:scale-[0.99] border-0"
				:class="[
					isSelectedRunning
						? 'bg-red-600 hover:bg-red-500 active:bg-red-700'
						: 'bg-[#2563eb] hover:bg-[#1d4ed8] active:bg-[#1e40af]',
					isLaunching ? 'opacity-80 cursor-wait' : '',
				]"
				:disabled="
					isLaunching || (selectedInstance && selectedInstance.install_stage !== 'installed')
				"
				@click="handlePlayButtonClick"
			>
				<div class="flex items-center gap-2">
					<SpinnerIcon v-if="isLaunching" class="size-4 animate-spin text-white" />
					<StopCircleIcon v-else-if="isSelectedRunning" class="size-4 text-white" />
					<PlayIcon v-else class="size-4 fill-white text-white" />
					<span class="text-sm font-black tracking-wider uppercase leading-none">
						{{ isLaunching ? 'LAUNCHING...' : isSelectedRunning ? 'STOP' : 'PLAY' }}
					</span>
				</div>
				<span class="text-[11px] font-normal opacity-90 truncate max-w-full px-1 mt-1 leading-none">
					{{ selectedInstance?.name || 'Create an Installation' }}
				</span>
			</button>
		</div>

		<ContextMenu ref="instanceOptions" :label="formatMessage(messages.instanceActions)" />
	</aside>
</template>

<style scoped>
.app-sidebar {
	--surface-4: #22242b;
	--surface-5: rgba(255, 255, 255, 0.1);
}

.sidebar-instances-list::-webkit-scrollbar {
	width: 4px;
}

.sidebar-instances-list::-webkit-scrollbar-track {
	background: transparent;
}

.sidebar-instances-list::-webkit-scrollbar-thumb {
	background: rgba(255, 255, 255, 0.15);
	border-radius: 2px;
}

.sidebar-instances-list::-webkit-scrollbar-thumb:hover {
	background: rgba(255, 255, 255, 0.3);
}
</style>
