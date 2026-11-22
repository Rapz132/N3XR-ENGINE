package com.rapz.n3xr;

import java.util.Random;

/** Mask face per blok. Bit f (0..5) menyala = face f digambar: -X,+X,-Y,+Y,-Z,+Z. */
public final class FaceMasker {
    public static final int SECTION_VOLUME = 4096;
    public static final byte FLAG_VISIBLE = 1;
    public static final byte FLAG_OPAQUE = 2;

    private static final int[][] DIRS = {
        {-1, 0, 0}, {1, 0, 0}, {0, -1, 0}, {0, 1, 0}, {0, 0, -1}, {0, 0, 1}
    };

    private FaceMasker() {}

    /** Native jika tersedia, selain itu Java. Hasil: count * 4096 byte. */
    public static byte[] build(int[] ids, byte[] flags, int count) {
        validate(ids, flags, count);
        if (NativeBridge.isAvailable()) {
            try {
                byte[] out = buildNative(ids, flags, count);
                if (out != null && out.length == count * SECTION_VOLUME) return out;
            } catch (RuntimeException | UnsatisfiedLinkError e) {
                // jatuh ke Java
            }
        }
        return buildJava(ids, flags, count);
    }

    static byte[] buildNative(int[] ids, byte[] flags, int count) {
        return FaceMaskBridge.nativeBuildFaceMasks(ids, flags, count);
    }

    static byte[] buildJava(int[] ids, byte[] flags, int count) {
        byte[] out = new byte[count * SECTION_VOLUME];
        for (int s = 0; s < count; s++) {
            int off = s * SECTION_VOLUME;
            for (int y = 0; y < 16; y++) {
                for (int z = 0; z < 16; z++) {
                    for (int x = 0; x < 16; x++) {
                        int i = y * 256 + z * 16 + x;
                        int id = ids[off + i];
                        if (id < 0 || id > 0xFFFF) {
                            throw new IllegalArgumentException("block id out of range: " + id);
                        }
                        if ((flagOf(flags, id) & FLAG_VISIBLE) == 0) continue;
                        int mask = 0;
                        for (int f = 0; f < 6; f++) {
                            int nx = x + DIRS[f][0];
                            int ny = y + DIRS[f][1];
                            int nz = z + DIRS[f][2];
                            boolean hidden = false;
                            if (nx >= 0 && nx < 16 && ny >= 0 && ny < 16 && nz >= 0 && nz < 16) {
                                int nid = ids[off + ny * 256 + nz * 16 + nx];
                                hidden = (flagOf(flags, nid) & FLAG_OPAQUE) != 0 || nid == id;
                            }
                            if (!hidden) mask |= 1 << f;
                        }
                        out[off + i] = (byte) mask;
                    }
                }
            }
        }
        return out;
    }

    private static int flagOf(byte[] flags, int id) {
        return (id < 0 || id >= flags.length) ? 0 : (flags[id] & 0xFF);
    }

    private static void validate(int[] ids, byte[] flags, int count) {
        if (ids == null || flags == null) {
            throw new IllegalArgumentException("ids/flags must not be null");
        }
        if (count <= 0 || ids.length != count * SECTION_VOLUME) {
            throw new IllegalArgumentException("ids length must be count * " + SECTION_VOLUME);
        }
    }

    /**
     * Benchmark native vs Java pada terrain acak. Hanya mengukur penghitungan mask;
     * biaya mengubah block state Minecraft menjadi array ID TIDAK ikut terhitung.
     */
    public static String benchmark(int sections, int rounds) {
        int[] ids = new int[sections * SECTION_VOLUME];
        Random r = new Random(42);
        for (int s = 0; s < sections; s++) {
            for (int z = 0; z < 16; z++) {
                for (int x = 0; x < 16; x++) {
                    int h = 6 + r.nextInt(4);
                    for (int y = 0; y < 16; y++) {
                        ids[s * SECTION_VOLUME + y * 256 + z * 16 + x] = y < h ? 1 : 0;
                    }
                }
            }
        }
        byte[] flags = {0, FLAG_VISIBLE | FLAG_OPAQUE, FLAG_VISIBLE | FLAG_OPAQUE, FLAG_VISIBLE};
        boolean nativeOk = NativeBridge.isAvailable();

        for (int i = 0; i < 3; i++) {
            buildJava(ids, flags, sections);
            if (nativeOk) buildNative(ids, flags, sections);
        }

        long javaNs = 0;
        long nativeNs = 0;
        for (int i = 0; i < rounds; i++) {
            long t0 = System.nanoTime();
            buildJava(ids, flags, sections);
            javaNs += System.nanoTime() - t0;
            if (nativeOk) {
                long t1 = System.nanoTime();
                buildNative(ids, flags, sections);
                nativeNs += System.nanoTime() - t1;
            }
        }

        String result = String.format(
            "sections=%d rounds=%d java=%.2fms/round native=%s",
            sections, rounds, javaNs / 1e6 / rounds,
            nativeOk ? String.format("%.2fms/round", nativeNs / 1e6 / rounds) : "unavailable");
        System.out.println("[N3XR] " + result);
        return result;
    }
}