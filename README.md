# Description

This is an ongoing project towards making a game with rust and vulkan.

# Todo

### Necessary fixes
* Pipelines are always shared between all instances of a drawable, this doesn't work when different instances require different pipeline configurations.
Should probably remake the Drawable system to create additional pipeline objects when necessary.
* Needs to do more than one collision-check at a time.

### Long Term
* Add an inventory
* Setup a physics singleton object to handle all physics-related systems
* GameObject struct?

### Possible Optimizations
* Split tile maps into chunks and only render chunks that are visible. (This may be slower in some cases and may only be useful for really large maps)
* Add push descriptor functionality for textures.
* Minimize use of egui widgets. Check how much gpu bandwidth is used by egui widgets to see if it's even necessary.
