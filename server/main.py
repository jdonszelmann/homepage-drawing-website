import websockets
import asyncio
import random
import collections
import json

class Client:
    history = collections.deque(maxlen=1000)
    allsockets = {}

    def __init__(self,socket,old, color):
        self.old = old
        self.color = color
        self.socket = socket
        self.__class__.allsockets[id(self.socket)] = self

    def destroy(self):
        del self.__class__.allsockets[id(self.socket)]
    
    @classmethod
    def remove(cls,socket):
        del cls.allsockets[id(socket)]

    @classmethod
    def get(cls,socket):
        return cls.allsockets[id(socket)]

    def __repr__(self):
        return f"Client: {self.socket} {self.color}"

async def messagehandler(message, sender):
    try:
  
        #  color = hash(sender) % 100
        #  for s in sockets:
        #      await s.send(f"{x},{y},{oldx},{oldy},{color},{len(sockets)}")
        m = json.loads(message);
    
        if m["command"] == "update":
            x,y = float(m["x"]),float(m["y"])
            oldx,oldy = sender.old
            sender.old = (x,y)

            if x == -1 or y == -1: 
                return

            for s in Client.allsockets.values():
                await s.socket.send(json.dumps(
                    {
                        "command":"update",
                        "x":x,
                        "y":y,
                        "oldx":oldx,
                        "oldy":oldy,
                        "color":sender.color,
                        "numonline":len(Client.allsockets)
                    }
                ))

            if oldx != -1 or oldy != -1:
                Client.history.append((x,y,oldx,oldy,sender.color))
            
        elif m["command"] == "history":
            await sender.socket.send(json.dumps(
                {
                    "command":"history",
                    "history":list(Client.history),
                    "numonline":len(Client.allsockets),
                }
            ))

    except Exception as e:
        print(f"An error ({e}) occured while handling a request with contents {message:20s}")

async def handle(websocket, path):
    c = Client(websocket, (-1,-1), random.randint(0,100))
    async for message in websocket:
        await messagehandler(message,c)
    
    Client.remove(websocket)

print("starting")
asyncio.get_event_loop().run_until_complete(
    websockets.serve(handle, '0.0.0.0', 80)
)
asyncio.get_event_loop().run_forever()
