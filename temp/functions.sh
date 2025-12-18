# Goldy, <> = from configuration.json, {} = from environment.json

function ping_test {
    # Purpose: Check if a host is reachable via ping
    # Inputs: <1: IP>
    local IP=$1
    if ! ping -c 3 -W 5 "$IP"; then
        echo "Host $IP is unreachable."
        return 1
    fi
    echo "$IP is reachable."
    return 0
}

function port_test {
    # Purpose: Check if a specific port on a host is open
    # Inputs: <1: IP, 2: Port>
    local IP=$1
    local PORT=$2
    if ! timeout 1 bash -c "echo > /dev/tcp/$IP/$PORT"; then
        echo "Port $PORT on $IP is closed or unreachable."
        return 1
    fi
    echo "$PORT is open."
    return 0
}

function setup_namespace {
    # Purpose: Set up network namespace, virtual ethernet pair, TUN interface, and DNS.
    # Inputs: <1: IP, 2: PORT, 3: NS, 4: TUN, 5: TUN_IP 6: VETH_HOST, 7: VETH_NS, 8: VETH_HOST_IP, 9: VETH_NS_IP, 10: DNS>
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local TUN_IP=$5
    local VETH_HOST=$6
    local VETH_NS=$7
    local VETH_HOST_IP=$8
    local VETH_NS_IP=$9
    local DNS=${10}

    # Create isolated enviroment
    sudo ip netns add "$NS"
    sudo ip netns exec "$NS" ip link set lo up

    #Create link from enviroment to host
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
    sudo ip netns exec "$NS" ip tuntap add mode tun dev "$TUN"
    sudo ip netns exec "$NS" ip addr add "$TUN_IP/24" dev "$TUN"
    sudo ip netns exec "$NS" ip link set "$TUN" up

    echo "[*] Setting up Routing..."
    # All data gets sent to TUN
    sudo ip netns exec "$NS" ip route add default dev "$TUN"
    # Proxied data leaves
    sudo ip netns exec "$NS" ip route add "$IP" via "$VETH_HOST_IP"

    sudo mkdir -p /etc/netns/"$NS"
    echo "nameserver $DNS" | sudo tee /etc/netns/"$NS"/resolv.conf > /dev/null
}

function reconstruct_user_environment {
    # Purpose: Manually rebuild the user's environment because sudo/netns strips it.
    # Outputs: JSON with necessary environment variables (stdout then into environment json)
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

    # Output JSON
    printf '%s\n' "{\"REAL_USER\":\"$REAL_USER\",\"REAL_UID\":$REAL_UID,\"REAL_HOME\":\"$REAL_HOME\",\"REAL_XDG_RUNTIME\":\"$REAL_XDG_RUNTIME\",\"PULSE_SOCK\":\"$PULSE_SOCK\",\"DBUS_SOCK\":\"$DBUS_SOCK\",\"TARGET_DISPLAY\":\"$TARGET_DISPLAY\",\"TARGET_XAUTH\":\"$TARGET_XAUTH\",\"TARGET_WAYLAND\":\"$TARGET_WAYLAND\"}"
}

function run_command_in_namespace {
    # Purpose: Run command in the namespace with reconstructed user environment
    # Inputs: {1: REAL_USER, 2: REAL_UID, 3: REAL_HOME, 4: REAL_XDG_RUNTIME, 5: PULSE_SOCK, 6: DBUS_SOCK, 7: TARGET_DISPLAY, 8: TARGET_XAUTH, 9: TARGET_WAYLAND,} <10: NS, 11: CMD>
    local REAL_USER=$1
    local REAL_UID=$2
    local REAL_HOME=$3
    local REAL_XDG_RUNTIME=$4
    local PULSE_SOCK=$5
    local DBUS_SOCK=$6
    local TARGET_DISPLAY=$7
    local TARGET_XAUTH=$8
    local TARGET_WAYLAND=$9
    local NS=${10}
    local CMD=${11}

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
    # Inputs: <1: namespace,> 2: tun2socks PID, (from setup) <3: veth host name>
    local NS=$1
    local T2S_PID=$2
    local VETH_HOST=$3
    sudo kill $T2S_PID 2>/dev/null
    sudo ip netns delete "$NS" 2>/dev/null
    sudo ip link delete "$VETH_HOST" 2>/dev/null
}

function tun2socks_sock5 {
    # Purpose: Start tun2socks with SOCKS5, TCP/UDP
    # Inputs: <1: IP, 2: PORT, 3: NS, 4: TUN> 5: USERNAME, 6: PASSWORD (5/6 optional)
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local USERNAME=$5
    local PASSWORD=$6

    if [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
        AUTH=""
    else
        AUTH="$USERNAME:$PASSWORD@"
    fi

    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "socks5://$AUTH$IP:$PORT" \
        -loglevel warning & disown

    echo $!
}

function tun2socks_sock4 {
    # Purpose: Start tun2socks with SOCKS4, TCP only
    # Inputs: <1: IP, 2: PORT, 3: NS, 4: TUN> 5: USERID (5 optional)
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local USERID=$5

    if [ -z "$USERID" ]; then
        USERID=""
    else
        USERID="$USERID@"
    fi

    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "socks4://$USERID$IP:$PORT" \
        -loglevel warning & disown

    echo $!
}

function tun2socks_http {
    # Purpose: Start tun2socks with HTTP, TCP only
    # Inputs: <1: IP, 2: PORT, 3: NS, 4: TUN>
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4

    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "http://$IP:$PORT" \
        -loglevel warning & disown

    echo $!
}

function tun2socks_shadowsocks {
    # Purpose: Start tun2socks with Shadowsocks, TCP/UDP
    # Inputs: <1: IP, 2: PORT, 3: NS, 4: TUN> 5: PASSWORD 6: METHOD
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local PASSWORD=$5
    local METHOD=$6

    if [ -z "$METHOD" ] || [ -z "$PASSWORD" ]; then
        AUTH=""
    else
        AUTH="$METHOD:$PASSWORD@"
    fi

    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "ss://$AUTH$IP:$PORT" \
        -loglevel warning & disown

    echo $!
}

function tun2socks_relay {
    # Purpose: Start tun2socks with relay, UDP over TCP
    # Inputs: <1: IP, 2: PORT, 3: NS, 4: TUN> 5: USERNAME, 6: PASSWORD (5/6 optional)
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local USERNAME=$5
    local PASSWORD=$6

    if [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
        AUTH=""
    else
        AUTH="$USERNAME:$PASSWORD@"
    fi

    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "relay://$AUTH$IP:$PORT" \
        -loglevel warning & disown

    echo $!
}

function tun2socks_direct {
    # Purpose: Start tun2socks with direct connection, for testing, TCP/UDP
    # Inputs: <1: NS, 2: TUN>
    local NS=$1
    local TUN=$2

    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "direct://" \
        -loglevel warning & disown

    echo $!
}

function tun2socks_reject {
    # Purpose: Start tun2socks and simply block all outgoing connections, for testing
    # Inputs: <1: NS, 2: TUN>
    local NS=$1
    local TUN=$2

    sudo ip netns exec "$NS" tun2socks \
        -device "tun://$TUN" \
        -proxy "reject://" \
        -loglevel warning & disown

    echo $!
}