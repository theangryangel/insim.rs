import threading

from insim import _insim

class Client:

    @classmethod
    def tcp(cls, addr="127.0.0.1:29999"):
        client = _insim.tcp(addr)
        return cls(client)

    @classmethod
    def relay(cls, select_host=None, use_websocket=False):
        client = _insim.relay(select_host, use_websocket)
        return cls(client)

    def __init__(self, client):
        # The _client attribute holds our Rust object
        self._client = client
        self._handlers = {} # Dictionary to map packet types to functions
        self._running = False

    def on(self, packet_type):
        """A decorator to register a handler for a specific packet type."""
        def decorator(func):
            # We store the handler function using its type as the key
            self._handlers[packet_type] = func
            return func
        return decorator

    def _listen(self):
        """The internal listening loop that runs in a separate thread."""
        # TODO Do we want to actually spawn _insim.Client here? probably.
        self._running = True
        while self._running:
            try:
                packet = self._client.read()
                handler = self._handlers.get(type(packet))
                if handler:
                    handler(packet)

            except IOError:
                print("Connection lost.")
                self._running = False
            except Exception as e:
                print(f"An error occurred: {e}")
                self._running = False

    def run(self):
        """Starts the client's listening loop in a background thread."""
        thread = threading.Thread(target=self._listen, daemon=True)
        thread.start()
        print("Client is running...")
        # In a real app, you'd have logic to keep the main thread alive
        thread.join()

    def stop(self):
        """Stops the client."""
        self._running = False
