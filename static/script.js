const socket = new WebSocket('ws://192.168.1.17:7878/ws')

buttons = ["forwardleft", "forward", "forwardright", "right", "backwardright", "backward", "backwardleft", "left", "leftrot", "rightrot"]

// Add event listeners to the buttons
buttons.forEach(id => {
    console.log("move_" + id);
    document.getElementById(id).addEventListener("mousedown", () => socket.send("move_" + id));
    document.getElementById(id).addEventListener("mouseup", () => socket.send("stop"));

    document.getElementById(id).addEventListener("touchstart", () => socket.send("move_" + id));
    document.getElementById(id).addEventListener("touchend", () => socket.send("stop"));
});

document.getElementById("stop").addEventListener("click", () => socket.send("stop"));
