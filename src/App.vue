<script setup lang="ts">
// UI imports
import RippleButton from "@/ui/RippleButton.vue";

// Library imports
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

// Environment
const env_result = ref<string | null>(null);
async function environment() {
  env_result.value = await invoke("configure_environment");
}

// Ping
const ping = ref<string | null>(null);
const isPinging = ref(false);
const dots = ref(".");

let dotsTimer: number | null = null;

function startDots() {
  dots.value = ".";
  dotsTimer = window.setInterval(() => {
    dots.value = dots.value.length >= 3 ? "." : dots.value + ".";
  }, 250);
}

function stopDots() {
  if (dotsTimer !== null) {
    clearInterval(dotsTimer);
    dotsTimer = null;
  }
}

async function ping_server(ip: string) {
  isPinging.value = true;
  ping.value = null;

  startDots();

  try {
    ping.value = await invoke("ping", { ip });
  } finally {
    isPinging.value = false;
    stopDots();
  }
}

// 

</script>

<template>
  <main class="container">
    <h1>MidnightBox <br /> MiniBox <br /> CatinBox</h1>

    <a>
      <img src="/vite.svg" class="logo vite" alt="Vite logo" />
    </a>

    <div class="grid place-content-center p-8">
      <RippleButton @click="environment">
        Grab Environment Variables
      </RippleButton>
    </div>

    <p>{{ env_result }}</p>

    <div class="grid place-content-center p-8">
      <RippleButton
        @click="ping_server('0.0.0.0')"
        :disabled="isPinging"
      >
        Ping Server
      </RippleButton>
    </div>

    <p v-if="isPinging">
      Pinging{{ dots }}
    </p>

    <p v-else>
      {{ ping }}
    </p>
  </main>
</template>

<style>
:root {
  font-family: Hack, Arial;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
  text-align: center;
}

.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

button {
  cursor: pointer;
}

</style>