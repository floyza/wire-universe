import { getTile } from "./tiles.js";

class Drawer {
  constructor() {
    this.canvas = document.getElementById("world-canvas");
    this.brushCanvas = document.getElementById("brush-canvas");
    this.zoom = 20;
    // x,y = top left x,y of world
    this.viewport = { x: 0, y: 0, w: 600, h: 800 };
    this.brush = "Wire";
    this.brushPos = null; // set if the cursor is poised over a point

    this.canvas.width = this.viewport.w;
    this.canvas.height = this.viewport.h;
    this.brushCanvas.width = this.viewport.w;
    this.brushCanvas.height = this.viewport.h;
  }

  paintTile(x, y, tile, target) {
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
    ctx.fillRect(
      x * this.zoom - this.viewport.x,
      y * this.zoom - this.viewport.y,
      this.zoom,
      this.zoom
    );
  }

  getViewedTileBounds() {
    // I'm unsure of this math. This should be tested
    // TODO test
    let startX = Math.floor(this.viewport.x / this.zoom);
    let endX = Math.floor((this.viewport.x + this.viewport.w) / this.zoom);
    let startY = Math.floor(this.viewport.y / this.zoom);
    let endY = Math.floor((this.viewport.y + this.viewport.h) / this.zoom);
    return { x: startX, y: startY, w: endX - startX + 1, h: endY - startY + 1 };
  }

  renderTiles(tiles) {
    const ctx = this.canvas.getContext("2d");
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    let bounds = this.getViewedTileBounds();
    for (let x = bounds.x; x < bounds.x + bounds.w; ++x) {
      for (let y = bounds.y; y < bounds.y + bounds.h; ++y) {
        let tile = getTile(tiles, x, y);
        if (tile !== null) {
          this.paintTile(x, y, tile, this.canvas);
        }
      }
    }
  }

  setBrush(newBrush) {
    this.brush = newBrush;
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
    if (this.brushPos !== null) {
      self.drawTip(this.brushPos);
    }
  }
  drawTip(pos) {
    let ctx = this.brushCanvas.getContext("2d");
    ctx.clearRect(0, 0, this.brushCanvas.width, this.brushCanvas.height);
    this.paintTile(pos.x, pos.y, this.brush, this.brushCanvas);
  }
}

export { Drawer };
