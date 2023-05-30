async function connectWebTransport() {
    const counter = document.getElementById("counter");
    const ip = document.getElementById('ip').value;
    const url = "https://" + ip + ":4433";

    console.log("WebTransport connecting...");
    const transport = new WebTransport(url);
    await transport.ready;
    console.log("WebTransport connected");

    const datagrams = transport.datagrams.readable.getReader();
    while (true) {
        let { value, done } = await datagrams.read();

        if (done) {
            console.log("WebTransport connection closed");
            break;
        }

        let number = decodeBigEndianUint64(value);
        counter.textContent = number.toString();
    }
}

async function connectWebSocket() {
    const counter = document.getElementById("counter");
    const ip = document.getElementById('ip').value;
    const url = "wss://" + ip + ":4434";

    console.log("WebSocket connecting...");
    const transport = new WebSocket(url);
    transport.binaryType = 'arraybuffer';

    transport.addEventListener('open', () => {
        console.log("WebSocket connected");
    });

    transport.addEventListener('message', (event) => {
        let value = new Uint8Array(event.data);
        let number = decodeBigEndianUint64(value);
        counter.textContent = number.toString();
    });
}

function decodeBigEndianUint64(byteArray) {
    let value = 0;

    for (let i = 0; i < byteArray.length; i++) {
        value = (value << 8) + byteArray[i];
    }

    return value;
}
