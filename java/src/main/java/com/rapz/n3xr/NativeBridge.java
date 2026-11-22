package com.rapz.n3xr;

/** Pemuat library native dan deklarasi JNI. Tidak melempar jika library tidak ada. */
public final class NativeBridge {
    private static final boolean AVAILABLE;
    private static volatile boolean initialized;

    static {
        boolean ok = false;
        try {
            System.loadLibrary("n3xr_engine");
            ok = true;
        } catch (UnsatisfiedLinkError | SecurityException e) {
            ok = false;
        }
        AVAILABLE = ok;
    }

    private NativeBridge() {}

    /** True jika libn3xr_engine.so berhasil dimuat. */
    public static boolean isAvailable() {
        return AVAILABLE;
    }

    /** Inisialisasi engine native. False jika library tidak tersedia atau gagal. */
    public static synchronized boolean init() {
        if (!AVAILABLE) return false;
        if (initialized) return true;
        try {
            initialized = nativeInit() == 0;
        } catch (UnsatisfiedLinkError e) {
            initialized = false;
        }
        return initialized;
    }

    public static synchronized void shutdown() {
        if (AVAILABLE && initialized) {
            nativeShutdown();
            initialized = false;
        }
    }

    public static boolean isInitialized() {
        return initialized;
    }

    /** Versi engine native, atau null jika tidak tersedia. */
    public static String version() {
        return AVAILABLE ? nativeVersion() : null;
    }

    /**
     * Membuat mesh untuk {@code count} section (masing-masing 4096 ID block).
     * Format hasil dijelaskan di SectionMesher.
     */
    static byte[] buildSections(int[] ids, byte[] flags, int count) {
        return nativeBuildSections(ids, flags, count);
    }

    private static native String nativeVersion();

    private static native int nativeInit();

    private static native void nativeShutdown();

    private static native byte[] nativeBuildSections(int[] ids, byte[] flags, int count);
}