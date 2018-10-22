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

const WORLD_WIDTH = 1000;
const WORLD_HEIGHT = 1000;

const DEFAULT_ZOOM = 10.0;
const ZOOM_SPRINGYNESS = 3;
const RENDER_CULL_BUFFER = 3;

const CAM_MOVE_SPEED = 50;
const CAM_ZOOM_MULT = 1.05;

var cameraState = null;
var cameraTarget = null;

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

	// TODO Set up world.

	// Set up connection.
	//let socketPort = window.location.port != 80 ? ":" + window.location.port : "";
	//let socketUrl = "ws://" + window.location.hostname + socketPort + "/socket";
	let socketUrl = "ws://localhost:8802/";
	let socket = new WebSocket(socketUrl, "gameoflight");

	socket.onopen = function(e) { handleSocketOpen(socket, e); };
	socket.onmessage = function(e) { handleSocketMessage(socket, e); };

	console.log("setup finished!");

}

function handleSocketOpen(sock, e) {
	console.log("Connected!");
}

function handleSocketMessage(sock, e) {
	console.log("msg: " + e.data);
	let msg = JSON.parse(e.data);
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

function getWorldState() {
	// Just a gider.  It won't work because the GoL logic happens server-side.
	return [
		{
			x: 500,
			y: 500,
			live: true,
			data: 0
		},
		{
			x: 501,
			y: 500,
			live: true,
			data: 0
		},
		{
			x: 502,
			y: 500,
			live: true,
			data: 0
		},
		{
			x: 502,
			y: 499,
			live: true,
			data: 0
		},
		{
			x: 501,
			y: 498,
			live: true,
			data: 0
		}
	];
}

function getNewFrame(worldState, cam, windowWidth, windowHeight) {

	let canvas = document.createElement("canvas");
	canvas.width = windowWidth;
	canvas.height = windowHeight;

	let ctx = canvas.getContext("2d");

	// Figure out how much we have to render around us.
	let frameRenderedTilesHoriz = (windowWidth / 2) / cam.zoom + RENDER_CULL_BUFFER;
	let frameRenderedTilesVert = (windowHeight / 2) / cam.zoom + RENDER_CULL_BUFFER;
	//console.log("horiz buf: " + frameRenderedTilesHoriz + " vert buf: " + frameRenderedTilesVert);

	// Figure out what the actual mins and maxes are for the viewport.
	let minRenderX = cam.x - frameRenderedTilesHoriz;
	let maxRenderX = cam.x + frameRenderedTilesHoriz;
	let minRenderY = cam.y - frameRenderedTilesVert;
	let maxRenderY = cam.y + frameRenderedTilesVert;
	//console.log("rxmin: " + minRenderX + " rxmax: " + maxRenderX + " rymin: " + minRenderY + " rymax: " + maxRenderY);

	// Now camera screen space calculations.
	let xMin = cam.x * cam.zoom - (windowWidth / 2);
	let yMin = cam.y * cam.zoom - (windowHeight / 2);
	//console.log("camxmin: " + xMin + " camymin: " + yMin);

	// Actually render each tile.
	worldState.forEach(function(ele) {
		if (!ele.live) {
			return; // nobody cares about it so whatever
		}

		let tx = ele.x;
		let ty = ele.y;

		//console.log("tile x: " + tx + " y: " + ty);

		if (tx >= minRenderX && tx <= maxRenderX && ty >= minRenderY && ty <= maxRenderY) {

			let tileRenderX = (tx * cam.zoom) - xMin;
			let tileRenderY = (ty * cam.zoom) - yMin;

			//console.log("tile renderx: " + tileRenderX + " rendery: " + tileRenderY);

			ctx.fillStyle = "#000000";
			ctx.fillRect(tileRenderX, tileRenderY, cam.zoom, cam.zoom);

		}
	});

	return canvas;

}

function updateDisplay() {

	let out = document.getElementById("game");
	let frame = getNewFrame(getWorldState(), cameraState, out.width, out.height);

	let ctx = out.getContext("2d");
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
