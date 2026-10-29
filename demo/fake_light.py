"""Fake Elgato light for the demo. Serves the /elgato/lights API on localhost
and prints a dimmed line whenever elgato-light changes its state."""

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

NAME = "Elgato Key Light Air 7C4F"
PORT = 19123

state = {"on": 0, "brightness": 20, "temperature": 200}


class Handler(BaseHTTPRequestHandler):
    def _send(self):
        body = json.dumps({"numberOfLights": 1, "lights": [state]}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self._send()

    def do_PUT(self):
        length = int(self.headers.get("Content-Length", 0))
        state.update(json.loads(self.rfile.read(length))["lights"][0])
        if state["on"]:
            kelvin = round(1_000_000 / state["temperature"])
            change = f"on, {state['brightness']}%, {kelvin}K"
        else:
            change = "off"
        print(f"\033[2m  💡 {NAME}: {change}\033[0m", flush=True)
        self._send()

    def log_message(self, *args):
        pass


def main():
    # The CLI reads discovered lights from this cache, so it skips Bonjour.
    with open(sys.argv[1], "w") as f:
        json.dump([{"name": NAME, "ip": "127.0.0.1", "port": PORT}], f)
    HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()


if __name__ == "__main__":
    main()
