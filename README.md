# Description

This is an ongoing project towards making a game with rust and vulkan.

# Todo

### Necessary fixes
* Pipelines are always shared between all instances of a drawable, this doesn't work when different instances require different pipeline configurations.

### Long Term
* Add an inventory
* Add support for map objects (should make chests possible)

### Possible Optimizations
* Split tile maps into chunks and only render chunks that are visible. (This may be slower in some cases and may only be useful for really large maps)
* Add push descriptor functionality for textures.
* Minimize use of egui widgets. Check how much gpu bandwidth is used by egui widgets to see if it's even necessary.
