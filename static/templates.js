var currentTemplateName = "single";
var templateFlip = false; // TODO
var templateRotate = 0; // TODO

var userCurrentTemplate = [{x: 0, y: 0}]; // TODO More templates.

function setUserPenStyle(name) {
	currentTemplateName = name;
	updateUserPen();
}

function setUserPenFlip(flip) {
	templateFlip = flip;
	updateUserPen();
}

function setUserPenRot(quarterturns) {
	templateRotate = quarterturns;
	updateUserPen();
}

function updateUserPen() {
	let t = TEMPLATES[currentTemplateName];
	let ut = [];
	for (let i = 0; i < t.length; i++) {
		let tc = t[i];
		if (templateFlip) {
			tc = mirrorPointHoriz(tc);
		}
		for (let j = 0; j < templateRotate; j++) {
			tc = rotatePointCcw(tc);
		}
		ut.push(tc);
	}
	userCurrentTemplate = ut;
}

function rotatePointCcw(p) {
	return {
		x: p.y,
		y: p.x * -1
	};
}

function mirrorPointHoriz(p) {
	return {
		x: p.x,
		y: p.y * -1
	};
}

var GLIDER_GUN_TEXT = "........................O.................................O.O.......................OO......OO............OO...........O...O....OO............OOOO........O.....O...OO..............OO........O...O.OO....O.O.....................O.....O.......O......................O...O................................OO......................";

var TEMPLATES = {
	"none" : [],
	"single": [
		{x: 0, y: 0}
	],
	"glider": [
		{x: 0, y: -1},
		{x: 1, y: 0},
		{x: -1, y: 1},
		{x: 0, y: 1},
		{x: 1, y: 1}
	],
	"rpent": [
		{x: 0, y: 0},
		{x: -1, y: 0},
		{x: 0, y: -1},
		{x: 1, y: -1},
		{x: 0, y: 1}
	],
	"glidergun": parseTextDesc(GLIDER_GUN_TEXT, 36, 9, "O", 18, 4)
}

function parseTextDesc(text, width, height, cellChar, centerX, centerY) {
	let cells = [];
	for (let x = 0; x < width; x++) {
		for (let y = 0; y < height; y++) {
			let i = y * width + x;
			let c = text.charAt(i);
			console.log(i + " " + c);
			if (c == cellChar) {
				cells.push({
					x: x - centerX,
					y: y - centerY
				});
			}
		}
	}
	console.log("cells: " + cells.length);
	return cells;
}
