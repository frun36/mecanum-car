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

// Start the socket connection
connectWebSocket();

// Move button events

// Convert button id to motion name
const snakeToPascal = str =>
    str.toLowerCase().replace(/(^[a-z])|([-_][a-z])/g, group =>
        group
            .toUpperCase()
            .replace('-', '')
            .replace('_', '')
    );

// Called when move button is clicked
function sendMoveMessage(motion) {
    speed_value = parseFloat(document.getElementById("speed").value) / 100;
    console.log(speed_value);
    const message = {
        message: "Move",
        variant: "Enable",
        motion: motion,
        speed: {
            Manual: speed_value,
        },
    };
    const messageJson = JSON.stringify(message);
    socket.send(messageJson)
}

// Called when Calibrate movement button is clicked
function calibrateMovementStart() {
    minDutyCycle = document.getElementById("min-duty-cycle").value;
    maxDutyCycle = document.getElementById("max-duty-cycle").value;
    step = document.getElementById("step").value;
    measurementsPerRepetition = document.getElementById("measurements-per-repetition").value;
    repetitions = document.getElementById("repetitions").value;

    message = JSON.stringify({
        message: "CalibrateMovement",
        variant: "Start",
        min_duty_cycle: parseFloat(minDutyCycle),
        max_duty_cycle: parseFloat(maxDutyCycle),
        step: parseFloat(step),
        measurements_per_repetition: parseInt(measurementsPerRepetition),
        repetitions: parseInt(repetitions)
    });
    socket.send(message);
    document.getElementById("calibrate-movement-stop").disabled = false;
}

// Adds events to move buttons
function addMoveButtonEvent(id, motion) {
    const button = document.getElementById(id);

    const stopMessage = {
        message: "Move",
        variant: "Disable",
    }
    const stopMessageJson = JSON.stringify(stopMessage);

    // Desktop
    button.addEventListener("mousedown", () => sendMoveMessage(motion));
    button.addEventListener("mouseup", () => socket.send(stopMessageJson));
    button.addEventListener("mouseout", () => socket.send(stopMessageJson));

    // Mobile
    button.addEventListener("touchstart", () => sendMoveMessage(motion));
    button.addEventListener("touchend", () => socket.send(stopMessageJson));

    console.log("Added move button event " + motion + " for button " + id);
}

// Add event listeners to buttons

// Reconnect button
reconnectButton.addEventListener('click', function () {
    connectWebSocket();
});

// Move buttons
move_buttons = ["forward-left", "forward", "forward-right", "right", "backward-right",
    "backward", "backward-left", "left", "left-rot", "right-rot", "stop"];
move_buttons.forEach(id => addMoveButtonEvent(id, snakeToPascal(id)));

// Measure distance button
document.getElementById("measure-distance").addEventListener("click", () => socket.send(JSON.stringify({ message: "MeasureDistance" })));

// Calibrate distance
document.getElementById("calibrate-movement-start").addEventListener("click", () => calibrateMovementStart());
document.getElementById("calibrate-movement-stop").addEventListener("click", () => {
    socket.send(JSON.stringify({
        message: "CalibrateMovement",
        variant: "Stop"
    }));
    document.getElementById("calibrate-movement-stop").disabled = true;
});

// Move distance
document.getElementById("move-distance").addEventListener("click", () => {
    speed_value = parseFloat(document.getElementById("speed").value) / 100;
    const message = {
        message: "Move",
        variant: "Move",
        motion: "Forward",
        speed: {
            Manual: speed_value,
        },
        distance: 0.5
    };
    const messageJson = JSON.stringify(message);
    socket.send(messageJson);
});

// Rotate angle
document.getElementById("rotate-angle").addEventListener("click", () => {
    speed_value = parseFloat(document.getElementById("speed").value) / 100;
    const message = {
        message: "Move",
        variant: "Rotate",
        motion: "RightRot",
        speed: {
            Manual: speed_value,
        },
        angle: 360.0 * (1 + document.getElementById("rotation-slip").value * 0.01)
    };
    const messageJson = JSON.stringify(message);
    socket.send(messageJson);
});

socket.addEventListener("message", (msg) => {
    msg = JSON.parse(msg.data);
    // console.log(msg)
    switch (msg.variant) {
        case "Move":
            console.log(msg.description);
            break;
        case "MeasureDistance":
            console.log(msg.measurement);
            document.getElementById("distance-label").innerHTML = msg.measurement + " m";
            break;
    }
});