This folder contains test data to validate the parser.

It should contain a `cubins` subdirectory full of files extracted as follows (for example):

`/usr/local/cuda-10.2/bin/cuobjdump --extract-elf all /path/to/python3.8/site-packages/torch/lib/libtorch_cuda.so`

The above command will provide ~3830 cubin files to test on.

The test data is not checked in because the total amount of data is fairly large.

---

The test suite creates a `cache` subdirectory that stores the cuobjdump output for each cubin. If the files change without the names changing, this cache dir should be deleted.