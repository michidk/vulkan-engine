#!/bin/bash

for src in *.hlsl; do
    echo Compiling Vulkan HLSL shader $src
    glslc.exe --target-env=vulkan1.2 -fauto-combined-image-sampler -fshader-stage=vert -fentry-point=vert $src -o ../assets/shaders/$(basename $src .hlsl)-vert.spv
    glslc.exe --target-env=vulkan1.2 -fauto-combined-image-sampler -fshader-stage=frag -fentry-point=frag $src -o ../assets/shaders/$(basename $src .hlsl)-frag.spv
done

for src in *.rgen.glsl; do
    [ -f "$src" ] || continue
    echo Compiling Raygen shader $src
    glslc.exe --target-env=vulkan1.2 -fshader-stage=rgen $src -o ../assets/shaders/$(basename $src .glsl).spv
done

for src in *.rchit.glsl; do
    [ -f "$src" ] || continue
    echo Compiling Ray closest-hit shader $src
    glslc.exe --target-env=vulkan1.2 -fshader-stage=rchit $src -o ../assets/shaders/$(basename $src .glsl).spv
done

for src in *.rmiss.glsl; do
    [ -f "$src" ] || continue
    echo Compiling Ray miss shader $src
    glslc.exe --target-env=vulkan1.2 -fshader-stage=rmiss $src -o ../assets/shaders/$(basename $src .glsl).spv
done
