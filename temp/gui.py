import subprocess
import os
import json
import shlex

def call_bash_function(function_name, *args):
    # Purpose: Calls a bash function defined in functions.sh with given arguments and returns, for other functions.
    # Inputs: function_name (str), *args (str)
    # Output: Returns a tuple: (return_code, stdout_output, stderr_output)

    script_dir = os.path.dirname(os.path.abspath(__file__))
    functions_sh_path = os.path.join(script_dir, "functions.sh")
    safe_args = " ".join(shlex.quote(str(a)) for a in args)
    command = f"source {shlex.quote(functions_sh_path)} && {function_name} {safe_args}"
    result = subprocess.run(
        ["bash", "-c", command],
        capture_output=True,
        text=True,
        check=False
    )
    return result.returncode, result.stdout.strip(), result.stderr.strip()

def call_bash(function_name, *args):
    # Calls a bash function interactively, inheriting the current terminal (TTY), for the run function.
    script_dir = os.path.dirname(os.path.abspath(__file__))
    functions_sh_path = os.path.join(script_dir, "functions.sh")
    safe_args = " ".join(shlex.quote(str(a)) for a in args)
    command = f"source {shlex.quote(functions_sh_path)} && {function_name} {safe_args}"
    return subprocess.call(["bash", "-c", command])

config = {}
config_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "proxy_configuration.json")
if os.path.exists(config_path):
    with open(config_path, "r") as f:
        config = json.load(f)

env = {}
return_code, stdout, stderr = call_bash_function("reconstruct_user_env")
env = json.loads(stdout)
environment_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "environment.json")
with open(environment_path, "w") as f:
    json.dump(env, f)

T2S_PID = None

