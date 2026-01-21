<script setup lang="ts">
// UI imports

// Library imports
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted, reactive, ref } from "vue";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import RippleButton from "@/components/ui/RippleButton.vue";
import VanishingInput from "@/components/ui/VanishingInput.vue";

// Profile making
const show_modal = ref(false);
const show_advanced_modal = ref(false);
const save_status = ref("");
const editing_path = ref<string | null>(null);
const form_errors = reactive({
	name: false,
	ip: false,
	port: false,
	protocol: false,
});

type profile = {
	ip?: string;
	port?: string;
	protocol?: string;
	namespace?: string;
	username?: string;
	password?: string;
	tun_interface?: string;
	tun_ip?: string;
	veth_host?: string;
	veth_ns?: string;
	veth_host_ip?: string;
	veth_ns_ip?: string;
	dns?: string;
};

type profile_entry = {
	filename: string;
	path: string;
	profile: profile;
	ping_status: string;
	port_status: string;
	ns_status: string;
	run_status: string;
	clean_status: string;
	is_pinging: boolean;
	is_port_checking: boolean;
	is_setting_up: boolean;
	is_running: boolean;
	is_cleaning: boolean;
};

type namespace_info = {
	name: string;
	processes: string;
};

const form = reactive({
	name: "",
	ip: "",
	port: "",
	protocol: "",
	cmd: "",
	dns: "8.8.8.8",
	namespace: "namespace",
	username: "",
	password: "",
	tun_interface: "",
	tun_ip: "",
	veth_host: "veth_host",
	veth_ns: "veth_ns",
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
	"reject",
];

// Random number for interfaces
function apply_random_network_values() {
	const r = Math.floor(Math.random() * 253) + 1;
	const r2 = Math.floor(Math.random() * 253) + 1;

	form.tun_interface = `tun${r}`;
	form.tun_ip = `10.${r}.${r2}.2`;
	form.veth_host_ip = `10.200.${r}.1`;
	form.veth_ns_ip = `10.200.${r}.2`;
}

// New profile
function new_profile() {
	reset_form();
	apply_random_network_values();
	editing_path.value = null;
	show_modal.value = true;
}

// Edit Profile
function edit_profile(index: number) {
	const p = profiles.value[index];

	form.name = p.filename.replace(/_/g, " ");
	form.ip = p.profile.ip || "";
	form.port = p.profile.port || "";
	form.protocol = p.profile.protocol || "";
	form.dns = p.profile.dns || "8.8.8.8";
	form.namespace = p.profile.namespace || "namespace";
	form.username = p.profile.username || "";
	form.password = p.profile.password || "";
	form.tun_interface = p.profile.tun_interface || "";
	form.tun_ip = p.profile.tun_ip || "";
	form.veth_host = p.profile.veth_host || "veth_host";
	form.veth_ns = p.profile.veth_ns || "veth_ns";
	form.veth_host_ip = p.profile.veth_host_ip || "";
	form.veth_ns_ip = p.profile.veth_ns_ip || "";

	editing_path.value = p.path;
	show_modal.value = true;
}

// Delete Profile
async function delete_profile_confirm(index: number) {
	const p = profiles.value[index];
	if (
		await confirm(`Are you sure you want to delete profile "${p.filename}"?`)
	) {
		try {
			await invoke("delete_profile", { path: p.path });
			load_profiles();
		} catch (e) {
			alert("Failed to delete: " + e);
		}
	}
}

// Reset Form
function reset_form() {
	form.name = "";
	form.ip = "";
	form.port = "";
	form.protocol = "";
	form.username = "";
	form.password = "";
}

// Save Profile
async function save_profile() {
	form_errors.name = !form.name;
	form_errors.ip = !form.ip;
	form_errors.port = !form.port;
	form_errors.protocol = !form.protocol;

	if (
		form_errors.name ||
		form_errors.ip ||
		form_errors.port ||
		form_errors.protocol
	) {
		save_status.value = "Missing mandatory fields";
		return;
	}

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

		if (editing_path.value) {
			const new_filename = form.name.replace(/ /g, "_") + ".json";
			const old_filename = editing_path.value.split(/[\\/]/).pop();

			if (old_filename !== new_filename) {
				await invoke("delete_profile", { path: editing_path.value });
			}
		}

		save_status.value = result as string;

		if (
			save_status.value.includes("success") ||
			!save_status.value.includes("Error")
		) {
			show_modal.value = false;
			save_status.value = "";
			editing_path.value = null;
			load_profiles();
		}
	} catch (e) {
		save_status.value = "Error: " + e;
	}
}

// Profile List
const profiles = ref<profile_entry[]>([]);
const active_namespaces = ref<namespace_info[]>([]);
const cmd = ref<string>("");

