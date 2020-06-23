var WebSocket = require('faye-websocket'),
    ws = new WebSocket.Client('ws://localhost:9001/socket');

ws.on('open', function (event) {
    console.log('open');
    var msg = {tag: 'connect', party_id: 0 /*signifies who it's addressed to*/ , message: 'Client3'};
    ws.send(JSON.stringify(msg));
    //ws.send('Clinet1');
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
    //ws.send('Clinet4');
    var msg = {tag: 'communicate', party_id: 2 /*signifies who it's addressed to*/ , message: 'my_message(Clinet3)'};
    ws.send(JSON.stringify(msg));
},5000);
