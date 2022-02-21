const userList = document.querySelector("#users");
const prompt = document.querySelector("#prompt");

const defaultPrompt = "Proszę ✅ czy Państwo rozumiecie";

let proto = document.location.protocol == "http:" ? "ws" : "wss";
const ws = new WebSocket(`${proto}://${document.location.host}/ws`);

let numUsers = 0;

setInterval(() => ws.send("2137"), 5000);

ws.onmessage = (message) => {
    if (message.data.startsWith("your id:")) {
        let id = message.data.split(": ")[1];
        document.querySelector("#yourid").innerHTML = id;
    }
    if (message.data.startsWith("list of users:")) {
        userList.innerHTML = "";
        let users = message.data.trim().split("\n").slice(1);
        numUsers = users.length;

        document.querySelector("#numconnected").innerHTML = numUsers;
        document.querySelector("#votes-total").innerHTML = numUsers;

        let votesCast = calcVotesCast(users);
        let votesGreen = calcVotesGreen(users);

        let percentagePositive = (votesGreen / numUsers).toFixed(2);

        document.querySelector("#votes-cast").innerHTML = votesCast;
        document.querySelector("#votes-percentage").innerHTML = `${percentagePositive * 100}%`;
        prompt.innerHTML = calcVoteResult(percentagePositive);

        users.forEach((user) => {
            let li = document.createElement("p");
            li.append(user);
            userList.appendChild(li);
        });

        if (votesCast == 0) {
            prompt.innerHTML = defaultPrompt;
        }
    }
};

document.querySelector("#green").addEventListener("click", (e) => {
    ws.send("green");
});

document.querySelector("#red").addEventListener("click", (e) => {
    ws.send("red");
});

const calcVotesCast = (users) => users.filter((line) => line.includes("-")).length;
const calcVotesGreen = (users) => users.filter((line) => line.includes("zielone")).length;

const calcVoteResult = (percentagePositive) => {
    if (percentagePositive <= 0.5) return "No nie widzę za dużo zielonego... D-d-dziękuję...";
    if (percentagePositive <= 0.75) return "No jest trochę zielonego... Dziękuję...";
    else return "Dobrze, widzę dużo zielonego, dziękuję!";
};
