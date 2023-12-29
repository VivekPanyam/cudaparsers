This repo contains parsers for [CUDA binary](https://docs.nvidia.com/cuda/cuda-binary-utilities/index.html#what-is-a-cuda-binary) files. These file formats are not publicly documented so correctly parsing them can be tricky.

## What are CUDA binaries?

These are the GPU executable files generated from compiling CUDA code. They are often embedded within other applications. Among other things, they contain the instructions that are actually executed on GPU.

You may have interacted with these files if you've used any of the [`cuModuleLoad*`](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MODULE.html) functions in the CUDA driver API.

There are two types of CUDA binaries:

- `cubin` files: These run on one particular GPU architecture. This is because each architecture can have different instruction sets and capabilities
- `fatbin` files: These are [fat binaries](https://en.wikipedia.org/wiki/Fat_binary) that contain `cubin` files for several architectures

This repo has well tested support for parsing `cubin` files and basic support for parsing `fatbin` files. See below for more details

The parsers are implemented in Rust, but can be exposed to other languages if there's interest (feel free to create an issue!).

## Why?

`cubin` files contain a lot of useful information that's not exposed by CUDA APIs.

The parsers in this repo were initially built for an internal project that needed detailed information about CUDA modules and kernels. 

## How can you confidently parse an undocumented format?

This post contains a detailed answer: https://blog.vivekpanyam.com/parsing-an-undocumented-file-format

Short version: we test on thousands of `cubin` files to ensure that the output produced by this parser matches the output of [`cuobjdump`](https://docs.nvidia.com/cuda/cuda-binary-utilities/index.html#cuobjdump) from NVIDIA.