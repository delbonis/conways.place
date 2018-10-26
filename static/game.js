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
const RENDER_CULL_BUFFER = 3;

const CAM_MOVE_SPEED = 50;
const CAM_ZOOM_MULT = 1.05;

const COLORS = [
	"#000000",
	"#0000ff",
	"#00ff00",
	"#00ffff",
	"#ff0000",
	"#ff00ff",
	"#ffff00",
	"#666666"
];

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
		zoom: DEFAULT_ZOOM / 20.0,
	};

	cameraTarget = {
		x: WORLD_WIDTH / 2,
		y: WORLD_HEIGHT / 2,
		zoom: DEFAULT_ZOOM
	};

	// Set up game UI rendering.
	let ctr = document.getElementById("gamecontainer");
	let game = document.getElementById("game");
	game.width = ctr.clientWidth - 5;
	game.height = ctr.clientHeight - 5;

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
		gWorldCells[u.pos[0]][u.pos[1]] = u.state;
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

function getNewFrame(worldState, cam, windowWidth, windowHeight) {

	// TODO Switch to the lib that lightingkoala talked about?

	let canvas = document.createElement("canvas");
	canvas.width = windowWidth;
	canvas.height = windowHeight;

	let ctx = canvas.getContext("2d");
	ctx.imageSmoothingEnabled = false;

	// Figure out how much we have to render around us.
	let frameRenderedTilesHoriz = (windowWidth / 2) / cam.zoom + RENDER_CULL_BUFFER;
	let frameRenderedTilesVert = (windowHeight / 2) / cam.zoom + RENDER_CULL_BUFFER;
	//console.log("horiz buf: " + frameRenderedTilesHoriz + " vert buf: " + frameRenderedTilesVert);

	// Figure out what the actual mins and maxes are for the viewport, also casting to int.
	let minRenderX = (cam.x - frameRenderedTilesHoriz) | 0;
	let maxRenderX = (cam.x + frameRenderedTilesHoriz) | 0;
	let minRenderY = (cam.y - frameRenderedTilesVert) | 0;
	let maxRenderY = (cam.y + frameRenderedTilesVert) | 0;
	//console.log("rxmin: " + minRenderX + " rxmax: " + maxRenderX + " rymin: " + minRenderY + " rymax: " + maxRenderY);

	// Now camera screen space calculations.
	let xMin = cam.x * cam.zoom - (windowWidth / 2);
	let yMin = cam.y * cam.zoom - (windowHeight / 2);
	//console.log("camxmin: " + xMin + " camymin: " + yMin);

	// Actually render each tile.
	let rendered = 0;
	for (let x = minRenderX; x <= maxRenderX; x++) {
		for (let y = minRenderY; y <= maxRenderY; y++) {

			// Make sure we don't bother rendering things that are out of bounds.
			if (x < 0 || x >= WORLD_WIDTH || y < 0 || y >= WORLD_HEIGHT) {
				continue;
			}

			// Lookup the tile.
			let t = worldState[x][y];

			// Don't render if it is a dead tile, although we might change this at some point.
			if (!t.live) {
				continue;
			}

			// Figure out which color to draw the tile as.
			let color = "#00FF00";
			if (t.data >= 0 && t.data < COLORS.length) {
				color = COLORS[t.data];
			}

			// Calculate positions.
			let tileRenderX = (x * cam.zoom) - xMin;
			let tileRenderY = (y * cam.zoom) - yMin;

			// Actually render the tile.
			ctx.fillStyle = color;
			ctx.fillRect(tileRenderX, tileRenderY, cam.zoom, cam.zoom);
			rendered++;

		}
	}

	if (debug.printTilesRendered) {
		console.log("drew " + rendered + " tiles");
	}

	return canvas;

}

var lastCanvasWidth = -1;
var lastCanvasHeight = -1;

function updateDisplay() {

	let out = document.getElementById("game");
	let frame = getNewFrame(gWorldState, cameraState, out.width, out.height);

	// Cache these since it avoids potentially updating the DOM layout if we don't have to.
	let dw = document.body.clientWidth;
	let dh = document.body.clientHeight;
	if (dw != lastCanvasWidth || dh != lastCanvasHeight) {
		out.width = dw;
		out.height = dh;
		lastCanvasWidth = dw;
		lastCanvasHeight = dh;
	}

	let ctx = out.getContext("2d");
	ctx.imageSmoothingEnabled = false;
	ctx.fillStyle = "#ffffff";
	ctx.fillRect(0, 0, out.width, out.height);
	ctx.drawImage(frame, 0, 0);

}

function lerp(a, b, t) {
	let ct = 0;
	if (t > 1) {
		ct = 1;
	} else if (t < 0) {
		ct = 0;
	} else {
		ct = t;
	}
	return a * (1.0 - ct) + b * ct;
}

var frames = 0;
var lastFrameTime = -1;
var fps = 1.0;

function runFrameUpdate(time) {

	// Do some time math.
	let dt = time - lastFrameTime
	fps = 1000.0 / dt;
	lastFrameTime = time;

	// Don't do any camera manipulation until we've got a stable FPS value.
	if (frames > 2) {

		// Calculate the new position of the camera.
		var nextCam = {};
		for (var p in cameraTarget) {
			nextCam[p] = lerp(cameraState[p], cameraTarget[p], (1.0 / fps) * ZOOM_SPRINGYNESS);
		}

		// Replace the camera state with the new one.
		cameraState = nextCam;

	}

	// Then actually update the canvas.
	updateDisplay();
	frames++;

	// Now set ourselves to be called again on the next frame.
	window.requestAnimationFrame(runFrameUpdate)

}
