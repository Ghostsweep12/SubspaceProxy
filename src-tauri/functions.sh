function ping_test {
    # Check if a host is reachable via ping
    local IP=$1
    if ! ping -c 3 -W 5 "$IP"; then
        return 1
    fi
    return 0
}

function port_test {
    # Check if a specific port on a host is open
    local IP=$1
    local PORT=$2
    if ! timeout 1 bash -c "echo > /dev/tcp/$IP/$PORT"; then
        echo "Port $PORT on $IP is closed or unreachable."
        return 1
    fi
    echo "$PORT is open."
    return 0
}

function get_active_namespaces {
    # Returns list of namespaces created by ip netns
    ip netns list | cut -d' ' -f1
}

function get_ns_pids {
    # Get PIDs inside a given namespace
    local NS=$1
    
    local PIDS=$(ip netns pids "$NS" 2>/dev/null)
    
    if [ -z "$PIDS" ]; then
        echo ""
        return
    fi

    for PID in $PIDS; do
        ps -p "$PID" -o comm= 2>/dev/null
    done | sort | uniq | tr '\n' ',' | sed 's/,$//'
}

function setup_namespace {
    # Set up network namespace, virtual ethernet pair, TUN interface, and DNS.
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
    ip netns add "$NS"
    ip netns exec "$NS" ip link set lo up
    #Create link from enviroment to host
    ip link add "$VETH_HOST" type veth peer name "$VETH_NS"
    ip link set "$VETH_NS" netns "$NS"
    # Configure Host Side
    ip addr add "$VETH_HOST_IP/24" dev "$VETH_HOST"
    ip link set "$VETH_HOST" up
    # Configure Namespace Side
    ip netns exec "$NS" ip addr add "$VETH_NS_IP/24" dev "$VETH_NS"
    ip netns exec "$NS" ip link set "$VETH_NS" up
    # Enable IP forwarding so packets can return
    sysctl -w net.ipv4.ip_forward=1 > /dev/null
    # Create network interface for namespace
    ip netns exec "$NS" ip tuntap add mode tun dev "$TUN"
    ip netns exec "$NS" ip addr add "$TUN_IP/24" dev "$TUN"
    ip netns exec "$NS" ip link set "$TUN" up
    # All data gets sent to TUN
    ip netns exec "$NS" ip route add default dev "$TUN"
    # Proxied data leaves
    ip netns exec "$NS" ip route add "$IP" via "$VETH_HOST_IP"
    mkdir -p /etc/netns/"$NS"
    echo "nameserver $DNS" | tee /etc/netns/"$NS"/resolv.conf > /dev/null
}

function run_command_in_namespace {
    # Run command in the namespace with reconstructed user environment
    local NS=$1
    local CMD=$2
    # Manually rebuild the user's environment because sudo/netns strips it.
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
    # Export necessary environment variables and run the command in the namespace
    # sudo -u here to drop privileges back to the real user
    ip netns exec "$NS" bash -c "
        sudo -u '$REAL_USER' bash -c '
            export XDG_RUNTIME_DIR=\"$REAL_XDG_RUNTIME\"
            export DBUS_SESSION_BUS_ADDRESS=\"$DBUS_SOCK\"
            export PULSE_SERVER='$PULSE_SOCK'
            export PULSE_COOKIE='$REAL_HOME/.config/pulse/cookie'
            export DISPLAY=\"$TARGET_DISPLAY\"
            export XAUTHORITY=\"$TARGET_XAUTH\"
            export WAYLAND_DISPLAY=\"$TARGET_WAYLAND\"

            $CMD &
            echo \$!
        '
    "
}

function cleanup {
    # Clean up the created namespace and related resources
    local NS=$1
    local VETH_HOST=$2

    PIDS=$(ip netns pids "$NS" 2>/dev/null)
    if [ -n "$PIDS" ]; then
        kill -9 $PIDS 2>/dev/null || true
    fi

    ip netns delete "$NS" 2>/dev/null || true
    ip link delete "$VETH_HOST" 2>/dev/null || true
    rm -rf /etc/netns/"$NS"
}

function tun2socks_socks5 {
    # Start tun2socks with SOCKS5, TCP/UDP
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local USERNAME=${5:-}
    local PASSWORD=${6:-}
    if [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
        AUTH=""
    else
        AUTH="$USERNAME:$PASSWORD@"
    fi
    ip netns exec "$NS" bash -c "
        set -m
        $TUN2SOCKS \
            -device tun://$TUN \
            -proxy socks5://$IP:$PORT \
            -loglevel warning \
            > /dev/null 2>&1 &
        echo \$!
    "
}

function tun2socks_socks4 {
    # Start tun2socks with SOCKS4, TCP only
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local USERID=${5:-}
    if [ -z "$USERID" ]; then
        USERID=""
    else
        USERID="$USERID@"
    fi
    ip netns exec "$NS" bash -c "
        set -m
        $TUN2SOCKS \
            -device tun://$TUN \
            -proxy socks4://$USERID$IP:$PORT \
            -loglevel warning \
            > /dev/null 2>&1 &
        echo \$!
    "
}

function tun2socks_http {
    # Start tun2socks with HTTP, TCP only
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    ip netns exec "$NS" bash -c "
        set -m
        $TUN2SOCKS \
            -device tun://$TUN \
            -proxy http://$IP:$PORT \
            -loglevel warning \
            > /dev/null 2>&1 &
        echo \$!
    "
}

function tun2socks_shadowsocks {
    # Start tun2socks with Shadowsocks, TCP/UDP
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local METHOD=${5:-}
    local PASSWORD=${6:-}
    if [ -z "$METHOD" ] || [ -z "$PASSWORD" ]; then
        AUTH=""
    else
        AUTH="$METHOD:$PASSWORD@"
    fi
    ip netns exec "$NS" bash -c "
        set -m
        $TUN2SOCKS \
            -device tun://$TUN \
            -proxy ss://$AUTH$IP:$PORT \
            -loglevel warning \
            > /dev/null 2>&1 &
        echo \$!
    "
}

function tun2socks_relay {
    # Start tun2socks with relay, UDP over TCP
    local IP=$1
    local PORT=$2
    local NS=$3
    local TUN=$4
    local USERNAME=${5:-}
    local PASSWORD=${6:-}
    if [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
        AUTH=""
    else
        AUTH="$USERNAME:$PASSWORD@"
    fi
    ip netns exec "$NS" bash -c "
        set -m
        $TUN2SOCKS \
            -device tun://$TUN \
            -proxy relay://$AUTH$IP:$PORT \
            -loglevel warning \
            > /dev/null 2>&1 &
        echo \$!
    "
}

function tun2socks_direct {
    # Start tun2socks with direct connection, for testing, TCP/UDP
    local NS=$1
    local TUN=$2
    ip netns exec "$NS" bash -c "
        set -m
        $TUN2SOCKS \
            -device tun://$TUN \
            -proxy direct:// \
            -loglevel warning \
            > /dev/null 2>&1 &
        echo \$!
    "
}

function tun2socks_reject {
    # Start tun2socks and simply block all outgoing connections, for testing
    local NS=$1
    local TUN=$2
    ip netns exec "$NS" bash -c "
        set -m
        $TUN2SOCKS \
            -device tun://$TUN \
            -proxy reject:// \
            -loglevel warning \
            > /dev/null 2>&1 &
        echo \$!
    "
}