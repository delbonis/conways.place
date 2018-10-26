const RENDER_CULL_BUFFER = 3;

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

var worldCanvas = null;
var screenCanvas = null;

function renderCellsToContext(cells, ctx, minX, minY, maxX, maxY, offX, offY) {

	ctx.imageSmoothingEnabled = false;

	let rendered = 0;

	for (let x = minX; x <= maxX; x++) {

		if (x < 0 || x >= cells.length) {
			continue;
		}

		let col = cells[x];

		for (let y = minY; y <= maxY; y++) {

			if (y < 0 || y >= col.length) {
				continue;
			}

			let cell = col[y];

			// Don't render anything if not alive.  This might change.
			if (!cell.live) {
				continue;
			}

			// Figure out the color, doing error checking.
			let color = "#000000";
			if (cell.data >= 0 && cell.data < COLORS.length) {
				//color = COLORS[cell.data];
			}

			// Then actually render.
			ctx.fillStyle = color;
			ctx.fillRect(x + offX, y + offY, 1, 1); // just 1 pixel
			rendered++;

		}
	}

	if (debug.printTilesRendered) {
		console.log("drew " + rendered + " cells");
	}

	// return the number of actual cells rendered.
	return rendered;

}

function renderGameStateToContext(ctx) {

	// Render the cells in the game, everything is on top of that.
	// TODO Make it not render *everything* in the world.
	renderCellsToContext(gWorldState, ctx, 0, 0, WORLD_WIDTH - 1, WORLD_HEIGHT - 1, 0, 0);

	// Draw the edit window, if there is one.
	if (gEditWindow != null) {
		let ew = gEditWindow;

		// FIXME This ends up making the border blurry because of reasons.
		ctx.fillStyle = "rgba(0, 187, 0, 0.2)";
		ctx.fillRect(ew.xpos, ew.ypos, ew.width, ew.height);
		ctx.strokeStyle = "#222222";
		ctx.strokeRect(ew.xpos, ew.ypos, ew.width, ew.height);
	}

}

var lastWorldWidth = -1;
var lastWorldHeight = -1;

function updateWorldSpaceCanvas() {

	// In case something changed with the world size, update it.
	if (WORLD_WIDTH != lastWorldWidth || WORLD_HEIGHT != lastWorldHeight) {
		worldCanvas.width = WORLD_WIDTH;
		worldCanvas.height = WORLD_HEIGHT;
	}

	// Setup render context.
	let wctx = worldCanvas.getContext("2d");
	wctx.imageSmoothingEnabled = false;

	// Clear it.
	wctx.fillStyle = "#ffffff";
	wctx.fillRect(0, 0, worldCanvas.width, worldCanvas.height);

	// Now redraw everything.
	renderGameStateToContext(wctx);

	// Draw a border so we know where the edges are.
	wctx.fillStyle = "#000000";
	wctx.beginPath()
	wctx.moveTo(0, 0);
	wctx.lineTo(0, worldCanvas.height);
	wctx.lineTo(worldCanvas.width, worldCanvas.height);
	wctx.lineTo(worldCanvas.width, 0);
	wctx.lineTo(0, 0);
	wctx.stroke();

}

function updateScreenSpaceCanvas() {

	// Setup render context.
	let sctx = screenCanvas.getContext("2d");
	sctx.imageSmoothingEnabled = false;
	let screenWidth = screenCanvas.width;
	let screenHeight = screenCanvas.height;

	// Clear it.
	sctx.fillStyle = "#ffffff";
	sctx.fillRect(0, 0, screenWidth, screenHeight);
	sctx.fillStyle = "#000000";

	/*
	 * Now figure out where to put the camera.  This is basically just a bunch
	 * of geometry, it's pretty bad but whatever.  Remember, the "zoom" is
	 * defined as how many pixels the edge of 1 tile should take up.  We can
	 * figure out everything else from there.
	 */

	let zoom = cameraState.zoom;
	let cellsHoriz = screenWidth / zoom + (RENDER_CULL_BUFFER * 2);
	let cellsVert = screenHeight / zoom + (RENDER_CULL_BUFFER * 2);

	let worldX = cameraState.x - (cellsHoriz / 2); // XXX
	let worldY = cameraState.y - (cellsVert / 2); // XXX

	// Actually render it.  (Some transform stack muckery going on here.)
	sctx.save();
	sctx.scale(zoom, zoom);
	sctx.drawImage(worldCanvas, worldX * -1, worldY * -1);
	sctx.restore();

}

var lastCanvasWidth = -1;
var lastCanvasHeight = -1;

function updateDisplay() {

	let out = document.getElementById("game");

	// Cache these since it avoids potentially updating the DOM layout if we don't have to.
	let dw = document.body.clientWidth;
	let dh = document.body.clientHeight;
	if (dw != lastCanvasWidth || dh != lastCanvasHeight) {
		out.width = dw;
		out.height = dh;
		screenCanvas.width = dw;
		screenCanvas.height = dh;
		lastCanvasWidth = dw;
		lastCanvasHeight = dh;
	}

	let ctx = out.getContext("2d");
	ctx.imageSmoothingEnabled = false;

	// First clear it.
	ctx.fillStyle = "#ffffff";
	ctx.fillRect(0, 0, out.width, out.height);

	// Now draw it.
	ctx.drawImage(screenCanvas, 0, 0);

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

		// Apply movements.
		if (keys["d"]) {
			cameraTarget.x += (1.0 / dt) * CAM_MOVE_SPEED / cameraState.zoom;
		}

		if (keys["w"]) {
			cameraTarget.y -= (1.0 / dt) * CAM_MOVE_SPEED / cameraState.zoom;
		}

		if (keys["a"]) {
			cameraTarget.x -= (1.0 / dt) * CAM_MOVE_SPEED / cameraState.zoom;
		}

		if (keys["s"]) {
			cameraTarget.y += (1.0 / dt) * CAM_MOVE_SPEED / cameraState.zoom;
		}

		if (keys["e"]) {
			cameraTarget.zoom *= CAM_ZOOM_MULT;
		}

		if (keys["r"]) {
			cameraTarget.zoom /= CAM_ZOOM_MULT;
		}

		// Calculate the new position of the camera.
		let nextCam = {};
		for (var p in cameraTarget) {
			nextCam[p] = lerp(cameraState[p], cameraTarget[p], (1.0 / fps) * ZOOM_SPRINGYNESS);
		}

		// Replace the camera state with the new one.
		cameraState = nextCam;

	}

	// Then actually update the canvas.
	// (this way of doing it is absolutely awful, I'm so sorry)
	updateWorldSpaceCanvas();
	updateScreenSpaceCanvas();
	updateDisplay();
	frames++;

	// Now set ourselves to be called again on the next frame.
	window.requestAnimationFrame(runFrameUpdate)

}
