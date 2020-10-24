.global _enable_caches
_enable_caches:
    mrc     p15, 0, r1, c1, c0, 0  // read config register
    orr     r1, r1, #0x1 << 12     // enable instruction cache
    orr     r1, r1, #0x1 << 2      // enable data cache

    mcr     p15, 0, r0, c7, c7, 0  // Invalidate entire instruction + data caches
    mcr     p15, 0, r1, c1, c0, 0  // enable caches

    bx      lr

.global _disable_caches
_disable_caches:
    mrc     p15, 0, r1, c1, c0, 0 // Read config register
    bic     r1, r1, #0x1 << 12    // instruction cache disable
    bic     r1, r1, #0x1 << 2     // data cache disable

    mcr     p15, 0, r1, c1, c0, 0  // disable caches

    bx      lr
