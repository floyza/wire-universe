import { Drawer } from "./modules/canvas.js";

const tileBuffer = 5;
let dragState = null;
let mousePos = null;
let currentBounds = null; // for checking if we /need/ to send a new setview
let fracZoom = 20;

let tileState = {};

const canvases = document.getElementById("canvases");

function deepEqual(x, y) {
  // why is this not in javascript?
  // from https://stackoverflow.com/a/32922084
  return x && y && typeof x === "object" && typeof y === "object"
    ? Object.keys(x).length === Object.keys(y).length &&
        Object.keys(x).reduce(function (isEqual, key) {
          return isEqual && deepEqual(x[key], y[key]);
        }, true)
    : x === y;
}

let appDrawer = new Drawer();

function getCanvasMousePosition(pt) {
  return {
    x: pt.x - canvases.offsetLeft - appDrawer.brushCanvas.offsetLeft,
    y: pt.y - canvases.offsetTop - appDrawer.brushCanvas.offsetTop,
  };
}

function globalMousePositionToTile(pt) {
  let x = pt.x - canvases.offsetLeft - appDrawer.brushCanvas.offsetLeft;
  let y = pt.y - canvases.offsetTop - appDrawer.brushCanvas.offsetTop;
  const tileX = Math.floor((x + appDrawer.viewport.x) / appDrawer.zoom);
  const tileY = Math.floor((y + appDrawer.viewport.y) / appDrawer.zoom);
  return {
    x: tileX,
    y: tileY,
  };
}

function applyDrag(drawer, distX, distY) {
  drawer.viewport.x += distX;
  drawer.viewport.y += distY;
  drawer.renderTiles(tileState);
  sendNewBounds(drawer.getViewedTileBounds());
}

function sendNewBounds(bounds) {
  bounds.x -= tileBuffer;
  bounds.y -= tileBuffer;
  bounds.w += tileBuffer * 2;
  bounds.h += tileBuffer * 2;
  if (!deepEqual(bounds, currentBounds)) {
    let viewset = {
      SetView: bounds,
    };
    currentBounds = bounds;
    socket.send(JSON.stringify(viewset));
  }
}

function changeZoomTo(drawer, newZoom) {
  let ratio = newZoom / drawer.zoom;
  drawer.viewport.x =
    (drawer.viewport.x + drawer.viewport.w / 2) * ratio - drawer.viewport.w / 2;
  drawer.viewport.x = Math.round(drawer.viewport.x);
  drawer.viewport.y =
    (drawer.viewport.y + drawer.viewport.h / 2) * ratio - drawer.viewport.h / 2;
  drawer.viewport.y = Math.round(drawer.viewport.y);
  drawer.zoom = newZoom;
  drawer.renderTiles(tileState);
  sendNewBounds(drawer.getViewedTileBounds());
}

appDrawer.brushCanvas.onmousedown = (event) => {
  appDrawer.brushPos = globalMousePositionToTile({
    x: event.pageX,
    y: event.pageY,
  });
  let position = getCanvasMousePosition({ x: event.pageX, y: event.pageY });
  dragState = { start: position, state: "still" };
};
appDrawer.brushCanvas.onmouseup = (event) => {
  if (dragState !== null) {
    if (dragState.state === "still") {
      let tile = globalMousePositionToTile({ x: event.pageX, y: event.pageY });
      appDrawer.paintTile(tile.x, tile.y, appDrawer.brush, appDrawer.canvas);
      const message = {
        ModifyCell: {
          x: tile.x,
          y: tile.y,
          cell: appDrawer.brush,
        },
      };
      socket.send(JSON.stringify(message));
    }
    dragState = null;
  }
  mousePos = null;
};
appDrawer.brushCanvas.onmousemove = (event) => {
  let tile = globalMousePositionToTile({
    x: event.pageX,
    y: event.pageY,
  });
  appDrawer.brushPos = tile;
  let position = getCanvasMousePosition({ x: event.pageX, y: event.pageY });
  if (dragState !== null) {
    switch (dragState.state) {
      case "still":
        if (
          Math.sqrt(
            (position.x - dragState.start.x) ** 2 +
              (position.y - dragState.start.y) ** 2
          ) >= 5
        ) {
          // distance >= 5 pixels = drag
          dragState.state = "drag";
          applyDrag(
            appDrawer,
            dragState.start.x - position.x,
            dragState.start.y - position.y
          );
          dragState.lastPos = position;
        }
        break;
      case "drag":
        applyDrag(
          appDrawer,
          dragState.lastPos.x - position.x,
          dragState.lastPos.y - position.y
        );
        dragState.lastPos = position;
        break;
    }
  }
  let ctx = appDrawer.brushCanvas.getContext("2d");
  ctx.clearRect(
    0,
    0,
    appDrawer.brushCanvas.width,
    appDrawer.brushCanvas.height
  );
  appDrawer.paintTile(tile.x, tile.y, appDrawer.brush, appDrawer.brushCanvas);
};
appDrawer.brushCanvas.onmouseleave = (_) => {
  let ctx = appDrawer.brushCanvas.getContext("2d");
  ctx.clearRect(
    0,
    0,
    appDrawer.brushCanvas.width,
    appDrawer.brushCanvas.height
  );
  dragState = null;
  appDrawer.brushPos = null;
};

window.onkeydown = (event) => {
  if (event.key === "a") {
    appDrawer.setBrush("Wire");
  } else if (event.key === "s") {
    appDrawer.setBrush("Alive");
  } else if (event.key === "d") {
    appDrawer.setBrush("Dead");
  } else if (event.key === "f") {
    appDrawer.setBrush("Empty");
  }
};

appDrawer.brushCanvas.onwheel = (event) => {
  let newZoom = (fracZoom += event.deltaY * -0.01);
  if (newZoom < 1) {
    fracZoom = 1;
    changeZoomTo(appDrawer, 1);
  } else {
    fracZoom = newZoom;
    changeZoomTo(appDrawer, Math.round(fracZoom));
  }
};

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
socket.onopen = (event) => {
  sendNewBounds(appDrawer.getViewedTileBounds());
  socket.send(JSON.stringify("StartStream"));
};

socket.onmessage = (event) => {
  let msg = JSON.parse(event.data);
  tileState.x = msg.Refresh.x;
  tileState.y = msg.Refresh.y;
  tileState.tiles = msg.Refresh.tiles;
  tileState.w = tileState.tiles[0].length;
  tileState.h = tileState.tiles.length;
  appDrawer.renderTiles(tileState);
};
