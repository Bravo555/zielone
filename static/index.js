const ws = new WebSocket("ws://localhost:3030/ws");

setInterval(() => ws.send("2137"), 5000);

ws.onmessage((message) => console.log(message));