async function load_profiles() {
	try {
		const result = await invoke<profile_entry[]>("list_profiles");
		profiles.value = result.map((p) => ({
			...p,
			ping_status: "Ping",
			port_status: "Port",
			ns_status: "Setup",
			run_status: "Run",
			clean_status: "Clean",
			is_pinging: false,
			is_port_checking: false,
			is_setting_up: false,
			is_running: false,
			is_cleaning: false,
		}));
	} catch (e) {
		alert("Failed to list profiles: " + e);
	}
}

async function refresh_active_namespaces() {
	try {
		active_namespaces.value = await invoke<namespace_info[]>(
			"get_active_namespaces",
		);
	} catch (e) {
		alert("Failed to get namespaces: " + e);
	}
}

// Refresh
let refreshTimer: number | null = null;
onMounted(() => {
	load_profiles();
	refresh_active_namespaces();
	refreshTimer = window.setInterval(() => {
		load_profiles();
		refresh_active_namespaces();
	}, 10000);
});

onUnmounted(() => {
	if (refreshTimer) clearInterval(refreshTimer);
});

// Ping
async function ping_profile(index: number) {
	const p = profiles.value[index];
	p.is_pinging = true;
	p.ping_status = "...";
	try {
		const res = await invoke<string>("ping", { ip: p.profile.ip });
		p.ping_status = `${res}ms`;
	} catch (e) {
		p.ping_status = "No Response: " + e;
	} finally {
		p.is_pinging = false;
	}
}

// Port
async function port_check_profile(index: number) {
	const p = profiles.value[index];
	p.is_port_checking = true;
	p.port_status = "...";
	try {
		const res = await invoke<string>("port", {
			ip: p.profile.ip,
			port: p.profile.port,
		});
		p.port_status = res;
	} catch (e) {
		p.port_status = "No Response: " + e;
	} finally {
		p.is_port_checking = false;
	}
}

// Setup Namespace
async function setup_ns_profile(index: number) {
	const p = profiles.value[index];
	p.is_setting_up = true;
	p.ns_status = "Setting up...";
	try {
		await invoke("setup_namespace", { profilePath: p.path });
		p.ns_status = "Ready";
		refresh_active_namespaces();
	} catch (e) {
		p.ns_status = "Failed: " + e;
	} finally {
		p.is_setting_up = false;
	}
}

// Run
async function run_profile(index: number) {
	const p = profiles.value[index];
	if (!cmd.value) {
		alert("Please enter a command above");
		return;
	}
	p.is_running = true;
	p.run_status = "Launching...";
	try {
		await invoke("run", { profilePath: p.path, cmd: cmd.value });
		p.run_status = "Sent";
		refresh_active_namespaces();
	} catch (e) {
		p.run_status = "Error: " + e;
	} finally {
		setTimeout(() => {
			p.is_running = false;
			p.run_status = "Run Again";
		}, 2000);
	}
}

// Cleanup
async function cleanup_profile(index: number) {
	const p = profiles.value[index];
	p.is_cleaning = true;
	p.clean_status = "Cleaning...";
	try {
		await invoke("cleanup", { profilePath: p.path });
		p.clean_status = "Cleaned";
		refresh_active_namespaces();
	} catch (e) {
		p.clean_status = "Error: " + e;
	} finally {
		p.is_cleaning = false;
	}
}

// Placeholder animation
const name_placeholders = [
	"Proxy1",
	"Server SS Port",
	"Server Sock5 Port",
	"MyVPN",
];
const ip_placeholders = ["192.168.1.1", "fe80::xxxx:xxxx:xxxx:xxxx"];
const port_placeholders = ["8080", "443", "1088"];
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
	"proxyns",
];
const username_placeholders = ["( ) (default)", "user123", "1001", "rc4-md5"];
const password_placeholders = ["( ) (default)"];
const tun_interface_placeholders = ["tun0 (default)"];
const tun_ip_placeholders = ["10.0.0.2 (default)"];
const veth_host_placeholders = ["veth_host (default)"];
const veth_ns_placeholders = ["veth_ns (default)"];
const veth_host_ip_placeholders = ["10.200.1.1 (default)"];
const veth_ns_ip_placeholders = ["10.200.1.2 (default)"];
const cmd_placeholders = [
	"main.py",
	"flatpak run org.mozilla.firefox",
	"steam",
];
</script>

