<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon, RightArrowIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { useRouter } from 'vue-router'

import LogoImage from '@/assets/logo.png'
import AppActionBar from './action-bar.vue'
import Breadcrumbs from './breadcrumbs.vue'
import WindowControls from './window-controls.vue'

defineProps<{
	canNavigateBack: boolean
	canNavigateForward: boolean
	showSidebarToggle: boolean
	sidebarToggled: boolean
}>()

const emit = defineEmits<{
	'toggle-sidebar': []
}>()

const router = useRouter()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	goBack: { id: 'app.navigation.go-back', defaultMessage: 'Go back' },
	goForward: { id: 'app.navigation.go-forward', defaultMessage: 'Go forward' },
	nextImage: { id: 'app.navigation.next-image', defaultMessage: 'Next image' },
})
</script>

<template>
	<div
		data-tauri-drag-region
		class="app-title-bar relative z-[2] flex items-center bg-bg-raised pl-2 pr-1 [grid-area:status] [.mac-traffic-lights_&]:pl-20"
	>
		<div data-tauri-drag-region class="flex items-center gap-2 shrink-0 py-1">
			<div class="flex items-center gap-2 shrink-0 select-none pointer-events-none mr-2">
				<img :src="LogoImage" alt="Craftrinth" class="h-6 w-6 shrink-0 object-contain drop-shadow-[0_0_10px_rgba(56,189,248,0.5)]" />
				<span class="font-black tracking-tight text-base bg-gradient-to-r from-white via-blue-100 to-sky-400 bg-clip-text text-transparent font-display">Craftrinth</span>
			</div>
			<div data-tauri-drag-region-exclude class="flex shrink-0 items-center gap-1">
				<IconButton
					type="outlined"
					:label="formatMessage(messages.goBack)"
					class="!h-7 !min-w-7 !w-7 !border !border-surface-4 !p-0 !opacity-100"
					:disabled="!canNavigateBack"
					@click="router.back()"
				>
					<ChevronLeftIcon
						class="!size-4 !text-primary"
						:class="{ 'opacity-20': !canNavigateBack }"
					/>
				</IconButton>
				<IconButton
					type="outlined"
					:label="formatMessage(messages.goForward)"
					class="!h-7 !min-w-7 !w-7 !border !border-surface-4 !p-0 !opacity-100"
					:disabled="!canNavigateForward"
					@click="router.forward()"
				>
					<ChevronRightIcon
						class="!size-4 !text-primary"
						:class="{ 'opacity-20': !canNavigateForward }"
					/>
				</IconButton>
			</div>
		</div>

		<div data-tauri-drag-region-exclude class="flex items-center gap-1 mx-2 shrink-0">
			<slot name="navigation" />
		</div>

		<div data-tauri-drag-region class="flex min-w-0 flex-1 items-center overflow-hidden px-2">
			<Breadcrumbs />
		</div>

		<section data-tauri-drag-region-exclude class="flex shrink-0 ml-auto items-center gap-2">
			<div data-tauri-drag-region-exclude class="flex items-center gap-1">
				<slot name="actions" />
			</div>
			<IconButton
				v-if="showSidebarToggle"
				:type="sidebarToggled ? 'base' : 'quiet'"
				:label="formatMessage(messages.nextImage)"
				class="transition-transform"
				:class="{ 'rotate-180': sidebarToggled }"
				@click="emit('toggle-sidebar')"
			>
				<RightArrowIcon />
			</IconButton>
			<div class="flex">
				<Suspense>
					<AppActionBar />
				</Suspense>
			</div>
			<WindowControls />
		</section>
	</div>
</template>

<style scoped>
.app-title-bar {
	height: var(--top-bar-height);
	padding-right: var(--window-controls-width, 0px);
}
</style>
