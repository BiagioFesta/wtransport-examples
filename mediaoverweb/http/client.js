async function connectWebTransport() {
    const videoDecoder = new VideoDecoder({
        output: handleDecodedFrame,
        error: err => { console.log(err); }
    });

    videoDecoder.configure({
        codec: "avc1.42E01E",
    })

    const ip = document.getElementById('ip').value;
    const url = "https://" + ip + ":4433";

    console.log("WebTransport connecting...");
    const transport = new WebTransport(url);
    await transport.ready;
    console.log("WebTransport connected");

    const streams = transport.incomingUnidirectionalStreams.getReader();

    while (true) {
        var { value, done } = await streams.read();

        if (done) {
            break;
        }

        let stream = value.getReader();

        var { value, done } = await stream.read();

        if (done) {
            break;
        }

        let encodedChunk = new EncodedVideoChunk({
            type: 'key',
            data: value,
            timestamp: performance.now(),
        });

        videoDecoder.decode(encodedChunk);
    }
}

async function connectWebSocket() {
    const videoDecoder = new VideoDecoder({
        output: handleDecodedFrame,
        error: err => { console.log(err); }
    });

    videoDecoder.configure({
        codec: "avc1.42E01E",
    })

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
        let packet = new Uint8Array(event.data);

        let encodedChunk = new EncodedVideoChunk({
            type: 'key',
            data: packet,
            timestamp: performance.now(),
        });

        videoDecoder.decode(encodedChunk);
    });
}


function handleDecodedFrame(decodedFrame) {
    const canvasElement = document.getElementById('canvas');
    const ctx = canvasElement.getContext('2d');
    const width = decodedFrame.displayWidth;
    const height = decodedFrame.displayHeight;

    canvasElement.width = width;
    canvasElement.height = height;
    ctx.drawImage(decodedFrame, 0, 0);
}