<template>
  <main class="container">
    <div class="header">
        <h2 class="title">Subspace Proxy</h2>
        
        <div class="cmd-bar">
            <label>Command:</label>
            <VanishingInput v-model="cmd" :placeholders="cmd_placeholders" />
        </div>
    </div>

    <div class="panel active-panel">
        <h3>Active Namespaces</h3>
        <div v-if="active_namespaces.length === 0" class="empty-state">No active namespaces found.</div>
        <div v-else class="ns-grid-header">
            <span>Namespace</span>
            <span>Running Processes</span>
        </div>

        <div v-for="ns in active_namespaces" :key="ns.name" class="ns-row">
            <span class="ns-name">{{ ns.name }}</span>
            <span class="ns-procs">{{ ns.processes || '(none)' }}</span>
        </div>
    </div>

    <div class="panel profiles-panel">
        <div class="profiles-header">
            <h3>Saved Profiles</h3>
            <RippleButton @click="new_profile()" class="sm-btn bg-blue-600 text-white">+ New</RippleButton>
        </div>

        <div class="profile-list">
            <div v-for="(p, index) in profiles" :key="p.filename" class="profile-card">
                <div class="card-info">
                    <span class="p-name">{{ p.filename }}</span>
                    <span class="p-detail">{{ (p.profile.protocol || '').toUpperCase() }} : {{ p.profile.ip }}:{{ p.profile.port }}</span>
                </div>
                
                <div class="card-actions">
                    <div class="manage-group">
                        <RippleButton 
                            @click="edit_profile(index)" 
                            class="sm-btn bg-amber-500 text-white"
                        >
                            Edit
                        </RippleButton>

                        <RippleButton 
                            @click="delete_profile_confirm(index)" 
                            class="sm-btn bg-red-600 text-white"
                        >
                            Delete
                        </RippleButton>
                    </div>

                    <div class="separator"></div>

                    <RippleButton 
                        @click="ping_profile(index)" 
                        :disabled="p.is_pinging" 
                        class="sm-btn action-btn"
                        :class="{'button-good': p.ping_status.includes('ms'), 'button-bad': p.ping_status === 'Error'}"
                    >
                        {{ p.ping_status }}
                    </RippleButton>

                    <RippleButton 
                        @click="port_check_profile(index)" 
                        :disabled="p.is_port_checking" 
                        class="sm-btn action-btn"
                        :class="{'button-good': p.port_status === 'Open', 'button-bad': p.port_status === 'Error'}"
                    >
                        {{ p.port_status }}
                    </RippleButton>

                    <RippleButton 
                        @click="setup_ns_profile(index)" 
                        :disabled="p.is_setting_up" 
                        class="sm-btn action-btn bg-purple-600"
                    >
                        {{ p.ns_status }}
                    </RippleButton>

                     <RippleButton 
                        @click="run_profile(index)" 
                        :disabled="p.is_running || !cmd" 
                        class="sm-btn action-btn bg-green-600"
                    >
                        {{ p.run_status }}
                    </RippleButton>

                    <RippleButton 
                        @click="cleanup_profile(index)" 
                        :disabled="p.is_cleaning" 
                        class="sm-btn action-btn bg-red-600"
                    >
                        {{ p.clean_status }}
                    </RippleButton>
                </div>
            </div>
        </div>
    </div>

    <div v-if="show_modal" class="modal-overlay">
      <div class="modal-content">
        <h2>{{ editing_path ? 'Edit Profile' : 'New Proxy Profile' }}</h2>
        <div class="form-grid">
            <div class="form-group">
              <label :class="{'text-red-500': form_errors.name}">Name</label>
              <VanishingInput v-model="form.name" :placeholders="name_placeholders" />
            </div>

            <div class="form-group">
              <label :class="{'text-red-500': form_errors.ip}">IP</label>
              <VanishingInput v-model="form.ip" :placeholders="ip_placeholders" />
            </div>

            <div class="form-group">
              <label :class="{'text-red-500': form_errors.port}">Port</label>
              <VanishingInput v-model="form.port" :placeholders="port_placeholders" />
            </div>

            <div class="form-group">
              <label :class="{'text-red-500': form_errors.protocol}">Protocol</label>
              <DropdownMenu>
                <DropdownMenuTrigger class="w-full">
                  <RippleButton type="button" class="w-full">
                      {{ form.protocol ? form.protocol.toUpperCase() : 'Select Protocol' }}
                  </RippleButton>
                </DropdownMenuTrigger>

                <DropdownMenuContent class="z-[200] bg-white border border-gray-200 shadow-xl min-w-[200px]">
                  <DropdownMenuItem 
                      v-for="type in proxy_types" 
                      :key="type" 
                      @click="form.protocol = type"
                      class="cursor-pointer hover:bg-gray-100 p-2"
                  >
                    {{ type.toUpperCase() }}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
        </div>

        <div class="grid place-content-center p-5">
            <RippleButton type="button" @click="show_advanced_modal = true" class="sm-btn">Advanced Settings (Optional)</RippleButton>
        </div>

        <div v-if="show_advanced_modal" class="modal-overlay">
          <div class="modal-content">
            <h2>Advanced</h2>
            <div class="scroll-form">
                <div class="form-group"><label>DNS</label><VanishingInput v-model="form.dns" :placeholders="dns_placeholders" /></div>
                <div class="form-group"><label>NS Name</label><VanishingInput v-model="form.namespace" :placeholders="namespace_placeholders" /></div>
                <div class="form-group"><label>Username</label><VanishingInput v-model="form.username" :placeholders="username_placeholders" /></div>
                <div class="form-group"><label>Password</label><VanishingInput v-model="form.password" :placeholders="password_placeholders" /></div>
                <div class="form-group"><label>Tun</label><VanishingInput v-model="form.tun_interface" :placeholders="tun_interface_placeholders" /></div>
                <div class="form-group"><label>Tun IP</label><VanishingInput v-model="form.tun_ip" :placeholders="tun_ip_placeholders" /></div>
                <div class="form-group"><label>Veth Host</label><VanishingInput v-model="form.veth_host" :placeholders="veth_host_placeholders" /></div>
                <div class="form-group"><label>Veth NS</label><VanishingInput v-model="form.veth_ns" :placeholders="veth_ns_placeholders" /></div>
                <div class="form-group"><label>Host IP</label><VanishingInput v-model="form.veth_host_ip" :placeholders="veth_host_ip_placeholders" /></div>
                <div class="form-group"><label>NS IP</label><VanishingInput v-model="form.veth_ns_ip" :placeholders="veth_ns_ip_placeholders" /></div>
            </div>
            
            <RippleButton @click="show_advanced_modal = false">Close Advanced</RippleButton>
          </div>
        </div>
        
        <div class="modal-actions">
          <RippleButton @click="show_modal = false" class="bg-red-600">Cancel</RippleButton>
          <RippleButton @click="save_profile" class="bg-blue-600">Save</RippleButton>
          <p>{{ save_status }}</p>
        </div>
      
      </div>
    </div>
  </main>
