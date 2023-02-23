let zoom = 20;
let brush = "Wire";
let tileState = {};
let dragState = null;
let viewport = { x: 0, y: 0, w: 300, h: 300 };

const canvas = document.getElementById("world-canvas");
canvas.width = viewport.w;
canvas.height = viewport.h;
const brushCanvas = document.getElementById("brush-canvas");
brushCanvas.width = viewport.w;
brushCanvas.height = viewport.h;
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
  ctx.fillRect(viewport.x + x * zoom, viewport.y + y * zoom, zoom, zoom);
}

function maybeResizeCanvas(w, h) {
  // NOTE: if canvas width and height are changed, it blanks the canvas
  if (w !== canvas.width) {
    canvas.width = w;
    brushCanvas.width = w;
  }
  if (h !== canvas.height) {
    canvas.height = h;
    brushCanvas.height = h;
  }
}

function getTile(x, y) {
  if (x >= 0 && y >= 0 && x < tileState.w && y < tileState.h) {
    return tileState.tiles[y][x];
  }
  return null;
}

function renderTiles() {
  const ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  // I'm unsure of this math. This should be tested
  let startX = Math.floor(-viewport.x / zoom);
  let endX = Math.floor((-viewport.x + viewport.w) / zoom);
  let startY = Math.floor(-viewport.y / zoom);
  let endY = Math.floor((-viewport.y + viewport.h) / zoom);
  for (let x = startX; x <= endX; ++x) {
    for (let y = startY; y <= endY; ++y) {
      let tile = getTile(x, y);
      if (tile !== null) {
        paintTile(x, y, tile, canvas);
      }
    }
  }
  // for (const [y, row] of tileState.tiles.entries()) {
  //   for (const [x, tile] of row.entries()) {
  //     paintTile(x, y, tile, canvas);
  //   }
  // }
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
  tileState.x = 0;
  tileState.y = 0;
  tileState.tiles = msg.Refresh.tiles;
  tileState.w = tileState.tiles[0].length;
  tileState.h = tileState.tiles.length;
  // maybeResizeCanvas(tileState.w * zoom, tileState.h * zoom);
  renderTiles();
};

function getCanvasMousePosition(event) {
  return {
    x: event.pageX - canvases.offsetLeft - brushCanvas.offsetLeft,
    y: event.pageY - canvases.offsetTop - brushCanvas.offsetTop,
  };
}

function applyDrag(distX, distY) {
  viewport.x += distX;
  viewport.y += distY;
}

brushCanvas.onmousedown = (event) => {
  let position = getCanvasMousePosition(event);
  dragState = { start: position, state: "still" };
};
brushCanvas.onmouseup = (event) => {
  if (dragState !== null) {
    if (dragState.state === "still") {
      let position = getCanvasMousePosition(event);
      const tileX = Math.floor((position.x - viewport.x) / zoom);
      const tileY = Math.floor((position.y - viewport.y) / zoom);
      paintTile(tileX, tileY, brush, canvas);
      const message = { ModifyCell: { x: tileX, y: tileY, cell: brush } };
      socket.send(JSON.stringify(message));
    }
    dragState = null;
  }
};
brushCanvas.onmousemove = (event) => {
  let position = getCanvasMousePosition(event);
  if (dragState !== null) {
    switch (dragState.state) {
      case "still":
        if (
          Math.sqrt(
            (position.x - dragState.start.x) ** 2 +
              (position.y - dragState.start.y) ** 2
          ) >= 20
        ) {
          // distance >= 20 pixels = drag
          dragState.state = "drag";
          applyDrag(
            position.x - dragState.start.x,
            position.y - dragState.start.y
          );
          dragState.lastPos = position;
          renderTiles();
        }
        break;
      case "drag":
        applyDrag(
          position.x - dragState.lastPos.x,
          position.y - dragState.lastPos.y
        );
        dragState.lastPos = position;
        renderTiles();
        break;
    }
  }
  const tileX = Math.floor((position.x - viewport.x) / zoom);
  const tileY = Math.floor((position.y - viewport.y) / zoom);
  let ctx = brushCanvas.getContext("2d");
  ctx.clearRect(0, 0, brushCanvas.width, brushCanvas.height);
  paintTile(tileX, tileY, brush, brushCanvas);
};
brushCanvas.onmouseleave = (_) => {
  let ctx = brushCanvas.getContext("2d");
  ctx.clearRect(0, 0, brushCanvas.width, brushCanvas.height);
  dragState = null;
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
