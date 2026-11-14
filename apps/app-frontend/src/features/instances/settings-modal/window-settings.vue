<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, Toggle, useVIntl } from '@modrinth/ui'
import { computed, onMounted, type Ref, ref, watch } from 'vue'

import { edit } from '@/helpers/instance'
import { get } from '@/helpers/settings.ts'
import type { AppSettings } from '@/helpers/types'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const globalSettings = ref<AppSettings | null>(null)

const overrideWindowSettings = ref(
	!!instance.value.game_resolution || !!instance.value.force_fullscreen,
)
const resolution: Ref<[number, number]> = ref(
	instance.value.game_resolution ?? [854, 480],
)
const fullscreenSetting: Ref<boolean> = ref(
	instance.value.force_fullscreen ?? false,
)

onMounted(async () => {
	const settings = await get().catch(handleError)
	if (settings) {
		globalSettings.value = settings
		if (!overrideWindowSettings.value) {
			resolution.value = settings.game_resolution ? (settings.game_resolution.slice() as [number, number]) : [854, 480]
			fullscreenSetting.value = settings.force_fullscreen ?? false
		}
	}
})

watch(overrideWindowSettings, (enabled) => {
	if (!enabled && globalSettings.value) {
		resolution.value = globalSettings.value.game_resolution ? (globalSettings.value.game_resolution.slice() as [number, number]) : [854, 480]
		fullscreenSetting.value = globalSettings.value.force_fullscreen ?? false
	}
})

const editInstanceObject = computed(() => {
	if (!overrideWindowSettings.value) {
		return {
			force_fullscreen: null,
			game_resolution: null,
		}
	}
	return {
		force_fullscreen: fullscreenSetting.value,
		game_resolution: fullscreenSetting.value ? null : resolution.value,
	}
})

watch(
	[overrideWindowSettings, resolution, fullscreenSetting],
	async () => {
		await edit(instance.value.id, editInstanceObject.value)
	},
	{ deep: true },
)

const messages = defineMessages({
	defaultSettingsDescription: {
		id: 'instance.settings.tabs.window.default-settings-description',
		defaultMessage: 'Your instance uses the default window settings unless you override them here.',
	},
	window: {
		id: 'instance.settings.tabs.window',
		defaultMessage: 'Custom window settings',
	},
	fullscreen: {
		id: 'instance.settings.tabs.window.fullscreen',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'instance.settings.tabs.window.fullscreen.description',
		defaultMessage: 'Make the game start in full screen when launched (using options.txt).',
	},
	width: {
		id: 'instance.settings.tabs.window.width',
		defaultMessage: 'Width',
	},
	widthDescription: {
		id: 'instance.settings.tabs.window.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	enterWidth: {
		id: 'instance.settings.tabs.window.width.enter',
		defaultMessage: 'Enter width...',
	},
	height: {
		id: 'instance.settings.tabs.window.height',
		defaultMessage: 'Height',
	},
	heightDescription: {
		id: 'instance.settings.tabs.window.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	enterHeight: {
		id: 'instance.settings.tabs.window.height.enter',
		defaultMessage: 'Enter height...',
	},
	mesa3dPreset: {
		id: 'instance.settings.tabs.window.mesa3d-preset',
		defaultMessage: 'Mesa3D Performance Presets',
	},
	mesa3dPresetDesc: {
		id: 'instance.settings.tabs.window.mesa3d-preset.desc',
		defaultMessage: 'Quick resolution presets optimized for CPU rendering when using Mesa3D software rasterization.',
	},
})

function applyMesaResolution(w: number, h: number) {
	overrideWindowSettings.value = true
	fullscreenSetting.value = false
	resolution.value = [w, h]
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<DefaultSettingsIntro
			tab="window"
			:description="formatMessage(messages.defaultSettingsDescription)"
		/>
		<div
			class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 pb-6"
		>
			<label for="override-window-settings" class="text-lg font-semibold text-contrast">
				{{ formatMessage(messages.window) }}
			</label>
			<Toggle id="override-window-settings" v-model="overrideWindowSettings" />
		</div>
		<div class="flex flex-col gap-6" :class="{ 'opacity-50': !overrideWindowSettings }">
			<div class="flex items-center gap-4 justify-between">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.fullscreen) }}
					</h2>
					<p class="m-0">
						{{ formatMessage(messages.fullscreenDescription) }}
					</p>
				</div>
				<Toggle id="fullscreen" v-model="fullscreenSetting" :disabled="!overrideWindowSettings" />
			</div>

			<div class="flex items-center gap-4 justify-between">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.width) }}
					</h2>
					<p class="m-0">
						{{ formatMessage(messages.widthDescription) }}
					</p>
				</div>
				<Input
					id="width"
					v-model="resolution[0]"
					autocomplete="off"
					:disabled="!overrideWindowSettings || fullscreenSetting"
					type="number"
					:placeholder="formatMessage(messages.enterWidth)"
				/>
			</div>

			<div class="flex items-center gap-4 justify-between">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.height) }}
					</h2>
					<p class="m-0">
						{{ formatMessage(messages.heightDescription) }}
					</p>
				</div>
				<Input
					id="height"
					v-model="resolution[1]"
					autocomplete="off"
					:disabled="!overrideWindowSettings || fullscreenSetting"
					type="number"
					:placeholder="formatMessage(messages.enterHeight)"
				/>
			</div>

			<!-- Mesa3D Performance Presets -->
			<div class="flex flex-col gap-2 p-3.5 rounded-lg border border-surface-5 bg-surface-2/40">
				<div class="flex flex-col gap-0.5">
					<h3 class="m-0 text-sm font-semibold text-contrast">
						{{ formatMessage(messages.mesa3dPreset) }}
					</h3>
					<p class="m-0 text-xs text-secondary">
						{{ formatMessage(messages.mesa3dPresetDesc) }}
					</p>
				</div>
				<div class="flex flex-wrap items-center gap-2 pt-1">
					<button
						type="button"
						class="px-2.5 py-1 text-xs rounded border border-surface-5 bg-surface-3 hover:bg-surface-4 text-contrast cursor-pointer transition-colors"
						:class="{ '!border-brand !bg-brand/20 font-semibold text-brand': resolution[0] === 854 && resolution[1] === 480 && !fullscreenSetting }"
						@click="applyMesaResolution(854, 480)"
					>
						480p (854 × 480) • Best FPS
					</button>
					<button
						type="button"
						class="px-2.5 py-1 text-xs rounded border border-surface-5 bg-surface-3 hover:bg-surface-4 text-contrast cursor-pointer transition-colors"
						:class="{ '!border-brand !bg-brand/20 font-semibold text-brand': resolution[0] === 1024 && resolution[1] === 768 && !fullscreenSetting }"
						@click="applyMesaResolution(1024, 768)"
					>
						768p (1024 × 768) • 4:3 Balanced
					</button>
					<button
						type="button"
						class="px-2.5 py-1 text-xs rounded border border-surface-5 bg-surface-3 hover:bg-surface-4 text-contrast cursor-pointer transition-colors"
						:class="{ '!border-brand !bg-brand/20 font-semibold text-brand': resolution[0] === 1280 && resolution[1] === 720 && !fullscreenSetting }"
						@click="applyMesaResolution(1280, 720)"
					>
						720p (1280 × 720) • HD Standard
					</button>
				</div>
			</div>
		</div>
	</div>
</template>
