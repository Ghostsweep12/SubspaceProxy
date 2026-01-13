## Experimental Test Build

Linux only. Sudo required in terminal for some commands. Run with **npm run tauri dev**.

### Execution model
- Commands are launched **non-interactively**
- No TTY / no interactive stdin/stdout
- Interactive shells (bash, zsh, fish) will not work

### GUI applications
- GUI applications **are supported**
- X11 / Wayland apps such as Steam and Firefox work
- The user’s desktop session (DISPLAY, audio, D-Bus) is reused

### Requirements
- GNU bash
- sudo (temporary; will be replaced by a privilege helper)
- iproute2
- iputils
- coreutils
- procps-ng

### Kernel requirements
- Network namespaces
- Virtual ethernet (veth)
- TUN/TAP support

### Warning
- Temporarily modifies system networking 
- IP forwarding is enabled globally while running (sysctl -w net.ipv4.ip_forward=1)
- May misbehave in containers or sandboxed environments