<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from "vue";

// Define interfaces for props and data structures
interface Props {
	placeholders?: string[];
}
// props
const props = withDefaults(defineProps<Props>(), {
	placeholders: () => ["Placeholder 1", "Placeholder 2", "Placeholder 3"],
});
const emit = defineEmits(["submit", "change"]);
const vanishingText = defineModel<string>({
	default: "",
});
const inputRef = useTemplateRef<HTMLInputElement>("inputRef");

// normal refs
const currentPlaceholder = ref<number>(0);
const animating = ref<boolean>(false);
const intervalRef = ref<number | null>(null);
const animationFrame = ref<number | null>(null);

// Focus on input when mounted
onMounted(() => {
	if (!inputRef.value) return;
	inputRef.value.focus();
});

function changePlaceholder(): void {
	intervalRef.value = window.setInterval(() => {
		currentPlaceholder.value =
			(currentPlaceholder.value + 1) % props.placeholders.length;
	}, 3000);
}

function handleVisibilityChange(): void {
	if (document.visibilityState !== "visible" && intervalRef.value) {
		clearInterval(intervalRef.value);
		intervalRef.value = null;
	} else if (document.visibilityState === "visible") {
		changePlaceholder();
	}
}

// Watch for value changes
watch(vanishingText, (newVal: string) => {
	if (!animating.value) {
		emit("change", { target: { value: newVal } });
	}
});

onMounted(() => {
	changePlaceholder();
	document.addEventListener("visibilitychange", handleVisibilityChange);
});

onBeforeUnmount(() => {
	if (intervalRef.value) {
		clearInterval(intervalRef.value);
	}
	if (animationFrame.value) {
		cancelAnimationFrame(animationFrame.value);
	}
	document.removeEventListener("visibilitychange", handleVisibilityChange);
});
</script>

<template>
  <form
    class="relative mx-auto h-12 w-full max-w-xl overflow-hidden rounded-full bg-white shadow-[0px_2px_3px_-1px_rgba(0,0,0,0.1),_0px_1px_0px_0px_rgba(25,28,33,0.02),_0px_0px_0px_1px_rgba(25,28,33,0.08)] transition duration-200 dark:bg-zinc-800"
    :class="[vanishingText && 'bg-gray-50']"
  >
    <!-- Canvas Element -->
    <canvas
      ref="canvasRef"
      class="pointer-events-none absolute top-[20%] left-2 origin-top-left scale-50 pr-20 text-base invert sm:left-8 dark:invert-0"
      :class="[animating ? 'opacity-100' : 'opacity-0']"
    />

    <!-- Text Input -->
    <input
      ref="inputRef"
      v-model="vanishingText"
      :disabled="animating"
      type="text"
      class="relative z-50 size-full rounded-full border-none bg-transparent pr-10 pl-4 text-sm text-black focus:ring-0 focus:outline-none sm:pl-10 sm:text-base dark:text-white"
      :class="{ 'text-transparent dark:text-transparent': animating }"
    />

    <!-- Placeholder Text -->
    <div class="pointer-events-none absolute inset-0 flex items-center rounded-full">
      <Transition
        v-show="!vanishingText"
        mode="out-in"
        enter-active-class="transition duration-300 ease-out"
        leave-active-class="transition duration-300 ease-in"
        enter-from-class="opacity-0 translate-y-4"
        enter-to-class="opacity-100 translate-y-0"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-4"
      >
        <p
          :key="currentPlaceholder"
          class="w-[calc(100%-2rem)] truncate pl-4 text-left text-sm font-normal text-neutral-500 sm:pl-10 sm:text-base dark:text-zinc-500"
        >
          {{ placeholders[currentPlaceholder] }}
        </p>
      </Transition>
    </div>
  </form>
</template>

