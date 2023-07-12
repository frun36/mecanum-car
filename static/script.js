const socket = new WebSocket('ws://192.168.1.17:7878/ws')

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