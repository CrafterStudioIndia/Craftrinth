<template>
	<Transition name="splash-fade" @after-leave="onAfterLeave">
		<div v-if="!doneLoading" class="splash-screen" :class="`${theme.active}-mode`">
			<div class="app-logo-wrapper" data-tauri-drag-region>
				<div class="flex flex-col items-center gap-3 justify-center mb-4 relative">
					<div class="absolute -inset-6 bg-gradient-to-r from-blue-600 via-sky-500 to-cyan-400 blur-2xl opacity-40 -z-10"></div>
					<img :src="LogoImage" alt="Craftrinth" class="w-28 h-28 object-contain drop-shadow-[0_0_24px_rgba(56,189,248,0.6)]" />
					<span class="text-3xl font-black bg-gradient-to-r from-white via-slate-100 to-sky-300 bg-clip-text text-transparent tracking-wider">Craftrinth</span>
				</div>
				<ProgressBar class="loading-bar" :progress="Math.min(loadingProgress, 100)" />
				<span v-if="message">{{ message }}</span>
			</div>
			<div class="gradient-bg" data-tauri-drag-region></div>
			<div class="glow-bg"></div>
			<div class="base-bg"></div>
		</div>
	</Transition>
</template>

<script setup>
import { injectLoadingState } from '@modrinth/ui'
import { onMounted, ref, watch } from 'vue'

import LogoImage from '@/assets/logo.png'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useTheme } from '@/composables/use-theme.ts'
import { debugStartup } from '@/helpers/startup-debug'

const props = defineProps({
	ready: {
		type: Boolean,
		default: false,
	},
})

const theme = useTheme()

const doneLoading = ref(false)
const loadingProgress = ref(0)
const message = ref()

const MIN_DISPLAY_MS = 600
const mountedAt = Date.now()

const loading = injectLoadingState()
let dismissalScheduled = false

function onAfterLeave() {
	debugStartup('Splash fade completed', { displayedMs: Date.now() - mountedAt })
	loading.setEnabled(true)
}

function finishAndDismiss() {
	if (doneLoading.value) return
	loadingProgress.value = 100
	setTimeout(() => {
		doneLoading.value = true
		debugStartup('Splash fade started', { displayedMs: Date.now() - mountedAt })
	}, 150)
}

function triggerDismissal() {
	if (doneLoading.value || dismissalScheduled) return
	dismissalScheduled = true
	const elapsed = Date.now() - mountedAt
	const delay = Math.max(0, MIN_DISPLAY_MS - elapsed)
	debugStartup('Splash dismissal scheduled', { delayMs: delay, displayedMs: elapsed })
	setTimeout(() => {
		finishAndDismiss()
	}, delay)
}

onMounted(() => {
	debugStartup('Splash mounted')
	fakeLoadingIncrease()

	// Hard safety timeout: Ensure the splash screen NEVER hangs the application indefinitely
	setTimeout(() => {
		if (!doneLoading.value) {
			debugStartup('Splash dismissed via fallback safety timeout')
			finishAndDismiss()
		}
	}, 1800)
})

watch(
	() => props.ready,
	(isReady) => {
		if (isReady) {
			debugStartup('Splash received ready signal')
			triggerDismissal()
		}
	},
	{ immediate: true },
)

watch(
	[loading.barEnabled, loading.pending],
	([barEnabled, pending]) => {
		debugStartup('Splash loading state changed', { barEnabled, pending })
		if (barEnabled) {
			return
		}

		if (!pending) {
			triggerDismissal()
		}
	},
	{ immediate: true },
)

function fakeLoadingIncrease() {
	if (loadingProgress.value < 90 && !doneLoading.value) {
		setTimeout(() => {
			loadingProgress.value += 3
			fakeLoadingIncrease()
		}, 15)
	}
}

useAppEvent('loading', (e) => {
	if (e.event.type === 'directory_move') {
		loadingProgress.value = 100 * (e.fraction ?? 1)
		message.value = 'Updating app directory...'
	}
})
</script>

<style scoped lang="scss">
.splash-screen {
	position: fixed;
	inset: 0;
	z-index: 10000;
	background: #070a13;

	--splash-tint-top: rgba(37, 99, 235, 0.28);
	--splash-tint-bottom: rgba(7, 10, 19, 0.96);
	--splash-overlay: rgba(7, 10, 19, 0.4);

	&.light-mode {
		--splash-tint-top: rgba(219, 234, 254, 0.75);
		--splash-tint-bottom: rgba(239, 246, 255, 0.95);
		--splash-overlay: rgba(191, 219, 254, 0.35);
	}
}

.splash-fade-leave-active {
	transition: opacity 0.3s ease-in-out;
}

.splash-fade-leave-to {
	opacity: 0;
}

.app-logo-wrapper {
	position: absolute;
	height: 100vh;
	width: 100%;

	display: flex;
	flex-direction: column;
	justify-content: center;
	align-items: center;

	gap: 1.25rem;
	color: #ffffff;

	z-index: 9998;
}

.app-logo {
	height: 2.25rem;
	width: fit-content;
}

.loading-bar {
	max-width: 22rem;
}

.gradient-bg {
	position: absolute;
	height: 100vh;
	width: 100vw;
	background:
		radial-gradient(ellipse 75% 65% at 50% 0%, rgba(56, 189, 248, 0.25) 0%, transparent 65%),
		radial-gradient(ellipse 60% 60% at 10% 90%, rgba(37, 99, 235, 0.2) 0%, transparent 70%),
		radial-gradient(ellipse 50% 50% at 90% 85%, rgba(14, 165, 233, 0.15) 0%, transparent 65%),
		linear-gradient(180deg, var(--splash-tint-top) 0%, var(--splash-tint-bottom) 97%),
		linear-gradient(0deg, var(--splash-overlay), var(--splash-overlay));
	z-index: 9997;
}

.glow-bg {
	position: absolute;
	inset: 0;
	background:
		radial-gradient(circle at 50% 45%, rgba(56, 189, 248, 0.16) 0%, transparent 50%),
		radial-gradient(circle at 80% 20%, rgba(14, 165, 233, 0.12) 0%, transparent 40%);
	z-index: 9996;
	pointer-events: none;
}

.base-bg {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background: #070a13;
	z-index: 9995;
}
</style>
