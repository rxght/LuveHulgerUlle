# Batako

### Long Term
* Add a character
* Add support for map objects (should make chests possible)

### Necessary fixes
* Make a TileMapLoader singleton class that is responsible creating TileMap structs and loading all the required textures. This would ensure that tileset atlas' only get loaded once and it would also mean that you don't have to load any textures manually.
* Animated tiles are not currently supported with the TileMap struct.

### Possible Optimizations
* Split tile maps into chunks and only render chunks that are visible. (This may be slower in some cases. May only be useful for really large maps)
