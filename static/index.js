const userList = document.querySelector("#users");
let proto = document.location.protocol == "http:" ? "ws" : "wss";
const ws = new WebSocket(`${proto}://${document.location.host}/ws`);

setInterval(() => ws.send("2137"), 5000);

ws.onmessage = (message) => {
    if (message.data.startsWith("your id:")) {
        let id = message.data.split(": ")[1];
        document.querySelector("#yourid").innerHTML = id;
    }
    if (message.data.startsWith("list of users:")) {
        userList.innerHTML = "";
        let users = message.data.trim().split("\n").slice(1);
        users.forEach((user) => {
            let li = document.createElement("p");
            li.append(user);
            userList.appendChild(li);
        });
    }
};

document.querySelector("#green").addEventListener("click", (e) => {
    ws.send("green");
});

document.querySelector("#red").addEventListener("click", (e) => {
    ws.send("red");
});
