import subprocess
import os
import json
import shlex

def call_bash_function(function_name, *args):
    # Purpose: Calls a bash function defined in functions.sh with given arguments.
    # Inputs: function_name (str), *args (str)
    # Output: Returns a tuple: (return_code, stdout_output, stderr_output)

    script_dir = os.path.dirname(os.path.abspath(__file__))
    functions_sh_path = os.path.join(script_dir, "functions.sh")

    # Properly shell-escape each argument
    safe_args = " ".join(shlex.quote(str(a)) for a in args)
    # Construct the command to source the functions.sh and then call the function
    command = f"source {shlex.quote(functions_sh_path)} && {function_name} {safe_args}"
    
    try:
        result = subprocess.run(
            ["bash", "-c", command],
            capture_output=True,
            text=True, # Decode stdout/stderr as text
            check=False # Do not raise an exception for non-zero exit codes
        )
        return result.returncode, result.stdout.strip(), result.stderr.strip()
    except FileNotFoundError:
        print("Error: 'bash' command not found. Ensure bash is installed and in your PATH.")
        return -1, "", "bash command not found"
    except Exception as e:
        print(f"An unexpected error occurred: {e}")
        return -1, "", str(e)

# add this interactive caller so interactive shells can use the real terminal
def call_bash_function_interactive(function_name, *args):
    script_dir = os.path.dirname(os.path.abspath(__file__))
    functions_sh_path = os.path.join(script_dir, "functions.sh")
    safe_args = " ".join(shlex.quote(str(a)) for a in args)
    command = f"source {shlex.quote(functions_sh_path)} && {function_name} {safe_args}"
    # Run without capturing output so the child inherits the current terminal (TTY)
    return subprocess.call(["bash", "-c", command])

T2S_PID = None

config = {}
config_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "configuration.json")
if os.path.exists(config_path):
    with open(config_path, "r") as f:
        config = json.load(f)

env = {}
return_code, stdout, stderr = call_bash_function("reconstruct_user_env")
if return_code == 0:
    try:
        env = json.loads(stdout)
    except json.JSONDecodeError:
        print("Failed to parse environment JSON from reconstruct_user_env.")
        print("Raw output:", stdout)
        exit()
environment_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "environment.json")
if os.path.exists(environment_path):
    with open(environment_path, "w") as f:
        json.dump(env, f)

else:
    print(f"Failed to reconstruct user environment: {stderr}")
    exit()

print("Welcome to the Proxy App CLI. Type 'exit' to quit. Type help for instructions.")

while True: 
    command = input("Enter command: ")
    
    if command.lower() == "exit":
        print("Exiting the CLI.")
        exit()

    elif command.lower() == "help":
        print("Commands:\n- exit: Exit the CLI\n- help: Show this help message\n- ping: Check if a host is reachable\n- port: Check if the port on the host is open\n- dependencies: Check for required dependencies\n- setup: Setup the proxied environment\n- clean: Remove the proxied environment\n- run: Run the command in the proxy namespace\n- config: Edit configuration settings")

    elif command.lower() == "ping":
        return_code, stdout, stderr = call_bash_function("ping_test", config["ip"])
        print(stdout)

    elif command.lower() == "port":
        return_code, stdout, stderr = call_bash_function("port_test", config["ip"], config["port"])
        print(stdout)

    elif command.lower() == "dependencies":
        return_code, stdout, stderr = call_bash_function("check_dependencies")
        print(stdout)

    elif command.lower() == "setup":
        return_code, stdout, stderr = call_bash_function(
            "setup", 
            config["ip"], 
            config["port"], 
            config["socks_url"], 
            config["namespace"], 
            config["tun_interface"], 
            config["tun_ip"], 
            config["veth_host"], 
            config["veth_ns"], 
            config["veth_host_ip"], 
            config["veth_ns_ip"]
        )
        print(stdout)
        T2S_PID = stdout[-10:]

    elif command.lower() == "clean":
        return_code, stdout, stderr = call_bash_function("cleanup", config["namespace"], T2S_PID, config["veth_host"])
        print(stdout)

    elif command.lower() == "run":
        # Use an interactive execution so sudo/ip can prompt and the shell inside the namespace is interactive.
        rc = call_bash_function_interactive(
            "run",
            env["REAL_USER"],
            env["REAL_UID"],
            env["REAL_HOME"],
            env["REAL_XDG_RUNTIME"],
            env["PULSE_SOCK"],
            env["DBUS_SOCK"],
            env["TARGET_DISPLAY"],
            env["TARGET_XAUTH"],
            env["TARGET_WAYLAND"],
            config["namespace"],
            config["cmd"]
        )
        if rc != 0:
            print(f"run exited with code {rc}")

    elif command.lower() == "config":
        print("Current configuration:")
        for key, value in config.items():
            print(f"  {key}: {value}")
        key_to_edit = input("Enter the configuration key to edit ('exit' to finish): ")

        if key_to_edit.lower() == "exit":
            continue

        else:
            if key_to_edit in config:
                new_value = input(f"Enter new value for '{key_to_edit}': ")
                config[key_to_edit] = new_value
            else:
                print(f"Key '{key_to_edit}' not found in configuration.")

        with open(config_path, "w") as f:
            json.dump(config, f)
        print("Configuration updated.")

    else:
        print(f"Unknown command: {command}")