// Manage socket connection
const connectionStatus = document.getElementById("connectionStatus");
const reconnectButton = document.getElementById("reconnectButton");

let socket;

function connectWebSocket() {
    socket = new WebSocket('ws://192.168.1.17:7878/ws');

    socket.onopen = function () {
        connectionStatus.innerHTML = 'Connection status: connected';
        reconnectButton.disabled = true;
    };

    socket.onclose = function () {
        connectionStatus.innerHTML = 'Connection status: disconnected';
        reconnectButton.disabled = false;
    };

    socket.onerror = function () {
        connectionStatus.innerHTML = 'Connection status: error';
        reconnectButton.disabled = false;
    };
}

reconnectButton.addEventListener('click', function () {
    connectWebSocket();
});

connectWebSocket();

buttons = ["forwardLeft", "forward", "forwardRight", "right", "backwardRight", "backward", "backwardLeft", "left", "leftRot", "rightRot"]

// Add event listeners to the buttons
buttons.forEach(id => {
    console.log("move_" + id);
    document.getElementById(id).addEventListener("mousedown", () => socket.send("move_" + id));
    document.getElementById(id).addEventListener("mouseup", () => socket.send("stop"));

    document.getElementById(id).addEventListener("touchstart", () => socket.send("move_" + id));
    document.getElementById(id).addEventListener("touchend", () => socket.send("stop"));
});

document.getElementById("stop").addEventListener("click", () => socket.send("stop"));

document.getElementById("dateLabel").innerHTML = "Date: " + Date();
document.getElementById("refreshDate").addEventListener("click", () => document.getElementById("dateLabel").innerHTML = "Date: " + Date());

document.getElementById("measureDistance").addEventListener("click", () => socket.send("measureDistance"));