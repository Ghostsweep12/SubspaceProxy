# Subspace Proxy Dev Build

[![status](https://shields.io/badge/status-Developer_Build-green.svg)](https://github.com/Ghostsweep12/SubspaceProxy)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)
![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=%23FFFFFF)
![Vue](https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vuedotjs&logoColor=4FC08D)
[![Tun2Socks](https://shields.io/badge/Tun2Socks-blue.svg)](https://github.com/xjasonlyu/tun2socks)

**Subspace Proxy** is a specialized, lightweight GUI application designed to isolate Linux applications within their own **Network Namespaces**.

It allows you to route specific processes (like Steam, Firefox, or terminal commands) through distinct proxy tunnels (SOCKS5, HTTP, Shadowsocks, etc.) without affecting your system-wide network configuration.

## Features

* **Network Isolation:** Uses Linux Namespaces (`ip netns`) to sandbox applications networking.
* **Protocols:** Connect via SOCKS5, SOCKS4, HTTP, Shadowsocks, and Relay. (More coming soon)
* **Desktop Integration:** Automatically injects PulseAudio, PipeWire, Wayland, X11, and D-Bus environment variables, ensuring **GUI applications** work with sound and video.
* **Tun2Socks Integration:** Tun2Socks converts both TCP and UDP traffic from the namespace into proxy-compatible packets.
* **Diagnostics:** Built-in tools to **Ping** the target server and check **Port** status before launching.
* **Smart Management:** Automatically handles virtual interfaces (`veth`, `tun`), routing tables, and cleanup.

## How It Works

Imagine you computer as a house:

1. **Namespace Creation:** The app creates a new Network Namespace (a sealed network environment). 
> *Like creating a new room in your house.*
2. **Bridging:** It links the namespace to your host system using a **Veth (Virtual Ethernet)** pair. 
> *A door to connect your house and your room.*
3. **Proxying:** Inside the namespace, traffic is routed to a **TUN** interface. The tun2socks utility captures traffic from the TUN interface and forwards it to your specified proxy server. 
> *Imagine a conveyor belt (TUN) leading to a packaging machine (Tun2Socks). Every "letter" is re-addressed in a new package before it leaves.*
4. **Launching:** When you run a command, the app injects your current user's desktop environment (X11/Wayland/Audio) into the namespace so the app behaves normally, but its traffic is forced through the tunnel. 
> *Now when an app in run in the room, all of its mail (Network Packets) is sent through the packaging machine and out the door, and once the recipient (Remote Server) opens the package and mails the real letter, its response can come back to the house.*

## Usage

Before using the appimage, ensure your kernal supports namespace, veth, and tun/tap. As well as having fully functioning `iproute2`, `iputils`, and `bash` utilities.

### 1. Creating a Profile

1. Click **+ New** to create a profile.
2. Enter your Proxy **Name** (cosmetic), **IP**, **Port**, and **Protocol**.
3. (Optional) Configuring authentication (User/Pass) and other specifics in the advanced settings.
4. **Save** the profile.

### 2. Launching an Application

1. **Test Connection:** Use the `Ping` and `Port` buttons to double check the connection that is about to be made.
2. **Setup Environment:** Click the **Setup** button (Purple). This creates the namespace and virtual interfaces.
3. **Enter Command:** In the top command bar, type the application you want to run:
* `firefox`
* `steam`
* `curl ifconfig.me`
4. **Run:** Click the **Run** button (Green) to launch the app. You can re-enter the command and run multiple apps in the same namespace!

### 3. Cleanup

When finished, click **Clean** (Red). This destroys the namespace, deletes the virtual interfaces, and kills any processes within.

## Warning

* **Root Privileges:** This application requires `sudo` permissions to create namespaces and modify network interfaces.
* **Non-Interactive:** Commands are launched non-interactively. You cannot run interactive shells like `bash` or `zsh` that require TTY input.
* **System Modification:** While running, the app enables global IP forwarding (`sysctl -w net.ipv4.ip_forward=1`).
* **Container Conflicts:** May misbehave if run inside Docker or other sandboxed environments due to nested namespace restrictions.

## Developer Installation

This project is built using **Tauri**, **Vue 3**, and **Tun2Socks**.

### Prerequisites

* **Linux** (Kernel with namespace, veth, and tun/tap support)
* **Node.js** & **npm**
* **Rust** & **Cargo**
* `tun2socks`
* `iproute2`
* `iputils`
* `bash`
* `sudo`

### Setup

```bash
# 1. Install dependencies
npm install

# 2. Run in development mode
npm run tauri dev
```

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. See the `LICENSE` file for details.