function getTile(tiles, x, y) {
  // TODO test
  x -= tiles.x;
  y -= tiles.y;
  if (x >= 0 && y >= 0 && x < tiles.w && y < tiles.h) {
    return tiles.tiles[y][x];
  }
  return null;
}

export { getTile };
