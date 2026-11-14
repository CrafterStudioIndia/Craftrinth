<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, Input, Toggle, useVIntl } from '@modrinth/ui'

import { useDefaultInstanceSettings } from './use-default-instance-settings'

const { formatMessage } = useVIntl()
const { settings, settingsQuery } = useDefaultInstanceSettings(
	(value) => ({
		force_fullscreen: value.force_fullscreen,
		game_resolution: [...value.game_resolution] as [number, number],
	}),
	(value) => {
		if (!value.game_resolution.every((dimension) => Number.isInteger(dimension) && dimension > 0)) {
			return null
		}
		return {
			force_fullscreen: value.force_fullscreen,
			game_resolution: [...value.game_resolution],
		}
	},
)

const messages = defineMessages({
	fullscreenTitle: {
		id: 'app.settings.default-instance-options.fullscreen.title',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'app.settings.default-instance-options.fullscreen.description',
		defaultMessage: 'Start instances in fullscreen by updating their options.txt file.',
	},
	widthTitle: {
		id: 'app.settings.default-instance-options.width.title',
		defaultMessage: 'Width',
	},
	widthDescription: {
		id: 'app.settings.default-instance-options.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	widthPlaceholder: {
		id: 'app.settings.default-instance-options.width.placeholder',
		defaultMessage: 'Enter width...',
	},
	heightTitle: {
		id: 'app.settings.default-instance-options.height.title',
		defaultMessage: 'Height',
	},
	heightDescription: {
		id: 'app.settings.default-instance-options.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	heightPlaceholder: {
		id: 'app.settings.default-instance-options.height.placeholder',
		defaultMessage: 'Enter height...',
	},
	mesa3dPreset: {
		id: 'app.settings.default-instance-options.mesa3d-preset',
		defaultMessage: 'Mesa3D Performance Presets',
	},
	mesa3dPresetDesc: {
		id: 'app.settings.default-instance-options.mesa3d-preset.desc',
		defaultMessage: 'Quick resolution presets optimized for CPU rendering when using Mesa3D software rasterization.',
	},
})

function applyMesaResolution(w: number, h: number) {
	if (!settings.value) return
	settings.value.force_fullscreen = false
	settings.value.game_resolution = [w, h]
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<template v-if="settings">
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.fullscreenTitle) }}
					</h2>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.fullscreenDescription) }}
					</p>
				</div>
				<Toggle
					id="fullscreen"
					v-model="settings.force_fullscreen"
					:aria-label="formatMessage(messages.fullscreenTitle)"
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.widthTitle) }}
					</h2>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.widthDescription) }}
					</p>
				</div>
				<Input
					id="width"
					v-model="settings.game_resolution[0]"
					:aria-label="formatMessage(messages.widthTitle)"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					:placeholder="formatMessage(messages.widthPlaceholder)"
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.heightTitle) }}
					</h2>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.heightDescription) }}
					</p>
				</div>
				<Input
					id="height"
					v-model="settings.game_resolution[1]"
					:aria-label="formatMessage(messages.heightTitle)"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					:placeholder="formatMessage(messages.heightPlaceholder)"
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
						:class="{ '!border-brand !bg-brand/20 font-semibold text-brand': settings.game_resolution[0] === 854 && settings.game_resolution[1] === 480 && !settings.force_fullscreen }"
						@click="applyMesaResolution(854, 480)"
					>
						480p (854 × 480) • Best FPS
					</button>
					<button
						type="button"
						class="px-2.5 py-1 text-xs rounded border border-surface-5 bg-surface-3 hover:bg-surface-4 text-contrast cursor-pointer transition-colors"
						:class="{ '!border-brand !bg-brand/20 font-semibold text-brand': settings.game_resolution[0] === 1024 && settings.game_resolution[1] === 768 && !settings.force_fullscreen }"
						@click="applyMesaResolution(1024, 768)"
					>
						768p (1024 × 768) • 4:3 Balanced
					</button>
					<button
						type="button"
						class="px-2.5 py-1 text-xs rounded border border-surface-5 bg-surface-3 hover:bg-surface-4 text-contrast cursor-pointer transition-colors"
						:class="{ '!border-brand !bg-brand/20 font-semibold text-brand': settings.game_resolution[0] === 1280 && settings.game_resolution[1] === 720 && !settings.force_fullscreen }"
						@click="applyMesaResolution(1280, 720)"
					>
						720p (1280 × 720) • HD Standard
					</button>
				</div>
			</div>
		</template>
		<Button v-else-if="settingsQuery.isError.value" @click="settingsQuery.refetch()">
			{{ formatMessage(commonMessages.refreshButton) }}
		</Button>
		<div v-else class="flex items-center gap-2 text-secondary">
			<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
			{{ formatMessage(commonMessages.loadingLabel) }}
		</div>
	</div>
</template>
