# Batako

> ### Necessary optimizations
> * All static tiles could be joined into one object drastically reducing rendering time. Animated tiles could be grouped toghether as long as they have the same animation_length and frame_interval.
> * Split tile maps into chunks and only render chunks that are visible.

> ### Necessary fixes
> * Need to fix GenericDrawable::new() so that it doesn't need a shared_id argument. Here's an idea: <href>https://stackoverflow.com/questions/60714284/how-can-i-access-a-functions-calling-location-each-time-its-called</href>
