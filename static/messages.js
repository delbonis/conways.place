
function sendMessageToServer(type, msg) {
	gSocket.send(JSON.stringify({
		type: type,
		body: msg
	}));
}

// Sends an alert message to other players on the server.
function submitAlert(msg) {
	sendMessageToServer("Alert", msg);
}

// This is ordered in a weird way so that it's easier to call by hand.
function submitTilesLive(cells) {
	let updates = [];
	for (let i = 0; i < cells.length; i += 3) {
		updates.push({
			x: cells[i],
			y: cells[i + 1],
			live: cells[i + 2]
		});
	}

	sendMessageToServer("SubmitTiles", {updates: updates})
}

function __submitGlider(x, y) {
	submitTilesLive([
		x + 1, y,     true,
		x + 2, y + 1, true,
		x,     y + 2, true,
		x + 1, y + 1, true,
		x + 2, y + 2, true
	]);
}

var invoices = {}

function handleInvoice(id, msg) {
	invoices[id] = msg;
	console.log("new invoice: " + invoices[id]);
	announceInvoice(id);
}

// TODO Make this not look like shit.
function announceInvoice(id) {
	alert("Invoice: " + id + "\n\nPlease pay this BOLT 11 invoice\n================\n" + invoices[id]);
}

function handleInvoicePaid(id) {
	alert("Invoice " + id + " paid!");
	if (invoices[id] != null && invoices[id] != undefined) {
		delete invoices[id];
	}
}
