<template>
	<component
		:is="isLink ? RouterLink : 'button'"
		:to="isLink ? to : undefined"
		v-bind="$attrs"
		data-tauri-drag-region-exclude
		:type="isLink ? undefined : 'button'"
		:disabled="isLink ? undefined : disabled"
		:active-class="isLink && isSubpage ? '' : undefined"
		:class="{
			'router-link-active': isPrimary && isPrimary(route),
			'subpage-active': isSubpage && isSubpage(route),
			disabled: disabled,
			'w-12 rounded-full h-12 text-2xl': !topbar,
			'topbar-nav-button': topbar,
		}"
		class="nav-button border-none text-primary cursor-pointer flex items-center justify-center transition-all bg-transparent hover:bg-button-bg hover:text-contrast"
		@click="onClick"
	>
		<slot />
	</component>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { RouteLocationNormalizedLoaded } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

type RouteFunction = (route: RouteLocationNormalizedLoaded) => boolean

const props = withDefaults(
	defineProps<{
		to: (() => void) | string
		isPrimary?: RouteFunction
		isSubpage?: RouteFunction
		highlightOverride?: boolean
		disabled?: boolean
		topbar?: boolean
	}>(),
	{
		disabled: false,
		topbar: false,
	},
)

const isLink = computed(() => typeof props.to === 'string')

function onClick(event: MouseEvent) {
	if (typeof props.to === 'function') {
		event.preventDefault()
		event.stopPropagation()
		props.to()
	}
}

defineOptions({
	inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.nav-button {
	position: relative;

	&:not(.topbar-nav-button)::before {
		content: '';
		position: absolute;
		inset: 0;
		background-color: var(--color-button-bg-selected);
		border-radius: 50%;
		opacity: 0;
		scale: 0.4;
		z-index: -1;
		transition:
			opacity 0.25s var(--ease-out-expo),
			scale 0.25s var(--ease-out-expo);
	}
}

.router-link-active,
.subpage-active {
	svg {
		filter: drop-shadow(0 0 0.5rem black);
	}
}

.router-link-active {
	@apply text-[--color-button-text-selected];
	&:not(.topbar-nav-button)::before {
		background-color: var(--color-button-bg-selected);
		scale: 1;
		opacity: 1;
	}
}

.subpage-active {
	@apply text-contrast bg-button-bg;
}
</style>
