# Notes on how to restructure

modules:
- vulkan (currently known as `renderer`): abstracts vulkan, debug stuff
- renderer: meshes, shaders, colors, text rendering
- assets: resource loader, asset handling, parser
- components: lights, camera, world (later to be reworked to ECS)
- utils

workspaces:
- crystal (currently known as `math`) (see crystal branch)
