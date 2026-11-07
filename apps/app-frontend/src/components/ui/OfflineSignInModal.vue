<template>
	<NewModal
		ref="modal"
		:header="isFirstBoot ? 'Set Up Your Profile' : 'Sign in offline'"
		max-width="480px"
	>
		<form class="flex flex-col gap-4 p-5" @submit.prevent="handleSubmit">
			<p class="text-sm text-secondary m-0 leading-relaxed">
				{{
					isFirstBoot
						? 'Welcome to Craftrinth! Enter your player username and pick an optional skin to get started.'
						: 'Enter an offline Minecraft username. Skins are synchronized locally without requiring online Mojang authentication.'
				}}
			</p>

			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-secondary uppercase tracking-wider">
					Username
				</label>
				<input
					v-model="username"
					type="text"
					placeholder="Enter username (e.g. Player, Crafter)..."
					maxlength="16"
					class="px-3.5 py-2.5 rounded-none bg-surface-1 border border-solid border-surface-4 text-contrast text-sm outline-none focus:border-white transition-colors"
					required
				/>
			</div>

			<div class="flex flex-col gap-2">
				<label class="text-xs font-semibold text-secondary uppercase tracking-wider">
					Player Skin (Optional)
				</label>
				<div class="flex items-center justify-between p-3.5 rounded-none bg-surface-1 border border-solid border-surface-4 gap-3">
					<div class="flex items-center gap-3 min-w-0">
						<div class="w-12 h-12 rounded-none bg-surface-3 border border-surface-4 flex items-center justify-center overflow-hidden shrink-0">
							<img
								v-if="headPreviewUrl"
								:src="headPreviewUrl"
								alt="Player Head"
								class="w-full h-full object-contain [image-rendering:pixelated]"
							/>
							<Avatar
								v-else
								:src="'/steve_head.png'"
								size="36px"
							/>
						</div>
						<div class="flex flex-col min-w-0">
							<span class="text-sm font-medium text-contrast truncate">
								{{ skinBase64 ? 'Custom Skin Selected' : 'Standard Steve Model' }}
							</span>
							<span class="text-xs text-secondary truncate">
								{{ skinBase64 ? 'Will be saved locally' : 'Supports 64x64 or 64x32 .png' }}
							</span>
						</div>
					</div>

					<div class="flex items-center gap-1.5 shrink-0">
						<label class="button-base px-3 py-1.5 rounded-none bg-surface-2 hover:bg-surface-3 border border-solid border-surface-4 text-contrast text-xs font-semibold cursor-pointer transition-colors flex items-center gap-1.5">
							<UploadIcon class="w-3.5 h-3.5" />
							{{ skinBase64 ? 'Change' : 'Upload .png' }}
							<input
								type="file"
								accept="image/png"
								class="hidden"
								@change="handleFileInput"
							/>
						</label>
						<IconButton
							v-if="skinBase64"
							type="quiet"
							color="red"
							label="Remove"
							@click="skinBase64 = null"
						>
							<TrashIcon class="w-4 h-4" />
						</IconButton>
					</div>
				</div>
			</div>

			<div class="flex justify-end gap-2 pt-3 border-0 border-t border-solid border-surface-5">
				<Button type="quiet" @click="hide">
					{{ isFirstBoot ? 'Skip for now' : 'Cancel' }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="!username.trim() || submitting"
					native-type="submit"
				>
					<SpinnerIcon v-if="submitting" class="animate-spin" />
					<CheckIcon v-else />
					{{ isFirstBoot ? 'Start Playing' : 'Sign In' }}
				</Button>
			</div>
		</form>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, SpinnerIcon, TrashIcon, UploadIcon } from '@modrinth/assets'
import { Avatar, Button, IconButton, injectNotificationManager, NewModal } from '@modrinth/ui'
import { ref, watch } from 'vue'

