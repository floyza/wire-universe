const tileSize = 20;
let brush = "Wire";
let tileState = [];

const canvas = document.getElementById("world-canvas");
const brushCanvas = document.getElementById("brush-canvas");
const canvases = document.getElementById("canvases");

function paintTile(x, y, tile, target) {
  const ctx = target.getContext("2d");
  switch (tile) {
    case "Alive":
      ctx.fillStyle = "blue";
      break;
    case "Wire":
      ctx.fillStyle = "orange";
      break;
    case "Empty":
      ctx.fillStyle = "white";
      break;
    case "Dead":
      ctx.fillStyle = "grey";
      break;
    default:
      alert("ack! paintTile() error: " + tile);
  }
  ctx.fillRect(x * tileSize, y * tileSize, tileSize, tileSize);
}

function maybeResizeCanvas(w, h) {
  // if canvas width and height are changed, it blanks the canvas
  if (w !== canvas.width) {
    canvas.width = w;
    brushCanvas.width = w;
  }
  if (h !== canvas.height) {
    canvas.height = h;
    brushCanvas.height = h;
  }
}

function renderTiles() {
  for (const [y, row] of tileState.entries()) {
    for (const [x, tile] of row.entries()) {
      paintTile(x, y, tile, canvas);
    }
  }
}

function setBrush(newBrush) {
  brush = newBrush;
  let wire = document.getElementById("paint-wire");
  let electron = document.getElementById("paint-electron");
  let tail = document.getElementById("paint-tail");
  let blank = document.getElementById("paint-blank");
  switch (newBrush) {
    case "Wire":
      wire.dataset["selected"] = "true";
      electron.dataset["selected"] = "false";
      tail.dataset["selected"] = "false";
      blank.dataset["selected"] = "false";
      break;
    case "Alive":
      wire.dataset["selected"] = "false";
      electron.dataset["selected"] = "true";
      tail.dataset["selected"] = "false";
      blank.dataset["selected"] = "false";
      break;
    case "Dead":
      wire.dataset["selected"] = "false";
      electron.dataset["selected"] = "false";
      tail.dataset["selected"] = "true";
      blank.dataset["selected"] = "false";
      break;
    case "Empty":
      wire.dataset["selected"] = "false";
      electron.dataset["selected"] = "false";
      tail.dataset["selected"] = "false";
      blank.dataset["selected"] = "true";
      break;
    default:
      alert("ack! setBrush error: invalid brush: " + newBrush);
  }
}

{
  let wire = document.getElementById("paint-wire");
  let electron = document.getElementById("paint-electron");
  let tail = document.getElementById("paint-tail");
  let blank = document.getElementById("paint-blank");
  wire.onclick = (_) => {
    setBrush("Wire");
  };
  electron.onclick = (_) => {
    setBrush("Alive");
  };
  tail.onclick = (_) => {
    setBrush("Dead");
  };
  blank.onclick = (_) => {
    setBrush("Empty");
  };
}

const socket = new WebSocket("ws://localhost:3000/ws");
socket.onopen = (event) => {};
socket.onmessage = (event) => {
  let msg = JSON.parse(event.data);
  tileState = msg.Refresh.tiles;
  if (tileState.length > 0) {
    maybeResizeCanvas(
      tileState[0].length * tileSize,
      tileState.length * tileSize
    );
  }
  renderTiles();
};

let dragState = null;

function getCanvasMousePosition(event) {
  return [
    (x = event.pageX - canvases.offsetLeft - brushCanvas.offsetLeft),
    event.pageY - canvases.offsetTop - brushCanvas.offsetTop,
  ];
}

brushCanvas.onmousedown = (event) => {
  let position = getCanvasMousePosition(event);
  dragState = { start: position };
  console.log(event);
};
brushCanvas.onmouseup = (event) => {
  console.log(event);
};
brushCanvas.onclick = (event) => {
  let position = getCanvasMousePosition(event);
  const tileX = parseInt(position[0] / tileSize, 10);
  const tileY = parseInt(position[1] / tileSize, 10);
  paintTile(tileX, tileY, brush, canvas);
  const message = { ModifyCell: { x: tileX, y: tileY, cell: brush } };
  socket.send(JSON.stringify(message));
};
brushCanvas.onmousemove = (event) => {
  const x = event.clientX - canvases.offsetLeft - brushCanvas.offsetLeft;
  const y = event.clientY - canvases.offsetTop - brushCanvas.offsetTop;
  const tileX = parseInt(x / tileSize, 10);
  const tileY = parseInt(y / tileSize, 10);
  let ctx = brushCanvas.getContext("2d");
  ctx.clearRect(0, 0, brushCanvas.width, brushCanvas.height);
  paintTile(tileX, tileY, brush, brushCanvas);
};
brushCanvas.onmouseleave = (_) => {
  let ctx = brushCanvas.getContext("2d");
  ctx.clearRect(0, 0, brushCanvas.width, brushCanvas.height);
};

document.onkeydown = (event) => {
  if (event.key === "a") {
    setBrush("Wire");
  } else if (event.key === "s") {
    setBrush("Alive");
  } else if (event.key === "d") {
    setBrush("Dead");
  } else if (event.key === "f") {
    setBrush("Empty");
  }
};
