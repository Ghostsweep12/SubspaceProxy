<script setup lang="ts">
// UI imports
import RippleButton from "@/components/ui/RippleButton.vue";
import VanishingInput from "@/components/ui/VanishingInput.vue";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

// Library imports
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";

// Environment
const env_result = ref<string | null>(null);
async function environment() {
  try {
    env_result.value = await invoke("configure_environment");
  } catch (error) {
    env_result.value = `Error: ${error}`;
  }
}

// Ping
const ping = ref<string | null>(null);
const is_pinging = ref(false);
const dots = ref(".");
let dots_timer: number | null = null;

function startDots() {
  dots.value = ".";
  dots_timer = window.setInterval(() => {
    dots.value = dots.value.length >= 3 ? "." : dots.value + ".";
  }, 250);
}

function stopDots() {
  if (dots_timer !== null) {
    clearInterval(dots_timer);
    dots_timer = null;
  }
}

async function ping_server(ip: string) {
  is_pinging.value = true;
  ping.value = null;

  startDots();

  try {
    ping.value = await invoke("ping", { ip });
  } 
  catch (error) {
    ping.value = `Error: ${error}`;
  } 
  finally {
    is_pinging.value = false;
    stopDots();
  }
}

// Profile making
const showModal = ref(false);
const saveStatus = ref("");

const form = reactive({
  name: "",
  ip: "",
  port: "",
  dns: "",
  protocol: "",
  cmd: ""
});

const proxyTypes = [
  "socks5",
  "socks4",
  "http",
  "shadowsocks",
  "relay",
  "direct",
  "reject"
];

async function saveProfile() {
  saveStatus.value = "Saving...";
  try {
    const result = await invoke("save_profile", {
      name: form.name,
      ip: form.ip,
      port: form.port,
      dns: form.dns,
      protocol: form.protocol,
      cmd: form.cmd
    });
    saveStatus.value = result as string;
  } 
  catch (error) {
    saveStatus.value = `Error: ${error}`;
  }
}

const name_placeholders = [
  "Proxy1",
  "Server SS Port",
  "Server Sock5 Port",
  "MyVPN",
];

const ip_placeholders = [
  "192.168.1.1",
  "fe80::xxxx:xxxx:xxxx:xxxx",
];

const port_placeholders = [
  "8080",
  "443",
  "1088",
];

const dns_placeholders = [
  "8.8.8.8",
  "2001:4860:4860::8888",
  "1.1.1.1",
  "2606:4700:4700::1111",
];

const cmd_placeholders = [
  "bash",
  "flatpak run org.mozilla.firefox",
  "hyprland",
];

//

</script>

<template>
  <main class="container">
    
    <a class="grid place-content-center p-5">
      <img src="/vite.svg" class="logo vite" alt="Vite logo" />
    </a>

    <div class="grid place-content-center p-5">
      <RippleButton @click="environment"> Grab Environment Variables </RippleButton>
    </div>

    <p>{{ env_result }}</p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="ping_server('0.0.0.0')" :disabled="is_pinging"> Ping Server </RippleButton>
    </div>

    <p v-if="is_pinging">
      Pinging{{ dots }}
    </p>

    <p v-else>
      {{ ping }}
    </p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="showModal = true" class="bg-blue-600"> + Create Profile </RippleButton>
    </div>

    <div v-if="showModal" class="modal-overlay">
      <div class="modal-content">
        <h2>New Proxy Profile</h2>

        <div class="form-group p-5">
          <label>Profile Name</label>
          <VanishingInput v-model="form.name" :placeholders="name_placeholders" />
        </div>

          <div class="form-group p-5">
            <label>IP Address</label>
            <VanishingInput v-model="form.ip" :placeholders="ip_placeholders" />
          </div>

          <div class="form-group p-5">
            <label>Port</label>
            <VanishingInput v-model="form.port" :placeholders="port_placeholders" />
          </div>

          <div class="form-group p-5">
            <label>DNS</label>
            <VanishingInput v-model="form.dns" :placeholders="dns_placeholders" />
          </div>

        <DropdownMenu class="form-group p-10">
          <DropdownMenuTrigger>
            <RippleButton>
              {{ form.protocol ? form.protocol.toUpperCase() : 'Protocol' }}
            </RippleButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem v-for="type in proxyTypes" :key="type" @click="form.protocol = type">
              {{ type.toUpperCase() }}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <div class="form-group p-5">
          <label>Command</label>
          <VanishingInput v-model="form.cmd" :placeholders="cmd_placeholders" />
        </div>

        <div class="modal-actions grid place-content-center p-5">
          <RippleButton @click="showModal = false">Close</RippleButton>
          <RippleButton @click="saveProfile">Save</RippleButton>
        </div>

        <p class="status-text">{{ saveStatus }}</p>
      </div>
    </div>
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

</style>