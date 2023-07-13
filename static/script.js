// Manage socket connection
const connectionStatus = document.getElementById("connection-status");
const reconnectButton = document.getElementById("reconnect-button");

let socket;

function connectWebSocket() {
    socket = new WebSocket('ws://192.168.1.17:7878/ws');

    socket.onopen = function () {
        console.log("Connection established");
        connectionStatus.innerHTML = 'Connection status: connected';
        reconnectButton.disabled = true;
    };

    socket.onclose = function () {
        console.log("Connection closed");
        connectionStatus.innerHTML = 'Connection status: disconnected';
        reconnectButton.disabled = false;
    };

    socket.onerror = function () {
        console.log("Connection error");
        connectionStatus.innerHTML = 'Connection status: error';
        reconnectButton.disabled = false;
    };
}

// Reconnect button event listener
reconnectButton.addEventListener('click', function () {
    connectWebSocket();
});


// Move button events

const snakeToPascal = str =>
    str.toLowerCase().replace(/(^[a-z])|([-_][a-z])/g, group =>
        group
            .toUpperCase()
            .replace('-', '')
            .replace('_', '')
    );

function addMoveButtonEvent(id, motion) {
    const button = document.getElementById(id);

    const message = {
        variant: "Move",
        motion: motion,
        speed: "Low",
    };
    const messageJson = JSON.stringify(message);

    const stopMessage = {
        variant: "Move",
        motion: "Stop",
        speed: "Low",
    }
    const stopMessageJson = JSON.stringify(stopMessage);

    button.addEventListener("mousedown", () => socket.send(messageJson));
    button.addEventListener("mouseup", () => socket.send(stopMessageJson));
    button.addEventListener("mouseout", () => socket.send(stopMessageJson));

    button.addEventListener("touchstart", () => socket.send(messageJson));
    button.addEventListener("touchend", () => socket.send(stopMessageJson));
    console.log("Added move button event " + motion + " for button " + id);
}

buttons = ["forward-left", "forward", "forward-right", "right", "backward-right",
    "backward", "backward-left", "left", "left-rot", "right-rot", "stop"];

// Add event listeners to the buttons
buttons.forEach(id => addMoveButtonEvent(id, snakeToPascal(id)));

document.getElementById("date-label").innerHTML = "Date: " + Date();
document.getElementById("refresh-date").addEventListener("click", () => document.getElementById("date-label").innerHTML = "Date: " + Date());

document.getElementById("measure-distance").addEventListener("click", () => socket.send(JSON.stringify({ variant: "MeasureDistance" })));

// Start the socket connection
connectWebSocket();