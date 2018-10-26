
var invoices = {};

function handleInvoice(id, msg) {
	invoices[id] = msg;
	console.log("new invoice: " + invoices[id]);

	let iventry = document.createElement("li");
	iventry.id = "invoicelist-item-" + id;
	iventry.innerHTML = msg;

	// Add it to the invoice list.
	//let ivs = document.getElementById("invoicelist");
	//ivs.appendChild(iventry);

	// Move the user draw data into a separate thing, and reset it.
	pendingDraws[id] = userDraw;
	userDraw = [];

	console.log(pendingDraws);

	showInvoiceBox(msg);
}

function showInvoiceBox(msg) {
	// Make the new invoice box visible.
	let nibe = document.getElementById("newinvoicebox");
	nibe.style.display = "block";

	// Set it to the newly-added invoice.
	let nie = document.getElementById("newinvoice");
	nie.innerHTML = msg;
}

function handleInvoicePaid(id) {
	if (invoices[id] != null && invoices[id] != undefined) {
		delete invoices[id];
	}

	console.log("invoice paid: " + id);

	dismissNewInvoiceBox();
	delete pendingDraws[id];

	// Just remove it.
	//let iv = document.getElementById("invoicelist-item-" + id);
	//iv.remove();
}

function dismissNewInvoiceBox() {

	// Hide the box.
	let nib = document.getElementById("newinvoicebox");
	nib.style.display = "none";

	// Put a message in the invoice box "just in case" it shows up again.
	let nie = document.getElementById("newinvoice");
	nie.innerHTML = "(No invoice pending.  Dismiss this box.)";
}
