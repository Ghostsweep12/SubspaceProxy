function ping_test {
    # Purpose: Check if a host is reachable via ping
    # Inputs: 1: IP address
    local IP=$1
    echo "[*] Checking reachability of $IP..."
    if ! ping -c 3 -W 5 "$IP" > /dev/null 2>&1; then
        echo -e "\e[31m [!] Host $IP is unreachable.\e[0m"
        return 1
    fi
    echo -e "\e[32m[+] $IP is reachable\e[0m"
    return 0
}

function port_test {
    # Purpose: Check if a specific port on a host is open
    # Inputs: 1: IP address, 2: Port number
    local IP=$1
    local PORT=$2
    echo "[*] Checking if $PORT on $IP is open..."
    if ! timeout 1 bash -c "echo > /dev/tcp/$IP/$PORT" 2 > /dev/null; then
        echo -e "\e[31m[!] Port $PORT on $IP is closed or unreachable.\e[0m"
        return 1
    fi
    echo -e "\e[32m[+] $PORT is open\e[0m"
    return 0
}

function check_dependencies {
    # Purpose: Check for required dependencies
    echo "[*] Checking for tun2socks..."
    if ! command -v tun2socks > /dev/null 2>&1; then
        echo -e "\e[31m[!] tun2socks is not installed."
        echo "Install it with: sudo pacman -S tun2socks (or your distro equivalent)\e[0m"
        return 1
    fi
    echo -e "\e[32m[+] tun2socks found.\e[0m"
    return 0
}

function setup {
    local IP=$1
    local PORT=$2
    local SOCKS_URL=$3
    local NS=$4
    local TUN=$5
    local VETH_HOST=$6
    local VETH_NS=$7
    local VETH_HOST_IP=$8
    local VETH_NS_IP=$9
    local TUN_IP=${10}
    # Create isolated enviroment
    echo "[*] Creating Namespace: $NS"
    sudo ip netns add "$NS"
    sudo ip netns exec "$NS" ip link set lo up

    #Create link from enviroment to host
    echo "[*] Creating Virtual Ethernet Pair..."
    sudo ip link add "$VETH_HOST" type veth peer name "$VETH_NS"
    sudo ip link set "$VETH_NS" netns "$NS"

    # Configure Host Side
    sudo ip addr add "$VETH_HOST_IP/24" dev "$VETH_HOST"
    sudo ip link set "$VETH_HOST" up

    # Configure Namespace Side
    sudo ip netns exec "$NS" ip addr add "$VETH_NS_IP/24" dev "$VETH_NS"
    sudo ip netns exec "$NS" ip link set "$VETH_NS" up

    # Enable IP forwarding so packets can return
    sudo sysctl -w net.ipv4.ip_forward=1 > /dev/null

    # Create network interface for namespace
    echo "[*] Creating TUN interface..."
    sudo ip netns exec "$NS" ip tuntap add mode tun dev "$TUN"
    sudo ip netns exec "$NS" ip addr add "$TUN_IP/24" dev "$TUN"
    sudo ip netns exec "$NS" ip link set "$TUN" up

    echo "[*] Setting up Routing..."
    # All data gets sent to TUN
    sudo ip netns exec "$NS" ip route add default dev "$TUN"
    # Proxied data leaves
    sudo ip netns exec "$NS" ip route add "$IP" via "$VETH_HOST_IP"

    # Start tun2socks on TUN to proxy traffic
    echo "[*] Starting tun2socks..."
    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "$SOCKS_URL" \
        -loglevel warning &

    T2S_PID=$!

    echo "[*] Configuring DNS..."
    sudo mkdir -p /etc/netns/"$NS"
    echo "nameserver 8.8.8.8" | sudo tee /etc/netns/"$NS"/resolv.conf > /dev/null

    echo "$T2S_PID"
}

function reconstruct_user_env {
    # Purpose: Manually rebuild the user's environment because sudo/netns strips it.
    # Outputs: dict of necessary environment variables
    REAL_USER=${SUDO_USER:-$USER}
    REAL_UID=$(id -u "$REAL_USER")
    REAL_HOME=$(getent passwd "$REAL_USER" | cut -d: -f6)
    REAL_XDG_RUNTIME="/run/user/$REAL_UID"

    # PulseAudio/PipeWire variables
    if [ -e "$REAL_XDG_RUNTIME/pulse/native" ]; then
        PULSE_SOCK="unix:$REAL_XDG_RUNTIME/pulse/native"
    else
        PULSE_SOCK="unix:$REAL_XDG_RUNTIME/pipewire-0"
    fi

    # D-Bus socket
    DBUS_SOCK="unix:path=$REAL_XDG_RUNTIME/bus"

    # Display variables
    TARGET_DISPLAY="${DISPLAY:-:0}"
    TARGET_XAUTH="${XAUTHORITY:-$REAL_HOME/.Xauthority}"
    TARGET_WAYLAND="${WAYLAND_DISPLAY:-wayland-0}"

    echo "$output"
}

function run {
    # Purpose: Run command in the namespace with reconstructed user environment
    # Inputs: 1: dict of user env variables from reconstruct_user_env, 2: namespace, 3: command to run
    local REAL_USER, REAL_UID, REAL_HOME, REAL_XDG_RUNTIME, PULSE_SOCK, DBUS_SOCK, TARGET_DISPLAY, TARGET_XAUTH, TARGET_WAYLAND = $1
    local NS=$2
    local CMD=$3
    # Export necessary environment variables and run the command in the namespace
    sudo ip netns exec "$NS" sudo -u "$REAL_USER" bash -c "
        export XDG_RUNTIME_DIR='$REAL_XDG_RUNTIME'
        export DBUS_SESSION_BUS_ADDRESS='$DBUS_SOCK'
        export PULSE_SERVER='$PULSE_SOCK'
        export PULSE_COOKIE='$REAL_HOME/.config/pulse/cookie'
        export DISPLAY='$TARGET_DISPLAY'
        export XAUTHORITY='$TARGET_XAUTH'
        export WAYLAND_DISPLAY='$TARGET_WAYLAND'
        
        exec $CMD
    "
}

function cleanup {
    # Purpose: Clean up the created namespace and related resources
    # Inputs: 1: namespace, 2: tun2socks PID, 3: veth host name
    local NS=$1
    local T2S_PID=$2
    local VETH_HOST=$3
    echo -e "\e[34m[-] Cleaning up..."
    sudo kill $T2S_PID 2>/dev/null
    sudo ip netns delete "$NS" 2>/dev/null
    sudo ip link delete "$VETH_HOST" 2>/dev/null
    echo -e "[-] Done.\e[0m"
}