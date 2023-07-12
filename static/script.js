const socket = new WebSocket('ws://192.168.1.17:7878/ws')

// Add event listeners to the buttons
document.getElementById("forward-left").addEventListener("click", () => socket.send("move_forwardleft"));
document.getElementById("forward").addEventListener("click", () => socket.send("move_forward"));
document.getElementById("forward-right").addEventListener("click", () => socket.send("move_forwardright"));
document.getElementById("right").addEventListener("click", () => socket.send("move_right"));
document.getElementById("backward-right").addEventListener("click", () => socket.send("move_backwardright"));
document.getElementById("backward").addEventListener("click", () => socket.send("move_backward"));
document.getElementById("backward-left").addEventListener("click", () => socket.send("move_backwardleft"));
document.getElementById("left").addEventListener("click", () => socket.send("move_left"));
document.getElementById("stop").addEventListener("click", () => socket.send("stop"));
document.getElementById("left-rot").addEventListener("click", () => socket.send("move_leftrot"));
document.getElementById("right-rot").addEventListener("click", () => socket.send("move_rightrot"));