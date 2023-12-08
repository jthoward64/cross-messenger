The `nac.c` file is a C version of code from [pypush](https://github.com/JJTech0130/pypush/tree/cc907bc66fe6e4772317a71ea8cb02031e26b5c0/emulated) with a corresponding Rust wrapper.

The pypush Python code was compiled to C using [Cython](https://cython.org/)

Absolutely no work went into minimizing the size of the C code, so it is quite large. It is also not optimized for speed, so it is quite slow. Basically it's quite bad, but it works (I think); so I'm not complaining.