</template>

<style>
:root {
  font-family: Hack, Arial, sans-serif;
  color: #0f0f0f;
  background-color: #f6f6f6;
}

.container {
  max-width: 900px;
  margin: 0 auto;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.title { 
    text-align: center; margin-bottom: 1rem; 
}

.cmd-bar {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.05);
}

.panel {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 5px rgba(0,0,0,0.05);
}

.profiles-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.ns-grid-header {
    display: grid;
    grid-template-columns: 1fr 2fr;
    font-weight: bold;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #eee;
}

.ns-row {
    display: grid;
    grid-template-columns: 1fr 2fr;
    padding: 0.75rem 0;
    border-bottom: 1px solid #f0f0f0;
}

.ns-name { 
    font-weight: 600; color: #2563eb; 
}

.ns-procs { 
    font-size: 0.9em; color: #555; 
}

.profile-list { 
    display: flex; flex-direction: column; gap: 1rem; 
}

.profile-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #fafafa;
    padding: 1rem;
    border: 1px solid #eee;
    border-radius: 6px;
}

.card-info { 
    display: flex; 
    flex-direction: column; 
    text-align: left; 
}

.p-name { 
    font-weight: bold; 
    font-size: 1.1em; 
}

.p-detail { 
    font-size: 0.85em; 
    color: #666; 
}

.sm-btn {
    padding: 0.4rem 0.8rem !important;
    font-size: 0.85rem !important;
    min-width: 80px;
}

.button-good { 
    background-color: #dcfce7 !important; color: #166534 !important; 
}

.button-bad { 
    background-color: #fee2e2 !important; color: #991b1b !important; 
}

.modal-overlay {
  position: fixed; top: 0; left: 0; width: 100%; height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex; justify-content: center; align-items: center;
  z-index: 100;
}

.modal-content {
  background: white; padding: 2rem; border-radius: 8px;
  width: 500px; max-width: 90%;
  max-height: 90vh; overflow-y: auto;
}

.text-red-500 { 
    color: #ef4444 !important; 
}

.form-grid { 
    display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; 
}

.form-group { 
    margin-bottom: 1rem; text-align: left; 
}

.form-group label { 
    display: block; font-size: 0.8em; margin-bottom: 0.3rem; color: #666; 
}

.modal-actions { 
    display: flex; justify-content: center; gap: 1rem; 
}

.scroll-form { 
    max-height: 60vh; overflow-y: auto; padding-right: 10px; 
}

.manage-group {
    display: flex;
    gap: 0.5rem;
}

.separator {
    width: 1px;
    height: 25px;
    background-color: #ddd;
    margin: 0 0.5rem;
}

.bg-amber-500 { background-color: #f59e0b !important; color: white; }
.text-white { color: white !important; }

.card-actions { 
    display: flex; 
    gap: 0.5rem; 
    align-items: center; 
    flex-wrap: wrap;
    justify-content: flex-end;
}
</style>