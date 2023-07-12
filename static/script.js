// Function to send a request to the API
async function sendRequest(direction, speed) {
    try {
        const response = await fetch(`/drive?direction=${direction}&speed=medium`, { method: 'POST' });
        if (response.ok) {
            console.log(`Driving ${direction}!`);
        } else {
            console.error("Error sending request!");
        }
    } catch (error) {
        console.error("Error sending request:", error);
    }
}

const socket = new WebSocket('ws://192.168.1.17:7878/ws')

// Add event listeners to the buttons
// document.getElementById("forward-left").addEventListener("click", () => sendRequest("forwardleft", "medium"));
document.getElementById("forward").addEventListener("click", () => socket.send("move_forward"));
// document.getElementById("forward-right").addEventListener("click", () => sendRequest("forwardright", "medium"));
// document.getElementById("right").addEventListener("click", () => sendRequest("right", "medium"));
// document.getElementById("backward-right").addEventListener("click", () => sendRequest("backwardright", "medium"));
// document.getElementById("backward").addEventListener("click", () => sendRequest("backward", "medium"));
// document.getElementById("backward-left").addEventListener("click", () => sendRequest("backwardleft", "medium"));
// document.getElementById("left").addEventListener("click", () => sendRequest("left", "medium"));
// document.getElementById("stop").addEventListener("click", () => sendRequest("stop", "medium"));
// document.getElementById("left-rot").addEventListener("click", () => sendRequest("leftrot", "medium"));
// document.getElementById("right-rot").addEventListener("click", () => sendRequest("rightrot", "medium"));