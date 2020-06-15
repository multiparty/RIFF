var WebSocket = require('faye-websocket'),
    ws = new WebSocket.Client('ws://localhost:9001/socket');

ws.on('open', function (event) {
    console.log('open');
    ws.send('Clinet2');
});

ws.on('message', function (event) {
    console.log('From server message', event.data);
});

ws.on('close', function (event) {
    console.log('close', event.code, event.reason);
    ws = null;
});

setInterval(function() {
    console.log('trying to send something');
    ws.send('Clinet2');
},5000);
