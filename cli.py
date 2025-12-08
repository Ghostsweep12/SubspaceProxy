def ping():
    pass

def port():
    pass

def dependencies():
    pass

def setup():
    pass

def clean():
    pass

def run():
    pass


while True: 
    print("Welcome to the Proxy App CLI. Type 'exit' to quit. Type help for instructions.")
    command = input("Enter command: ")
    if command.lower() == "exit":
        print("Exiting the CLI.")
        break
    elif command.lower() == "help":
        print("Available commands:\n- exit: Exit the CLI\n- help: Show this help message\n- ping: Check if a host is reachable\n- port: Check if a specific port on a host is open\n- dependencies: Check for required dependencies\n- setup: Setup the proxied environment\n- clean: Clean up and remove the proxied environment\n- run: Run the proxy namespace command (defaults to a new bash instance)")
    elif command.lower() == "ping":
        pass
    elif command.lower() == "port":
        pass
    elif command.lower() == "dependencies":
        pass
    elif command.lower() == "setup":
        pass
    elif command.lower() == "clean":
        pass
    elif command.lower() == "run":
        pass
    else:
        print(f"Unknown command: {command}")