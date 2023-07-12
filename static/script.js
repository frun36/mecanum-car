const socket = new WebSocket('ws://192.168.1.17:7878/ws')

// Add event listeners to the buttons

document.getElementById("forward-left").addEventListener("mousedown", () => socket.send("move_forwardleft"));
document.getElementById("forward-left").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("forward").addEventListener("mousedown", () => socket.send("move_forward"));
document.getElementById("forward").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("forward-right").addEventListener("mousedown", () => socket.send("move_forwardright"));
document.getElementById("forward-right").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("right").addEventListener("mousedown", () => socket.send("move_right"));
document.getElementById("right").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("backward-right").addEventListener("mousedown", () => socket.send("move_backwardright"));
document.getElementById("backward-right").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("backward").addEventListener("mousedown", () => socket.send("move_backward"));
document.getElementById("backward").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("backward-left").addEventListener("mousedown", () => socket.send("move_backwardleft"));
document.getElementById("backward-left").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("left").addEventListener("mousedown", () => socket.send("move_left"));
document.getElementById("left").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("left-rot").addEventListener("mousedown", () => socket.send("move_leftrot"));
document.getElementById("left-rot").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("right-rot").addEventListener("mousedown", () => socket.send("move_rightrot"));
document.getElementById("right-rot").addEventListener("mouseup", () => socket.send("stop"));

document.getElementById("stop").addEventListener("click", () => socket.send("stop"));
