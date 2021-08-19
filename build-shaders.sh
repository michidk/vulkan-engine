#!/bin/bash

echo "Building HLSL shaders..."

for src in ./shaders/*.hlsl; do
    echo "Compiling shader $src"
    glslc --target-env=vulkan1.2 -fauto-combined-image-sampler -fshader-stage=vert -fentry-point=vert $src -o assets/shaders/$(basename $src .hlsl)-vert.spv
    glslc --target-env=vulkan1.2 -fauto-combined-image-sampler -fshader-stage=frag -fentry-point=frag $src -o assets/shaders/$(basename $src .hlsl)-frag.spv
done
