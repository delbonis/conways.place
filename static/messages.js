
/*
 * Game of Light, it's satoshis.place but with Conway's Game of Life
 * Copyright (C) 2018 Trey Del Bonis
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
	for (let i = 0; i < cells.length; i += 4) {
		updates.push({
			x: cells[i],
			y: cells[i + 1],
			live: cells[i + 2],
			data: cells[i + 3]
		});
	}

	sendMessageToServer("SubmitTiles", {updates: updates})
}

function __submitGlider(x, y) {
	submitTilesLive([
		x + 1, y,     true, 0,
		x + 2, y + 1, true, 1,
		x,     y + 2, true, 2,
		x + 1, y + 1, true, 3,
		x + 2, y + 2, true, 4
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
