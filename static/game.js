/*
 * Hey, you're looking at my code!
 *
 * In case you don't know me, I absolutely dispise JavaScript.  All the backend
 * for this website is written in Rust since it's the coolest thing since sliced
 * bread.
 *
 * Anyways, go ahead and keep digging through this code and have fun!  Don't do
 * anything evil with it please.  https://gitlab.com/treyzania/gameoflight
 */

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

var WORLD_WIDTH = 256;
var WORLD_HEIGHT = WORLD_WIDTH;

const DEFAULT_ZOOM = 10.0;
const ZOOM_SPRINGYNESS = 3;

const CAM_MOVE_SPEED = 50;
const CAM_ZOOM_MULT = 1.05;

var cameraState = null;
var cameraTarget = null;

var gWorldState = getStartingWorldCells();
var gEditWindow = null;

var gSocket = null;
var msgHandlers = {}

var userColor = Math.floor(Math.random() * COLORS.length);

var debug = {
	printTilesRendered: false
};

function init() {

	console.log("hello!");

	// Set up camera.
	cameraState = {
		x: WORLD_WIDTH / 2,
		y: WORLD_HEIGHT / 2,
		zoom: DEFAULT_ZOOM / 5.0,
	};

	cameraTarget = {
		x: WORLD_WIDTH / 2,
		y: WORLD_HEIGHT / 2,
		zoom: DEFAULT_ZOOM
	};

	// Set up game UI rendering.
	worldCanvas = document.createElement("canvas"); // never used directly.
	screenCanvas = document.createElement("canvas");

	// Controls
	// FIXME This acts as if the player is typing.  It's not a smooth flow.
	window.onkeydown = handleKeyDown;

	// Start render procedures.
	updateDisplay();
	window.requestAnimationFrame(runFrameUpdate)

	// Set up message handlers.  See messages.rs for more info.
	msgHandlers["Alert"] = function(sock, m) { alert(m); };
	msgHandlers["Log"] = function(sock, m) { console.log("remote: " + m); };
	msgHandlers["NewWorldState"] = function(sock, m) { applyNewWorldState(m.world) };
	msgHandlers["UpdateCells"] = function(sock, m) { applyWorldUpdates(m) };
	msgHandlers["UpdateEditWindow"] = function(sock, m) { gEditWindow = m; };
	msgHandlers["Invoice"] = function(sock, m) { handleInvoice(m[0], m[1]); };
	msgHandlers["InvoicePaid"] = function(sock, m) { handleInvoicePaid(m); };

	// Set up connection.
	let socket = new WebSocket(getSocketUrl(), "gameoflight");
	gSocket = socket;
	socket.onopen = function(e) { handleSocketOpen(socket, e); };
	socket.onmessage = function(e) { handleSocketMessage(socket, e); };

	console.log("setup finished!");

}

function applyNewWorldState(s) {

	// Update the dimensions we know about.
	WORLD_WIDTH = s.dimensions[0];
	WORLD_HEIGHT = s.dimensions[1];

	// Create a new empty world grid.
	let nw = [];
	for (let i = 0; i < WORLD_WIDTH; i++) {
		let nc = [];
		for (let j = 0; j < WORLD_HEIGHT; j++) {
			nc.push(newBlankTile());
		}
		nw.push(nc);
	}

	// Populate the grid with the tiles from the new state.
	for (let i = 0; i < s.cells.length; i++) {
		let tx = i % WORLD_WIDTH;
		let ty = (i - tx) / WORLD_WIDTH;
		nw[tx][ty] = s.cells[i];
	}

	// Apply it.
	gWorldState = nw;

}

function applyWorldUpdates(updates) {
	for (let i = 0; i < updates.length; i++) {
		let u = updates[i];
		gWorldState[u.pos[0]][u.pos[1]] = u.state;
	}
}

function getSocketUrl() {
	let proto = window.location.protocol;
	if (proto == "file:") {
		/*
		 * We're running it directly, let's hope that the webapp server is
		 * running too.  Normal users will never get here.  Don't worry, 7908
		 * isn't exposed publicly on the server.
		 */
		return "ws://localhost:7908";
	} else {
		/*
		 * We're running on a web server.  This looks more complicated but it's
		 * really not.
		 */
		return "ws://" + window.location.hostname + (window.location.port ? ":" + window.location.port : "") + "/api";
	}
}

function handleSocketOpen(sock, e) {
	console.log("Connected!");
}

function handleSocketMessage(sock, e) {
	let msg = JSON.parse(e.data);
	let handle = msgHandlers[msg.type];
	if (handle != null && handle != undefined) {
		handle(sock, msg.body);
	}
}

function handleKeyDown(e) {
	var k = e.key.toLowerCase();
	if (k == 'a') {
		cameraTarget.x -= CAM_MOVE_SPEED / cameraState.zoom;
	} else if (k == 'd') {
		cameraTarget.x += CAM_MOVE_SPEED / cameraState.zoom;
	} else if (k == 'w') {
		cameraTarget.y -= CAM_MOVE_SPEED / cameraState.zoom;
	} else if (k == 's') {
		cameraTarget.y += CAM_MOVE_SPEED / cameraState.zoom;
	} else if (k == 'e') {
		cameraTarget.zoom *= CAM_ZOOM_MULT;
	} else if (k == 'r') {
		cameraTarget.zoom /= CAM_ZOOM_MULT;
	}
}

function getStartingWorldCells() {
	let w = [];
	for (let i = 0; i < WORLD_HEIGHT; i++) {
		let r = [];
		for (let j = 0; j < WORLD_WIDTH; j++) {
			let t = newBlankTile();
			t.live = (i + j) % 7 == 0;
			t.data = 0;
			r.push(t);
		}
		w.push(r);
	}
	return w;
}

function newBlankTile() {
	return {
		live: false,
		last_update: 0,
		data: 0
	}
}
