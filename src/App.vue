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
const show_advanced_modal = ref(false);
const save_status = ref("");

type Profile = {
    ip?: string,
    port?: string,
    protocol?: string,
    namespace?: string,
    username?: string,
    password?: string,
    tun_interface?: string,
    tun_ip?: string,
    veth_host?: string,
    veth_ns?: string,
    veth_host_ip?: string,
    veth_ns_ip?: string,
    dns?: string,
}

const form = reactive({
  name: "",
  ip: "",
  port: "",
  protocol: "",
  cmd: "",
  dns: "",
  namespace: "",
  username: "",
  password: "",
  tun_interface: "",
  tun_ip: "",
  veth_host: "",
  veth_ns: "",
  veth_host_ip: "",
  veth_ns_ip: "",
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
      protocol: form.protocol,
      dns: form.dns,
      namespace: form.namespace,
      username: form.username,
      password: form.password,
      tunInterface: form.tun_interface,
      tunIp: form.tun_ip,
      vethHost: form.veth_host,
      vethNs: form.veth_ns,
      vethHostIp: form.veth_host_ip,
      vethNsIp: form.veth_ns_ip,
    });
    save_status.value = result as string;
  } 
  catch (error) {
    save_status.value = `Error: ${error}`;
  }
}

const name_placeholders = [
  "_ (default)",
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
  "8.8.8.8 (default)",
  "2001:4860:4860::8888",
  "1.1.1.1",
  "2606:4700:4700::1111",
];

const namespace_placeholders = [
  "namespace (default)",
  "default",
  "custom_ns",
  "proxyns"
];

const username_placeholders = [
  "( ) (default)",
  "user123",
  "1001",
  "rc4-md5",
];

const password_placeholders = [
  "( ) (default)",
];

const tun_interface_placeholders = [
  "tun0 (default)",
];

const tun_ip_placeholders = [
  "10.0.0.2 (default)",
];

const veth_host_placeholders = [
  "veth_host (default)",
];

const veth_ns_placeholders = [
  "veth_ns (default)",
];

const veth_host_ip_placeholders = [
  "10.200.1.1 (default)",
];

const veth_ns_ip_placeholders = [
  "10.200.1.2 (default)",
];

const cmd_placeholders = [
  "main.py",
  "flatpak run org.mozilla.firefox",
  "steam",
];

// Profile selection
const profile_path = ref<string | null>(null);
const selected_profile = ref<Profile | null>(null);

async function select_profile() {
  const app_data = await appDataDir();
  const selected_path  = await open({
    defaultPath: await join(app_data, "profiles"),
    multiple: false,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });

  if (selected_path && typeof selected_path === "string") {
    profile_path.value = selected_path;
  }
  try {
    selected_profile.value = await invoke<Profile>(
      "fetch_profile",
      { profilePath: profile_path.value }
    );
  } 
  catch (e) {
    selected_profile.value = null;
    console.error("Failed to load profile:", e);
  }
}

// Namespace creating
const ns_result = ref<string | null>(null);

async function setup_namespace() {
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
const cmd = ref<string | undefined>(undefined);

async function run() {
  try {
    run_result.value = await invoke("run", {
      profilePath: profile_path.value,
      cmd: cmd.value,
    });
  } 
  catch (error) {
    run_result.value = `Error running: ${error}`;
  }
}

// Cleanup
const cleanup_result = ref<string | null>(null);

async function cleanup() {
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
      <RippleButton @click="setup_namespace" :disabled="!selected_profile">2.Setup Namespace</RippleButton>
    </div>

    <p>{{ ns_result }}</p>

    <div class="form-group p-5">
      <label>Command</label>
      <VanishingInput v-model="cmd" :placeholders="cmd_placeholders" />
    </div>

    <div class="grid place-content-center p-5">
      <RippleButton @click="run" :disabled="!selected_profile || !cmd"> 3.Run </RippleButton>
    </div>

    <p>{{ run_result }}</p>

    <div class="grid place-content-center p-5">
      <RippleButton @click="cleanup" :disabled="!selected_profile"> 4.Cleanup </RippleButton>
    </div>

    <p>{{ cleanup_result }}</p>
    
    <div class="grid place-content-center p-5">
      <RippleButton @click="ping_server(selected_profile?.ip ?? '')" :disabled="is_pinging || !selected_profile?.ip"> Ping Server </RippleButton>
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

        <div class="grid place-content-center p-5">
          <RippleButton @click="show_advanced_modal = true" class="bg-blue-600"> + Advanced Settings </RippleButton>
        </div>

        <div v-if="show_advanced_modal" class="modal-overlay">
          <div class="modal-content">
            <h2>Advanced Settings</h2>

            <div class="form-group p-5">
              <label>DNS</label>
              <VanishingInput v-model="form.dns" :placeholders="dns_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Namespace Name</label>
              <VanishingInput v-model="form.namespace" :placeholders="namespace_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Username/UID/Method</label>
              <VanishingInput v-model="form.username" :placeholders="username_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Password</label>
              <VanishingInput v-model="form.password" :placeholders="password_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Tun Interface</label>
              <VanishingInput v-model="form.tun_interface" :placeholders="tun_interface_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Tun IP</label>
              <VanishingInput v-model="form.tun_ip" :placeholders="tun_ip_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Veth Host</label>
              <VanishingInput v-model="form.veth_host" :placeholders="veth_host_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Veth Namespace</label>
              <VanishingInput v-model="form.veth_ns" :placeholders="veth_ns_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Veth Host IP</label>
              <VanishingInput v-model="form.veth_host_ip" :placeholders="veth_host_ip_placeholders" />
            </div>

            <div class="form-group p-5">
              <label>Veth Namespace IP</label>
              <VanishingInput v-model="form.veth_ns_ip" :placeholders="veth_ns_ip_placeholders" />
            </div>

            <div class="modal-actions grid place-content-center p-5">
              <RippleButton @click="show_advanced_modal = false">Close Advanced</RippleButton>
            </div>

          </div>
        </div>

        <div class="modal-actions grid place-content-center p-5">
          <RippleButton @click="show_modal = false">Close</RippleButton>
          <RippleButton @click="saveProfile">Save</RippleButton>
        </div>

        <p>{{ save_status }}</p>

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