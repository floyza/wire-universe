const tileSize = 20;

function renderTiles(tiles) {
  const canvas = document.getElementById("canvas");
  canvas.width = tiles[0].length * tileSize;
  canvas.height = tiles.length * tileSize;

  const ctx = canvas.getContext("2d");

  for (const [y, row] of tiles.entries()) {
    for (const [x, tile] of row.entries()) {
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
          alert("ack! renderTiles() error: " + tile);
      }
      ctx.fillRect(x * tileSize, y * tileSize, tileSize, tileSize);
    }
  }
}

const socket = new WebSocket("ws://localhost:3000/ws");
socket.onopen = (event) => {};
socket.onmessage = (event) => {
  let msg = JSON.parse(event.data);
  renderTiles(msg.Refresh.tiles);
};

const canvas = document.getElementById("canvas");
canvas.onclick = (event) => {
  const x = event.pageX - canvas.offsetLeft;
  const y = event.pageY - canvas.offsetTop;
  let tileX = parseInt(x / tileSize, 10);
  let tileY = parseInt(y / tileSize, 10);
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "orange";
  ctx.fillRect(tileX * tileSize, tileY * tileSize, tileSize, tileSize);
  let message = { ModifyCell: { x: tileX, y: tileY, cell: "Wire" } };
  socket.send(JSON.stringify(message));
};
