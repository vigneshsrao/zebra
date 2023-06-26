/// List of interesting Integer values. Copy pasted from Fuzzilli
pub const INTERESTING_INTS: [isize ; 61] = [
    -9007199254740993, 9007199254740992, -9007199254740991,           // Smallest integer value that is still precisely representable by a double
    -4294967297, -4294967296, -4294967295,                            // Negative Uint32 max
    -2147483649, -2147483648, -2147483647,                            // Int32 min
    -1073741824, -536870912, -268435456,                              // -2**32 / {4, 8, 16}
    -65537, -65536, -65535,                                           // -2**16
    -4096, -1024, -256, -128,                                         // Other powers of two
    -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 16, 64, -0,             // Numbers around 0
    127, 128, 129,                                                    // 2**7
    255, 256, 257,                                                    // 2**8
    512, 1000, 1024, 4096, 10000,                                     // Misc numbers
    65535, 65536, 65537,                                              // 2**16
    268435456, 536870912, 1073741824,                                 // 2**32 / {4, 8, 16}
    2147483647, 2147483648, 2147483649,                               // Int32 max
    4294967295, 4294967296, 4294967297,                               // Uint32 max
    9007199254740991, 9007199254740992, 9007199254740993,             // Biggest integer value that is still precisely representable by a double
];
