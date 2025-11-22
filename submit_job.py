import socket
import sys
import time

def send_resp(host, port, payload):
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.connect((host, port))
        s.sendall(payload)
        
        # Wait for response
        response = s.recv(4096)
        print(f"Response: {response.decode('utf-8', errors='replace')}")
        
        s.close()
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 submit_job.py <payload_file>")
        sys.exit(1)

    payload_file = sys.argv[1]
    with open(payload_file, "rb") as f:
        payload = f.read()

    send_resp("127.0.0.1", 6379, payload)
