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
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir, join } from "@tauri-apps/api/path";

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
const show_modal = ref(false);
const save_status = ref("");

const form = reactive({
  name: "",
  ip: "",
  port: "",
  dns: "",
  protocol: "",
  cmd: ""
});

const proxy_types = [
  "socks5",
  "socks4",
  "http",
  "shadowsocks",
  "relay",
  "direct",
  "reject"
];

async function saveProfile() {
  save_status.value = "Saving...";
  try {
    const result = await invoke("save_profile", {
      name: form.name,
      ip: form.ip,
      port: form.port,
      dns: form.dns,
      protocol: form.protocol,
      cmd: form.cmd
    });
    save_status.value = result as string;
  } 
  catch (error) {
    save_status.value = `Error: ${error}`;
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
  "main.py",
  "flatpak run org.mozilla.firefox",
  "steam",
];

// Profile selection
const profile_path = ref<string | null>(null);

async function select_profile() {
  const app_data = await appDataDir();
  const selected = await open({
    defaultPath: await join(app_data, "profiles"),
    multiple: false,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });

  if (selected && typeof selected === "string") {
    profile_path.value = selected;
  }
}

// Namespace creating
const ns_result = ref<string | null>(null);

async function setup_namespace() {
  if (!profile_path.value || !profile_path.value.endsWith(".json")) {
    ns_result.value = "No valid profile selected";
    return;
  }
  try {
    ns_result.value = await invoke("setup_namespace", {
      profilePath: profile_path.value,
    });
  } 
  catch (error) {
    ns_result.value = `Error creating namespace: ${error}`;
  }
}

// Running
const run_result = ref<string | null>(null);

async function run() {
  if (!profile_path.value) {
    run_result.value = "No valid profile selected";
    return;
  }
  try {
    run_result.value = await invoke("run", {
      profilePath: profile_path.value,
    });
  } 
  catch (error) {
    run_result.value = `Error running: ${error}`;
  }
}

// Cleanup
const cleanup_result = ref<string | null>(null);

async function cleanup() {
  if (!profile_path.value) {
    cleanup_result.value = "No valid profile selected";
    return;
  }
  try {
    const app_data = await appDataDir();
    const pid_file = await join(app_data, "pid.json");
    cleanup_result.value = await invoke("cleanup", {
      profilePath: profile_path.value,
      pidPath: pid_file.toString(),
    });
  } 
  catch (error) {
    cleanup_result.value = `Error cleaning up: ${error}`;
  }
}
</script>

<template>
  <main class="container">
    
    <a class="grid place-content-center p-5">
      <img src="/vite.svg" class="logo vite" alt="Vite logo" />
    </a>

    <div class="grid place-content-center p-5">
      <RippleButton @click="select_profile">1.Select profile config</RippleButton>
    </div>

    <p>{{ profile_path }}</p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="setup_namespace"> 2.Create Namespace </RippleButton>
    </div>

    <p>{{ ns_result }}</p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="run"> 3.Run </RippleButton>
    </div>

    <p>{{ run_result }}</p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="cleanup"> 4.Cleanup </RippleButton>
    </div>

    <p>{{ cleanup_result }}</p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="ping_server(form.ip)" :disabled="is_pinging"> Ping Server </RippleButton>
    </div>

    <p v-if="is_pinging">
      Pinging{{ dots }}
    </p>

    <p v-else>
      {{ ping }}
    </p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="show_modal = true" class="bg-blue-600"> + Create Profile </RippleButton>
    </div>

    <div v-if="show_modal" class="modal-overlay">
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
            <DropdownMenuItem v-for="type in proxy_types" :key="type" @click="form.protocol = type">
              {{ type.toUpperCase() }}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <div class="form-group p-5">
          <label>Command</label>
          <VanishingInput v-model="form.cmd" :placeholders="cmd_placeholders" />
        </div>

        <div class="modal-actions grid place-content-center p-5">
          <RippleButton @click="show_modal = false">Close</RippleButton>
          <RippleButton @click="saveProfile">Save</RippleButton>
        </div>

        <p class="status-text">{{ save_status }}</p>
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