import {
	add_offline_user,
	get_cracked_skin,
	save_cracked_skin,
	set_default_user,
	sync_cracked_skins_to_instances,
} from '@/helpers/auth'
import { generatePlayerHeadBlob } from '@/helpers/rendering/player-head'

const emit = defineEmits<{
	'signed-in': [creds: any]
}>()

const { handleError, addNotification } = injectNotificationManager()

const modal = ref<InstanceType<typeof NewModal>>()
const username = ref('')
const skinBase64 = ref<string | null>(null)
const headPreviewUrl = ref<string | null>(null)
const submitting = ref(false)
const isFirstBoot = ref(false)

async function updateHeadFromUsername(name: string) {
	if (skinBase64.value) return
	if (!name) {
		headPreviewUrl.value = null
		return
	}
	try {
		const existingSkin = await get_cracked_skin(name)
		if (existingSkin) {
			const blob = await generatePlayerHeadBlob(existingSkin, 64)
			headPreviewUrl.value = URL.createObjectURL(blob)
		} else {
			headPreviewUrl.value = `https://mc-heads.net/avatar/${encodeURIComponent(name)}/64`
		}
	} catch {
		headPreviewUrl.value = null
	}
}

watch(skinBase64, async (newVal) => {
	if (newVal) {
		try {
			const blob = await generatePlayerHeadBlob(newVal, 64)
			headPreviewUrl.value = URL.createObjectURL(blob)
		} catch {
			headPreviewUrl.value = null
		}
	} else if (username.value.trim()) {
		await updateHeadFromUsername(username.value.trim())
	} else {
		headPreviewUrl.value = null
	}
})

watch(username, (newVal) => {
	if (!skinBase64.value) {
		void updateHeadFromUsername(newVal.trim())
	}
})

function show(options?: { isFirstBoot?: boolean }) {
	isFirstBoot.value = !!options?.isFirstBoot
	username.value = ''
	skinBase64.value = null
	headPreviewUrl.value = null
	modal.value?.show()
}

function hide() {
	if (isFirstBoot.value) {
		localStorage.setItem('craftrinth_first_boot_prompted', 'true')
	}
	modal.value?.hide()
}

function handleFileInput(e: Event) {
	const input = e.target as HTMLInputElement
	if (!input.files || input.files.length === 0) return
	const file = input.files[0]
	if (!file.name.toLowerCase().endsWith('.png')) {
		addNotification({
			type: 'error',
			title: 'Invalid File',
			text: 'Please choose a .png skin image file.',
		})
		return
	}
	if (file.size > 2 * 1024 * 1024) {
		addNotification({
			type: 'error',
			title: 'File Too Large',
			text: 'Skin file cannot exceed 2 MB.',
		})
		return
	}

	const reader = new FileReader()
	reader.onload = () => {
		skinBase64.value = reader.result as string
	}
	reader.readAsDataURL(file)
	input.value = ''
}

async function handleSubmit() {
	const trimmed = username.value.trim()
	if (!trimmed || submitting.value) return

	if (!/^[a-zA-Z0-9_-]{1,16}$/.test(trimmed)) {
		addNotification({
			type: 'error',
			title: 'Invalid Username',
			text: 'Username must contain only letters, numbers, underscores, and dashes (max 16 characters).',
		})
		return
	}

	submitting.value = true

	try {
		const creds = await add_offline_user(trimmed)
		if (skinBase64.value) {
			await save_cracked_skin(trimmed, skinBase64.value).catch(() => {})
		}
		if (creds?.profile?.id) {
			await set_default_user(creds.profile.id).catch(() => {})
		}
		await sync_cracked_skins_to_instances().catch(() => {})

		addNotification({
			type: 'success',
			title: 'Signed In Offline',
			text: `Active player profile set to '${trimmed}'.`,
		})

		emit('signed-in', creds)
		hide()
	} catch (err) {
		handleError(err)
	} finally {
		submitting.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>

