// ctx.fillStyle = "green";
// ctx.fillRect(10, 10, 150, 100);

function renderTiles(tiles) {
  console.log(tiles);

  const canvas = document.getElementById("canvas");
  const tileSize = 20;
  canvas.width = tiles[0].length * tileSize;
  canvas.height = tiles.length * tileSize;

  const ctx = canvas.getContext("2d");

  for (const [y, row] of tiles.entries()) {
    console.log(row);
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

const socket = new WebSocket("ws://localhost:8080/ws");
socket.onopen = (event) => {
  socket.send("test!");
};
socket.onmessage = (event) => {
  console.log(event.data);
};

// fetch("/tiles?x=0&y=0&w=50&h=50")
//   .then((response) => response.json())
//   .then(function (data) {
//     renderTiles(data);
//   })
//   .catch(function (err) {
//     console.log("Fetch error :-S", err);
//   });
