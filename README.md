# Batako Todo:

### Long Term
* Add a character
* Add support for map objects (should make chests possible)

### Necessary fixes
* Change TileMap struct to use a texture array instead of a normal texture
* Add a draw() function that replaces register_drawable() and unregister_drawable()

### Possible Optimizations
* Split tile maps into chunks and only render chunks that are visible. (This may be slower in some cases. May only be useful for really large maps)
