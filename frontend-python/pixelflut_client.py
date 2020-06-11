import socket
import base64
import time


class BinaryAlgorithms:
    RgbBase64 = "rgb64"
    RgbaBase64 = "rgba64"


class Client():
    sock = None  # type: socket.socket
    x_size = 0  # type: int
    y_size = 0  # type: int

    def __init__(self):
        self.sock = socket.socket()

    def connect(self, hostname, port):
        self.sock.connect((hostname, int(port)))
        self.x_size, self.y_size = self.get_size()

    def get_size(self) -> (int, int):
        self.sock.send(b"SIZE\n")
        response = self.sock.recv(256).decode("ASCII")
        # SIZE $X $Y
        x = response.split(" ")[0]
        y = response.split(" ")[1]

        return (int(x), int(y))

    def set_pixel(self, x: int, y: int, color: str):
        self.sock.send(f"PX {x} {y} {color}\n".encode("ASCII"))

    def get_pixel(self, x: int, y: int) -> str:
        self.sock.send(f"PX {x} {y}\n".encode("ASCII"))
        response = self.sock.recv(256).decode("ASCII")
        # PX $X $X $COLOR

        return response.split(" ")[3]

    def receive_binary(self, algorithm: str) -> list:
        """
        Returns a list of 8-bit integer values.
        Each value being one color channel.
        3 values representing one pixel
        """
        self.sock.send(f"STATE {algorithm}\n".encode("ASCII"))

        response = b''
        while len(response) == 0 or response[-1] != 10:     # 10 is \n
            response += self.sock.recv(256)
        response = response[:-1]        # remove \n

        return base64.b64decode(response)
