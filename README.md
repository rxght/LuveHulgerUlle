# Batako Todo:

### Long Term
* Add a character
* Add support for map objects (should make chests possible)

### Necessary fixes
* Add a draw() function that replaces register_drawable() and unregister_drawable()

### Possible Optimizations
* Split tile maps into chunks and only render chunks that are visible. (This may be slower in some cases. May only be useful for really large maps)
* Add push descriptor functionality for textures.